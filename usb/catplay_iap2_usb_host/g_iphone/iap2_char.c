// SPDX-License-Identifier: GPL-2.0
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/usb.h>
#include <linux/miscdevice.h>
#include <linux/fs.h>
#include <linux/uaccess.h>
#include <linux/slab.h>
#include <linux/poll.h>
#include <linux/wait.h>
#include <linux/spinlock.h>
#include <linux/completion.h>
#include <linux/kref.h>
#include <linux/idr.h>
#include <linux/list.h>
#include <linux/workqueue.h>

#define DRV_NAME            "iap2_char"
#define IAP2_MAX_XFER       65535U
#define IAP2_TX_INFLIGHT    3
#define IAP2_RX_INFLIGHT    3
#define IAP2_RX_BACKLOG_MAX 65535U
#define IAP2_WORK_RX_CLEAR_HALT 0
#define IAP2_WORK_TX_CLEAR_HALT 1

struct iap2_dev;

struct iap2_rx_record {
    struct list_head node;
    size_t len;
    size_t off;
    u8 *buf;
};

struct iap2_tx_slot {
    struct iap2_dev *dev;
    struct urb *urb;
    u8 *buf;
    int index;

    struct completion done;
    int status;
    bool in_use;
};

struct iap2_rx_slot {
    struct iap2_dev *dev;
    struct urb *urb;
    u8 *buf;
    int index;
    bool submitted;
};

struct iap2_dev {
    struct usb_device *udev;
    struct usb_interface *intf;

    struct kref kref;
    bool disconnected;

    u8 ep_in;
    u8 ep_out;

    struct miscdevice miscdev;
    int minor_id;
    char name[32];
    bool sysfs_link_added;

    /* open/close */
    atomic_t open_count;

    /* TX slot allocation */
    spinlock_t tx_lock;
    wait_queue_head_t tx_wq;
    struct iap2_tx_slot tx[IAP2_TX_INFLIGHT];
    int tx_async_error;

    /* RX completed records */
    spinlock_t rx_lock;
    wait_queue_head_t rx_wq;
    struct list_head rx_done_list;
    size_t rx_queued_bytes;
    int rx_submit_error;
    struct iap2_rx_slot rx[IAP2_RX_INFLIGHT];

    struct work_struct rx_clear_halt_work;
    struct work_struct tx_clear_halt_work;
    unsigned long work_flags;
};

static DEFINE_IDA(iap2_ida);

static void iap2_rx_complete(struct urb *urb);
static int iap2_respin_rx_urbs(struct iap2_dev *dev, gfp_t mem_flags);

static void iap2_schedule_rx_clear_halt(struct iap2_dev *dev);
static void iap2_schedule_tx_clear_halt(struct iap2_dev *dev);

static void iap2_rx_clear_halt_workfn(struct work_struct *work)
{
    struct iap2_dev *dev = container_of(work, struct iap2_dev,
                                        rx_clear_halt_work);
    int ret;

    if (READ_ONCE(dev->disconnected))
        goto out;

    ret = usb_clear_halt(dev->udev, usb_rcvbulkpipe(dev->udev, dev->ep_in));
    if (ret) {
        spin_lock_irq(&dev->rx_lock);
        dev->rx_submit_error = ret;
        spin_unlock_irq(&dev->rx_lock);
        dev_err(&dev->intf->dev, "failed to clear RX halt: %d\n", ret);
        wake_up_interruptible(&dev->rx_wq);
        goto out;
    }

    ret = iap2_respin_rx_urbs(dev, GFP_KERNEL);
    if (ret && !READ_ONCE(dev->disconnected)) {
        dev_err(&dev->intf->dev, "RX resubmit failed after clear halt: %d\n", ret);
        wake_up_interruptible(&dev->rx_wq);
    }

out:
    clear_bit(IAP2_WORK_RX_CLEAR_HALT, &dev->work_flags);
}

static void iap2_tx_clear_halt_workfn(struct work_struct *work)
{
    struct iap2_dev *dev = container_of(work, struct iap2_dev,
                                        tx_clear_halt_work);
    int ret;

    if (READ_ONCE(dev->disconnected))
        goto out;

    ret = usb_clear_halt(dev->udev, usb_sndbulkpipe(dev->udev, dev->ep_out));
    if (ret)
        dev_err(&dev->intf->dev, "failed to clear TX halt: %d\n", ret);

    wake_up_interruptible(&dev->tx_wq);

out:
    clear_bit(IAP2_WORK_TX_CLEAR_HALT, &dev->work_flags);
}

