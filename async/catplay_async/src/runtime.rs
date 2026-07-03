use std::{
    pin::Pin,
    task::{Context, Poll},
};

// API change vs Tokio: we default to AbortOnDropHandle with option to detach on demand.

pub struct AbortOnDropHandle<T>(tokio_util::task::AbortOnDropHandle<T>);
pub struct JoinHandle<T>(tokio::task::JoinHandle<T>);
#[derive(Clone, Debug)]
pub struct Handle(tokio::runtime::Handle);
#[derive(Clone, Debug)]
pub struct AbortHandle(tokio::task::AbortHandle);

impl<T> AbortOnDropHandle<T> {
    /// Abort the task associated with this handle,
    /// equivalent to [`JoinHandle::abort`].
    pub fn abort(&self) {
        self.0.abort()
    }

    /// Checks if the task associated with this handle is finished,
    /// equivalent to [`JoinHandle::is_finished`].
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }

    /// Returns a new [`AbortHandle`] that can be used to remotely abort this task,
    /// equivalent to [`JoinHandle::abort_handle`].
    pub fn abort_handle(&self) -> AbortHandle {
        AbortHandle(self.0.abort_handle())
    }

    /// Cancels aborting on drop and returns the original [`JoinHandle`].
    pub fn detach(self) -> JoinHandle<T> {
        JoinHandle(self.0.detach())
    }
}

impl<T> Future for AbortOnDropHandle<T> {
    type Output = Result<T, tokio::task::JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

impl<T> JoinHandle<T> {
    /// Abort the task associated with this handle.
    pub fn abort(&self) {
        self.0.abort()
    }

    /// Checks if the task associated with this handle is finished.
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }

    /// Returns a new [`AbortHandle`] that can be used to remotely abort this task.
    pub fn abort_handle(&self) -> AbortHandle {
        AbortHandle(self.0.abort_handle())
    }

    /// Returns the unique identifier of the task associated with this handle.
    pub fn id(&self) -> tokio::task::Id {
        self.0.id()
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, tokio::task::JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

impl AbortHandle {
    /// Abort the task associated with this handle.
    pub fn abort(&self) {
        self.0.abort()
    }

    /// Checks if the task associated with this handle is finished.
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }

    /// Returns the unique identifier of the task associated with this handle.
    pub fn id(&self) -> tokio::task::Id {
        self.0.id()
    }
}

impl Handle {
    /// Spawns tasks on this event loop.
    #[must_use = "aborts on drop"]
    pub fn spawn<F>(&self, future: F) -> AbortOnDropHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        AbortOnDropHandle(tokio_util::task::AbortOnDropHandle::new(self.0.spawn(future)))
    }
}

/// Spawns tasks inside current event loop, panics if not inside event loop context.
#[must_use = "aborts on drop"]
pub fn spawn<F>(future: F) -> AbortOnDropHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    AbortOnDropHandle(tokio_util::task::AbortOnDropHandle::new(tokio::spawn(future)))
}

#[must_use = "aborts on drop"]
pub fn spawn_blocking<F, R>(f: F) -> AbortOnDropHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    AbortOnDropHandle(tokio_util::task::AbortOnDropHandle::new(tokio::task::spawn_blocking(f)))
}

pub mod handle {
    use super::Handle;

    /// Returns a handle to the current event loop.
    pub fn current() -> Handle {
        Handle(tokio::runtime::Handle::current())
    }
}

// Spawn task in a dedicated event loop backed by a dedicated thread.
// pub fn spawn_rt<F>(future: F) -> AbortOnDropHandle<F::Output>
// where
//     // Possibly remove Send requirement later
//     F: Future + Send + 'static,
//     F::Output: Send + 'static,
// {
//     let thread = thread::spawn(move || {
//         let builder = tokio::runtime::Builder::new_current_thread();
//         builder.build();

//         builder.enable_all().thread_name(val);
//     });
// }
