use libc::{iovec, mmsghdr, msghdr, sendmmsg, sockaddr_storage};
use std::{io, mem::MaybeUninit, net::SocketAddr, os::fd::AsRawFd};

use super::{sockaddr_storage_to_std, socketaddr_to_storage};

const IOV_MAX: usize = 128;

/// Implements missing recvmmsg/sendmmsg functions from standard library to reduce overhead of RTP streaming.
pub trait UdpSender {
    /// Writes as many datagrams as possible into the socket and
    /// returns the number of datagrams that were successfuly sent.
    ///
    /// Datagrams past that success index may be retried again in another call if desired.
    ///
    /// If peer is `None`, it is assumed that `connect()` was called and the system will use that address instead.
    fn sendmmsg(&self, bufs: &[&[u8]], peer: Option<SocketAddr>) -> io::Result<usize>;

    /// Reads as many messages as possible into user buffers, runs a callback on each(sliced to the real size of data), and returns `true` if
    /// there is more data to read or `false` if everything was read.
    fn recvmmsg<F>(&self, bufs: &mut [&mut [u8]], on_data: &mut F) -> io::Result<bool>
    where
        F: FnMut(&mut [u8], Option<SocketAddr>);
}

fn sendmmsg_parts(fd: &dyn AsRawFd, bufs: &[&[u8]], peer: Option<SocketAddr>) -> io::Result<usize> {
    let iov_size = bufs.len().min(IOV_MAX);

    let mut iovecs: [_; IOV_MAX] = [MaybeUninit::<iovec>::uninit(); IOV_MAX];
    let mut hdrs: [_; IOV_MAX] = [MaybeUninit::<mmsghdr>::uninit(); IOV_MAX];
    let mut names: [_; IOV_MAX] = [MaybeUninit::<sockaddr_storage>::uninit(); IOV_MAX];

    let (peer_storage, peer_len) = if let Some(addr) = peer {
        let (storage, len) = socketaddr_to_storage(&addr);
        (Some(storage), len)
    } else {
        (None, 0)
    };

    for i in 0..iov_size {
        let buf = bufs[i];

        iovecs[i].write(iovec {
            iov_base: buf.as_ptr() as *mut _,
            iov_len: buf.len(),
        });

        let (msg_name, msg_namelen) = if let Some(storage) = peer_storage {
            names[i].write(storage);
            (names[i].as_mut_ptr() as *mut _, peer_len)
        } else {
            (std::ptr::null_mut(), 0)
        };

        hdrs[i].write(mmsghdr {
            msg_hdr: msghdr {
                msg_name,
                msg_namelen,
                msg_iov: iovecs[i].as_mut_ptr(),
                msg_iovlen: 1,
                msg_control: std::ptr::null_mut(),
                msg_controllen: 0,
                msg_flags: 0,
            },
            msg_len: 0,
        });
    }

    let ret = unsafe {
        sendmmsg(
            fd.as_raw_fd(),
            hdrs.as_mut_ptr() as *mut mmsghdr,
            iov_size as u32,
            libc::MSG_DONTWAIT as _,
        )
    };

    if ret < 0 {
        let err: io::Error = std::io::Error::last_os_error();
        if let Some(libc::EWOULDBLOCK) = err.raw_os_error() {
            return Ok(0); // nothing was written
        } else {
            return Err(err);
        }
    }

    Ok(ret as usize)
}

impl<S: AsRawFd> UdpSender for S {
    fn sendmmsg(&self, bufs: &[&[u8]], peer: Option<SocketAddr>) -> io::Result<usize> {
        if bufs.is_empty() {
            return Ok(0);
        }

        let mut total_sent = 0;

        for chunk in bufs.chunks(IOV_MAX) {
            let sent = sendmmsg_parts(self, chunk, peer)?;
            total_sent += sent;

            // if we sent less datagrams than the amount in chunk, the socket cannot accept any more datagrams at this time so stop
            if sent < chunk.len() {
                break;
            }
        }

        Ok(total_sent)
    }

    fn recvmmsg<F>(&self, bufs: &mut [&mut [u8]], on_data: &mut F) -> io::Result<bool>
    where
        F: FnMut(&mut [u8], Option<SocketAddr>),
    {
        let mut iovecs: [_; IOV_MAX] = [MaybeUninit::<iovec>::uninit(); IOV_MAX];
        let mut addrs = [MaybeUninit::<sockaddr_storage>::uninit(); IOV_MAX];
        let mut hdrs = [MaybeUninit::<mmsghdr>::uninit(); IOV_MAX];

        let iov_size = bufs.len().min(IOV_MAX);

        for i in 0..iov_size {
            let buf = unsafe { &mut *bufs.as_mut_ptr().add(i) };

            iovecs[i].write(iovec {
                iov_base: buf.as_mut_ptr() as *mut _,
                iov_len: buf.len(),
            });

            let hdr = mmsghdr {
                msg_hdr: libc::msghdr {
                    msg_name: addrs[i].as_mut_ptr() as *mut _,
                    msg_namelen: std::mem::size_of::<sockaddr_storage>() as _,
                    msg_iov: iovecs[i].as_mut_ptr(),
                    msg_iovlen: 1,
                    msg_control: std::ptr::null_mut(),
                    msg_controllen: 0,
                    msg_flags: 0,
                },
                msg_len: 0,
            };
            hdrs[i].write(hdr);
        }

        let ret = unsafe {
            libc::recvmmsg(
                self.as_raw_fd(),
                hdrs.as_mut_ptr() as *mut mmsghdr,
                iov_size as u32,
                libc::MSG_DONTWAIT as _,
                std::ptr::null_mut(),
            )
        };

        if ret == 0 {
            return Ok(false); // no more data
        }

        if ret < 0 {
            let err: io::Error = std::io::Error::last_os_error();
            if let Some(libc::EWOULDBLOCK) = err.raw_os_error() {
                return Ok(false); // no more data
            } else {
                return Err(err);
            }
        }

        let received = ret as usize;
        for i in 0..received {
            let len = unsafe { hdrs[i].assume_init_ref().msg_len } as usize;
            let peer = unsafe { addrs[i].assume_init_ref() };
            let peer = sockaddr_storage_to_std(peer);

            let data = &mut bufs[i][..len];
            on_data(data, peer);
        }

        Ok(true) // more data
    }
}