static void iap2_schedule_rx_clear_halt(struct iap2_dev *dev)
{
    if (!test_and_set_bit(IAP2_WORK_RX_CLEAR_HALT, &dev->work_flags))
        schedule_work(&dev->rx_clear_halt_work);
}

static void iap2_schedule_tx_clear_halt(struct iap2_dev *dev)
{
    if (!test_and_set_bit(IAP2_WORK_TX_CLEAR_HALT, &dev->work_flags))
        schedule_work(&dev->tx_clear_halt_work);
}

static void iap2_dev_release(struct kref *kref)
{
    struct iap2_dev *dev = container_of(kref, struct iap2_dev, kref);
    struct iap2_rx_record *rec, *tmp;
    int i;

    list_for_each_entry_safe(rec, tmp, &dev->rx_done_list, node) {
        list_del(&rec->node);
        kfree(rec->buf);
        kfree(rec);
    }

    for (i = 0; i < IAP2_TX_INFLIGHT; i++) {
        usb_free_urb(dev->tx[i].urb);
        kfree(dev->tx[i].buf);
    }

    for (i = 0; i < IAP2_RX_INFLIGHT; i++) {
        usb_free_urb(dev->rx[i].urb);
        kfree(dev->rx[i].buf);
    }

    if (dev->minor_id >= 0)
        ida_free(&iap2_ida, dev->minor_id);

    usb_put_dev(dev->udev);
    kfree(dev);
}

static inline void iap2_get(struct iap2_dev *dev)
{
    kref_get(&dev->kref);
}

static inline void iap2_put(struct iap2_dev *dev)
{
    kref_put(&dev->kref, iap2_dev_release);
}

static int iap2_take_tx_error(struct iap2_dev *dev)
{
    unsigned long flags;
    int ret;

    spin_lock_irqsave(&dev->tx_lock, flags);
    ret = dev->tx_async_error;
    dev->tx_async_error = 0;
    spin_unlock_irqrestore(&dev->tx_lock, flags);

    return ret;
}

int iap2_char_devnode_path(struct usb_interface *intf, char *buf, size_t size)
{
    struct iap2_dev *dev = usb_get_intfdata(intf);
    int ret;

    if (!dev || READ_ONCE(dev->disconnected))
        return -ENODEV;

    ret = scnprintf(buf, size, "/dev/%s", dev->name);
    if (ret >= size)
        return -ENAMETOOLONG;

    return 0;
}
EXPORT_SYMBOL_GPL(iap2_char_devnode_path);

static bool iap2_rx_has_submitted_locked(struct iap2_dev *dev)
{
    int i;

    for (i = 0; i < IAP2_RX_INFLIGHT; i++) {
        if (dev->rx[i].submitted)
            return true;
    }

    return false;
}

static int iap2_respin_rx_urbs(struct iap2_dev *dev, gfp_t mem_flags)
{
    int i, ret = 0;
    bool any_submitted = false;
    unsigned long flags;

    for (i = 0; i < IAP2_RX_INFLIGHT; i++) {
        struct iap2_rx_slot *slot = &dev->rx[i];
        unsigned long flags;
        bool submit = false;
        int rc;

        if (READ_ONCE(dev->disconnected))
            return ret ? ret : -ENODEV;

        spin_lock_irqsave(&dev->rx_lock, flags);
        any_submitted |= slot->submitted;
        if (!slot->submitted && dev->rx_queued_bytes < IAP2_RX_BACKLOG_MAX) {
            slot->submitted = true;
            submit = true;
            any_submitted = true;
        }
        spin_unlock_irqrestore(&dev->rx_lock, flags);

        if (!submit)
            continue;

        usb_fill_bulk_urb(slot->urb,
                          dev->udev,
                          usb_rcvbulkpipe(dev->udev, dev->ep_in),
                          slot->buf,
                          IAP2_MAX_XFER,
                          iap2_rx_complete,
                          slot);

        dev_dbg(&dev->intf->dev,
                "RX submit slot=%d len=%u\n",
                slot->index, IAP2_MAX_XFER);

        rc = usb_submit_urb(slot->urb, mem_flags);
        if (rc) {
            spin_lock_irqsave(&dev->rx_lock, flags);
            slot->submitted = false;
            any_submitted = iap2_rx_has_submitted_locked(dev);
            spin_unlock_irqrestore(&dev->rx_lock, flags);

            if (rc == -EPIPE)
                iap2_schedule_rx_clear_halt(dev);

            if (!ret)
                ret = rc;
        }
    }

    spin_lock_irqsave(&dev->rx_lock, flags);
    if (ret && !any_submitted)
        dev->rx_submit_error = ret;
    else if (any_submitted)
        dev->rx_submit_error = 0;
    spin_unlock_irqrestore(&dev->rx_lock, flags);

    return ret;
}

