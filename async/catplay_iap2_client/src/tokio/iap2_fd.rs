#![allow(unused)]

use std::{
    fs::OpenOptions,
    io,
    os::fd::{AsRawFd, RawFd},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::unix::AsyncFd,
    io::{AsyncRead, AsyncWrite, ReadBuf},
};

fn read_nonblock(fd: RawFd, buf: &mut [u8]) -> io::Result<usize> {
    let ret = unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) };
    if ret >= 0 {
        return Ok(ret as usize);
    }

    let err = io::Error::last_os_error();
    match err.kind() {
        io::ErrorKind::WouldBlock => Err(io::Error::from(io::ErrorKind::WouldBlock)),
        _ => Err(err),
    }
}

fn write_nonblock(fd: RawFd, buf: &[u8]) -> io::Result<usize> {
    let ret = unsafe { libc::write(fd, buf.as_ptr().cast(), buf.len()) };
    if ret >= 0 {
        return Ok(ret as usize);
    }

    let err = io::Error::last_os_error();
    match err.kind() {
        io::ErrorKind::WouldBlock => Err(io::Error::from(io::ErrorKind::WouldBlock)),
        _ => Err(err),
    }
}

/// Bridge for /dev/iap2-0.
pub struct IAP2Fd {
    inner: AsyncFd<std::fs::File>,
}

impl IAP2Fd {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(path)?;

        Ok(Self {
            inner: AsyncFd::new(file)?,
        })
    }

    fn raw_fd(&self) -> RawFd {
        self.inner.get_ref().as_raw_fd()
    }
}

impl AsyncRead for IAP2Fd {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        let fd = self.as_ref().get_ref().raw_fd();
        let this = self.get_mut();

        loop {
            let mut guard = match this.inner.poll_read_ready_mut(cx) {
                Poll::Ready(Ok(guard)) => guard,
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                Poll::Pending => return Poll::Pending,
            };

            let unfilled = buf.initialize_unfilled();
            match guard.try_io(|_| read_nonblock(fd, unfilled)) {
                Ok(Ok(0)) => return Poll::Ready(Ok(())),
                Ok(Ok(n)) => {
                    buf.advance(n);
                    return Poll::Ready(Ok(()));
                }
                Ok(Err(err)) => return Poll::Ready(Err(err)),
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncWrite for IAP2Fd {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        let fd = self.as_ref().get_ref().raw_fd();
        let this = self.get_mut();

        loop {
            let mut guard = match this.inner.poll_write_ready_mut(cx) {
                Poll::Ready(Ok(guard)) => guard,
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                Poll::Pending => return Poll::Pending,
            };

            match guard.try_io(|_| write_nonblock(fd, buf)) {
                Ok(Ok(n)) => return Poll::Ready(Ok(n)),
                Ok(Err(err)) => return Poll::Ready(Err(err)),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
