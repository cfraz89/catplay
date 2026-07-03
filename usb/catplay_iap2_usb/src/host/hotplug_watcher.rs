use catplay_util::{EventReconciler, EventSleeper, EventToken};
use log::debug;
use rusb::{Context, Device, Hotplug, HotplugBuilder, Registration, UsbContext};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};
use tokio::sync::Notify;

#[derive(Default)]
struct HotplugState {
    pending: AtomicBool,
    notify: Notify,
    error: Mutex<Option<rusb::Error>>,
}

struct HotplugCallback {
    state: Arc<HotplugState>,
}

impl<T: UsbContext> Hotplug<T> for HotplugCallback {
    fn device_arrived(&mut self, device: Device<T>) {
        debug!("rusb hotplug arrived bus={} addr={}", device.bus_number(), device.address());
        self.state.pending.store(true, Ordering::Release);
        self.state.notify.notify_waiters();
    }

    fn device_left(&mut self, device: Device<T>) {
        debug!("rusb hotplug left bus={} addr={}", device.bus_number(), device.address());
        self.state.pending.store(true, Ordering::Release);
        self.state.notify.notify_waiters();
    }
}

pub struct RusbHotplugWatcher {
    state: Arc<HotplugState>,
    context: Context,
    registration: Option<Registration<Context>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl RusbHotplugWatcher {
    pub fn new() -> rusb::Result<Self> {
        if !rusb::has_hotplug() {
            return Err(rusb::Error::NotSupported);
        }

        let context = Context::new()?;
        let state = Arc::new(HotplugState::default());
        let stop = Arc::new(AtomicBool::new(false));

        let registration = Some(
            HotplugBuilder::new()
                .enumerate(true)
                .register(&context, Box::new(HotplugCallback { state: state.clone() }))?,
        );

        let worker_context = context.clone();
        let worker_state = state.clone();
        let worker_stop = stop.clone();
        let worker = thread::Builder::new()
            .name("rusb-hotplug".into())
            .spawn(move || {
                while !worker_stop.load(Ordering::Acquire) {
                    match worker_context.handle_events(None) {
                        Ok(()) => {}
                        Err(rusb::Error::Interrupted) if worker_stop.load(Ordering::Acquire) => break,
                        Err(err) => {
                            debug!("rusb hotplug worker failed: {err}");
                            *worker_state.error.lock().unwrap() = Some(err);
                            worker_state.notify.notify_waiters();
                            break;
                        }
                    }
                }
            })
            .expect("failed to spawn rusb hotplug worker");

        Ok(Self {
            state,
            context,
            registration,
            stop,
            worker: Some(worker),
        })
    }

    pub fn take_pending_hotplug(&mut self) -> bool {
        self.state.pending.swap(false, Ordering::AcqRel)
    }
}

impl EventSleeper for RusbHotplugWatcher {
    async fn sleep(&mut self) -> Option<EventToken> {
        if self.state.error.lock().unwrap().is_some() {
            return None;
        }

        let notified = self.state.notify.notified();
        tokio::pin!(notified);

        if self.state.pending.load(Ordering::Acquire) {
            return Some(EventToken(1));
        }

        notified.await;
        Some(EventToken(1))
    }
}

impl EventReconciler for RusbHotplugWatcher {
    type Error = rusb::Error;

    async fn reconcile(&mut self) -> rusb::Result<()> {
        if let Some(err) = *self.state.error.lock().unwrap() {
            return Err(err);
        }

        Ok(())
    }
}

impl Drop for RusbHotplugWatcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.context.interrupt_handle_events();

        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }

        self.registration.take();
    }
}