static int iap2_find_bulk_eps(struct usb_interface *intf, struct iap2_dev *dev)
{
    struct usb_host_interface *alts = intf->cur_altsetting;
    int i;

    dev_info(&intf->dev,
             "iap2_probe: alt=%u if=%u class=%02x subclass=%02x proto=%02x eps=%u\n",
             alts->desc.bAlternateSetting,
             alts->desc.bInterfaceNumber,
             alts->desc.bInterfaceClass,
             alts->desc.bInterfaceSubClass,
             alts->desc.bInterfaceProtocol,
             alts->desc.bNumEndpoints);

    dev->ep_in = 0;
    dev->ep_out = 0;

    for (i = 0; i < alts->desc.bNumEndpoints; i++) {
        struct usb_endpoint_descriptor *ep = &alts->endpoint[i].desc;

        dev_info(&intf->dev,
                 "iap2_probe: ep[%d]=0x%02x attr=0x%02x maxp=%u dir=%s\n",
                 i, ep->bEndpointAddress, ep->bmAttributes,
                 le16_to_cpu(ep->wMaxPacketSize),
                 usb_endpoint_dir_in(ep) ? "in" : "out");

        if (usb_endpoint_is_bulk_in(ep) && !dev->ep_in)
            dev->ep_in = ep->bEndpointAddress;
        else if (usb_endpoint_is_bulk_out(ep) && !dev->ep_out)
            dev->ep_out = ep->bEndpointAddress;
    }

    if (!dev->ep_in || !dev->ep_out) {
        dev_info(&intf->dev,
                 "iap2_probe: missing bulk eps ep_in=0x%02x ep_out=0x%02x\n",
                 dev->ep_in, dev->ep_out);
        return -ENODEV;
    }

    return 0;
}

static void iap2_tx_complete(struct urb *urb)
{
    struct iap2_tx_slot *slot = urb->context;
    struct iap2_dev *dev = slot->dev;
    unsigned long flags;

    dev_dbg(&dev->intf->dev,
            "TX complete slot=%d status=%d actual=%u\n",
            slot->index, urb->status, urb->actual_length);

    slot->status = urb->status;

    if (urb->status == -EPIPE)
        iap2_schedule_tx_clear_halt(dev);

    spin_lock_irqsave(&dev->tx_lock, flags);
    if (urb->status && !READ_ONCE(dev->disconnected))
        dev->tx_async_error = urb->status;
    slot->in_use = false;
    spin_unlock_irqrestore(&dev->tx_lock, flags);

    complete(&slot->done);
    wake_up_interruptible(&dev->tx_wq);
}

static void iap2_rx_complete(struct urb *urb)
{
    struct iap2_rx_slot *slot = urb->context;
    struct iap2_dev *dev = slot->dev;
    size_t len = urb->actual_length;
    unsigned long flags;
    bool reserved = false;
    int ret;

    dev_dbg(&dev->intf->dev,
            "RX complete slot=%d status=%d actual=%u\n",
            slot->index, urb->status, urb->actual_length);

    if (likely(!READ_ONCE(dev->disconnected))) {
        spin_lock_irqsave(&dev->rx_lock, flags);
        slot->submitted = false;
        if (urb->status == 0 && len > 0 &&
            dev->rx_queued_bytes + len <= IAP2_RX_BACKLOG_MAX) {
            dev->rx_queued_bytes += len;
            reserved = true;
        }
        spin_unlock_irqrestore(&dev->rx_lock, flags);

        if (urb->status == 0 && len > 0 && reserved) {
            struct iap2_rx_record *rec;

            rec = kzalloc(sizeof(*rec), GFP_ATOMIC);
            if (rec) {
                rec->buf = kmemdup(slot->buf, len, GFP_ATOMIC);
                if (rec->buf) {
                    rec->len = len;

                    spin_lock_irqsave(&dev->rx_lock, flags);
                    list_add_tail(&rec->node, &dev->rx_done_list);
                    spin_unlock_irqrestore(&dev->rx_lock, flags);

                    wake_up_interruptible(&dev->rx_wq);
                } else {
                    spin_lock_irqsave(&dev->rx_lock, flags);
                    dev->rx_queued_bytes -= len;
                    spin_unlock_irqrestore(&dev->rx_lock, flags);
                    kfree(rec);
                }
            } else {
                spin_lock_irqsave(&dev->rx_lock, flags);
                dev->rx_queued_bytes -= len;
                spin_unlock_irqrestore(&dev->rx_lock, flags);
            }
        } else if (urb->status == 0 && len > 0) {
            dev_warn_ratelimited(&dev->intf->dev,
                                 "RX backlog limit reached, throttling\n");
        }

        if (urb->status == -EPIPE)
            iap2_schedule_rx_clear_halt(dev);

        ret = iap2_respin_rx_urbs(dev, GFP_ATOMIC);
        if (ret) {
            dev_err(&dev->intf->dev, "RX resubmit failed: %d\n", ret);
            wake_up_interruptible(&dev->rx_wq);
        }
    } else {
        spin_lock_irqsave(&dev->rx_lock, flags);
        slot->submitted = false;
        spin_unlock_irqrestore(&dev->rx_lock, flags);
        wake_up_interruptible(&dev->rx_wq);
    }
}

static int iap2_start_rx(struct iap2_dev *dev)
{
    int i;

    for (i = 0; i < IAP2_RX_INFLIGHT; i++) {
        dev->rx[i].dev = dev;
        dev->rx[i].index = i;
        dev->rx[i].buf = kmalloc(IAP2_MAX_XFER, GFP_KERNEL);
        if (!dev->rx[i].buf)
            return -ENOMEM;

        dev->rx[i].urb = usb_alloc_urb(0, GFP_KERNEL);
        if (!dev->rx[i].urb)
            return -ENOMEM;
    }

    return iap2_respin_rx_urbs(dev, GFP_KERNEL);
}

static void iap2_kill_all_urbs(struct iap2_dev *dev)
{
    int i;

    for (i = 0; i < IAP2_TX_INFLIGHT; i++) {
        if (dev->tx[i].urb)
            usb_kill_urb(dev->tx[i].urb);
    }

    for (i = 0; i < IAP2_RX_INFLIGHT; i++) {
        if (dev->rx[i].urb)
            usb_kill_urb(dev->rx[i].urb);
    }
}

static void iap2_stop_io(struct iap2_dev *dev)
{
    WRITE_ONCE(dev->disconnected, true);

    iap2_kill_all_urbs(dev);
    cancel_work_sync(&dev->rx_clear_halt_work);
    cancel_work_sync(&dev->tx_clear_halt_work);
}

static int iap2_open(struct inode *inode, struct file *file)
{
    struct miscdevice *mdev = file->private_data;
    struct iap2_dev *dev = container_of(mdev, struct iap2_dev, miscdev);

    if (READ_ONCE(dev->disconnected))
        return -ENODEV;

    iap2_get(dev);
    atomic_inc(&dev->open_count);
    file->private_data = dev;
    return 0;
}

static int iap2_release_file(struct inode *inode, struct file *file)
{
    struct iap2_dev *dev = file->private_data;

    if (dev) {
        atomic_dec(&dev->open_count);
        iap2_put(dev);
    }

    return 0;
}

static __poll_t iap2_poll(struct file *file, poll_table *wait)
{
    struct iap2_dev *dev = file->private_data;
    __poll_t mask = 0;
    unsigned long flags;
    bool tx_free = false;
    int tx_error = 0;
    bool rx_ready = false;
    int rx_error = 0;
    int i;

    poll_wait(file, &dev->tx_wq, wait);
    poll_wait(file, &dev->rx_wq, wait);

    if (READ_ONCE(dev->disconnected))
        return EPOLLERR | EPOLLHUP;

    spin_lock_irqsave(&dev->tx_lock, flags);
    for (i = 0; i < IAP2_TX_INFLIGHT; i++) {
        if (!dev->tx[i].in_use) {
            tx_free = true;
            break;
        }
    }
    tx_error = dev->tx_async_error;
    spin_unlock_irqrestore(&dev->tx_lock, flags);

    spin_lock_irqsave(&dev->rx_lock, flags);
    rx_ready = !list_empty(&dev->rx_done_list);
    rx_error = dev->rx_submit_error;
    spin_unlock_irqrestore(&dev->rx_lock, flags);

    if (tx_free)
        mask |= EPOLLOUT | EPOLLWRNORM;
    if (tx_error)
        mask |= EPOLLERR;
    if (rx_ready)
        mask |= EPOLLIN | EPOLLRDNORM;
    if (rx_error)
        mask |= EPOLLERR;

    dev_dbg(&dev->intf->dev,
            "poll mask=0x%x tx_free=%d tx_error=%d rx_ready=%d rx_error=%d\n",
            mask, tx_free, tx_error, rx_ready, rx_error);

    return mask;
}

static ssize_t iap2_write(struct file *file, const char __user *buf,
                          size_t count, loff_t *ppos)
{
    struct iap2_dev *dev = file->private_data;
    struct iap2_tx_slot *slot = NULL;
    unsigned long flags;
    int i, ret;
    ssize_t rc;

    if (count == 0)
        return 0;
    if (count > IAP2_MAX_XFER)
        return -EMSGSIZE;
    ret = iap2_take_tx_error(dev);
    if (ret)
        return ret;
    if (READ_ONCE(dev->disconnected))
        return -ENODEV;

    for (;;) {
        ret = iap2_take_tx_error(dev);
        if (ret)
            return ret;
        if (READ_ONCE(dev->disconnected))
            return -ENODEV;

        spin_lock_irqsave(&dev->tx_lock, flags);
        for (i = 0; i < IAP2_TX_INFLIGHT; i++) {
            if (!dev->tx[i].in_use) {
                slot = &dev->tx[i];
                slot->in_use = true;
                reinit_completion(&slot->done);
                slot->status = -EINPROGRESS;
                break;
            }
        }
        spin_unlock_irqrestore(&dev->tx_lock, flags);

        if (slot)
            break;

        if (file->f_flags & O_NONBLOCK)
            return -EAGAIN;

        ret = wait_event_interruptible(dev->tx_wq,
                                       READ_ONCE(dev->disconnected) ||
                                       ({
                                           bool free_found = false;
                                           unsigned long f2;
                                           int j;
                                           spin_lock_irqsave(&dev->tx_lock, f2);
                                           for (j = 0; j < IAP2_TX_INFLIGHT; j++) {
                                               if (!dev->tx[j].in_use) {
                                                   free_found = true;
                                                   break;
                                               }
                                           }
                                           spin_unlock_irqrestore(&dev->tx_lock, f2);
                                           free_found;
                                       }));
        if (ret)
            return ret;
    }

    if (copy_from_user(slot->buf, buf, count)) {
        spin_lock_irqsave(&dev->tx_lock, flags);
        slot->in_use = false;
        spin_unlock_irqrestore(&dev->tx_lock, flags);
        wake_up_interruptible(&dev->tx_wq);
        return -EFAULT;
    }

    usb_fill_bulk_urb(slot->urb,
                      dev->udev,
                      usb_sndbulkpipe(dev->udev, dev->ep_out),
                      slot->buf,
                      count,
                      iap2_tx_complete,
                      slot);

    dev_dbg(&dev->intf->dev,
            "TX submit slot=%d len=%zu\n",
            slot->index, count);

    /*
     * Intentionally no URB_ZERO_PACKET.
     * For 65535 we want "non-multiple-of-maxpacket" behavior.
     */
    slot->urb->transfer_flags = 0;

    ret = usb_submit_urb(slot->urb, GFP_KERNEL);
    if (ret) {
        spin_lock_irqsave(&dev->tx_lock, flags);
        slot->in_use = false;
        spin_unlock_irqrestore(&dev->tx_lock, flags);
        wake_up_interruptible(&dev->tx_wq);

        if (ret == -EPIPE)
            iap2_schedule_tx_clear_halt(dev);

        if (READ_ONCE(dev->disconnected))
            return -ENODEV;
        return ret;
    }

    if (file->f_flags & O_NONBLOCK)
        return count;

    ret = wait_for_completion_interruptible(&slot->done);
    if (ret) {
        /*
         * Preserve 1 write syscall <-> 1 URB completion.
         * We do not return early with partial success; we wait for actual
         * URB completion unless interrupted. If interrupted, caller gets EINTR.
         */
        usb_kill_urb(slot->urb);
        return ret;
    }

    if (READ_ONCE(dev->disconnected))
        return -ENODEV;

    if (slot->status)
        return slot->status;

    rc = count;
    return rc;
}

static ssize_t iap2_read(struct file *file, char __user *buf,
                         size_t count, loff_t *ppos)
{
    struct iap2_dev *dev = file->private_data;
    struct iap2_rx_record *rec;
    size_t copy_len;
    unsigned long flags;
    int ret;

    if (READ_ONCE(dev->disconnected))
        return -ENODEV;

retry:
    spin_lock_irqsave(&dev->rx_lock, flags);
    if (list_empty(&dev->rx_done_list)) {
        ret = dev->rx_submit_error;
        spin_unlock_irqrestore(&dev->rx_lock, flags);

        if (ret)
            return ret;

        if (file->f_flags & O_NONBLOCK)
            return -EAGAIN;

        ret = wait_event_interruptible(dev->rx_wq,
                                       READ_ONCE(dev->disconnected) ||
                                       READ_ONCE(dev->rx_submit_error) ||
                                       ({
                                           bool ready;
                                           unsigned long f2;
                                           spin_lock_irqsave(&dev->rx_lock, f2);
                                           ready = !list_empty(&dev->rx_done_list);
                                           spin_unlock_irqrestore(&dev->rx_lock, f2);
                                           ready;
                                       }));
        if (ret)
            return ret;
        if (READ_ONCE(dev->disconnected))
            return -ENODEV;
        if (READ_ONCE(dev->rx_submit_error))
            return READ_ONCE(dev->rx_submit_error);
        goto retry;
    }

    rec = list_first_entry(&dev->rx_done_list, struct iap2_rx_record, node);
    copy_len = min(count, rec->len - rec->off);
    if (!copy_len) {
        spin_unlock_irqrestore(&dev->rx_lock, flags);
        return 0;
    }
    spin_unlock_irqrestore(&dev->rx_lock, flags);

    if (copy_to_user(buf, rec->buf + rec->off, copy_len))
        return -EFAULT;

    spin_lock_irqsave(&dev->rx_lock, flags);
    rec->off += copy_len;
    if (rec->off == rec->len) {
        list_del(&rec->node);
        dev->rx_queued_bytes -= rec->len;
        spin_unlock_irqrestore(&dev->rx_lock, flags);

        ret = iap2_respin_rx_urbs(dev, GFP_KERNEL);
        if (ret && !READ_ONCE(dev->disconnected))
            dev_err(&dev->intf->dev, "RX resubmit failed: %d\n", ret);

        kfree(rec->buf);
        kfree(rec);
    } else {
        spin_unlock_irqrestore(&dev->rx_lock, flags);
    }

    return copy_len;
}

static const struct file_operations iap2_fops = {
    .owner = THIS_MODULE,
    .open = iap2_open,
    .release = iap2_release_file,
    .read = iap2_read,
    .write = iap2_write,
    .poll = iap2_poll,
    .llseek = noop_llseek,
};

static int iap2_probe(struct usb_interface *intf, const struct usb_device_id *id)
{
    struct iap2_dev *dev;
    int ret, i;

    dev_info(&intf->dev, "iap2_probe: enter\n");

    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    dev->udev = usb_get_dev(interface_to_usbdev(intf));
    dev->intf = intf;
    dev->minor_id = -1;
    kref_init(&dev->kref);

    spin_lock_init(&dev->tx_lock);
    spin_lock_init(&dev->rx_lock);
    init_waitqueue_head(&dev->tx_wq);
    init_waitqueue_head(&dev->rx_wq);
    INIT_LIST_HEAD(&dev->rx_done_list);
    atomic_set(&dev->open_count, 0);
    INIT_WORK(&dev->rx_clear_halt_work, iap2_rx_clear_halt_workfn);
    INIT_WORK(&dev->tx_clear_halt_work, iap2_tx_clear_halt_workfn);

    ret = iap2_find_bulk_eps(intf, dev);
    if (ret) {
        dev_err(&intf->dev, "bulk endpoints not found\n");
        goto err_put;
    }

    for (i = 0; i < IAP2_TX_INFLIGHT; i++) {
        dev->tx[i].dev = dev;
        dev->tx[i].index = i;
        init_completion(&dev->tx[i].done);

        dev->tx[i].buf = kmalloc(IAP2_MAX_XFER, GFP_KERNEL);
        if (!dev->tx[i].buf) {
            ret = -ENOMEM;
            goto err_put;
        }

        dev->tx[i].urb = usb_alloc_urb(0, GFP_KERNEL);
        if (!dev->tx[i].urb) {
            ret = -ENOMEM;
            goto err_put;
        }
    }

    dev->minor_id = ida_alloc(&iap2_ida, GFP_KERNEL);
    if (dev->minor_id < 0) {
        ret = dev->minor_id;
        dev->minor_id = -1;
        goto err_put;
    }

    snprintf(dev->name, sizeof(dev->name), "iap2-%d", dev->minor_id);

    dev->miscdev.minor = MISC_DYNAMIC_MINOR;
    dev->miscdev.name = dev->name;
    dev->miscdev.fops = &iap2_fops;
    dev->miscdev.parent = &intf->dev;
    dev->miscdev.mode = 0600;

    ret = misc_register(&dev->miscdev);
    if (ret) {
        dev_err(&intf->dev, "misc_register failed: %d\n", ret);
        goto err_put;
    }

    ret = sysfs_create_link(&intf->dev.kobj, &dev->miscdev.this_device->kobj,
                            "iap2_devnode");
    if (ret) {
        dev_err(&intf->dev, "sysfs_create_link failed: %d\n", ret);
        misc_deregister(&dev->miscdev);
        goto err_put;
    }
    dev->sysfs_link_added = true;

    usb_set_intfdata(intf, dev);

    ret = iap2_start_rx(dev);
    if (ret) {
        dev_err(&intf->dev, "failed to start RX ring: %d\n", ret);
        usb_set_intfdata(intf, NULL);
        if (dev->sysfs_link_added) {
            sysfs_remove_link(&intf->dev.kobj, "iap2_devnode");
            dev->sysfs_link_added = false;
        }
        misc_deregister(&dev->miscdev);
        goto err_put;
    }

    dev_info(&intf->dev,
             "bound iAP2 interface, device node /dev/%s (ep_in=0x%02x ep_out=0x%02x)\n",
             dev->name, dev->ep_in, dev->ep_out);

    return 0;

err_put:
    iap2_stop_io(dev);
    iap2_put(dev);
    return ret;
}

static void iap2_disconnect(struct usb_interface *intf)
{
    struct iap2_dev *dev = usb_get_intfdata(intf);

    if (!dev)
        return;

    usb_set_intfdata(intf, NULL);

    if (dev->sysfs_link_added) {
        sysfs_remove_link(&intf->dev.kobj, "iap2_devnode");
        dev->sysfs_link_added = false;
    }

    misc_deregister(&dev->miscdev);
    iap2_stop_io(dev);

    wake_up_interruptible_all(&dev->tx_wq);
    wake_up_interruptible_all(&dev->rx_wq);

    dev_info(&intf->dev, "disconnected\n");
    iap2_put(dev);
}

static const struct usb_device_id iap2_id_table[] = {
    { USB_INTERFACE_INFO(0xFF, 0xF0, 0x00) },
    { }
};
MODULE_DEVICE_TABLE(usb, iap2_id_table);

static struct usb_driver iap2_usb_driver = {
    .name = DRV_NAME,
    .probe = iap2_probe,
    .disconnect = iap2_disconnect,
    .id_table = iap2_id_table,
};

module_usb_driver(iap2_usb_driver);

MODULE_AUTHOR("CatPlay");
MODULE_DESCRIPTION("Simple iAP2 USB char driver");
MODULE_LICENSE("GPL");
