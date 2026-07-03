use async_trait::async_trait;

pub trait AsyncShutdown: Send {
    fn shutdown(&mut self) -> impl Send + Future<Output = ()> {
        core::future::ready(())
    }
}

#[async_trait]
pub trait AsyncShutdownDyn: Send {
    async fn shutdown_pinned(&mut self);
}

#[async_trait]
impl<T: AsyncShutdown> AsyncShutdownDyn for T {
    async fn shutdown_pinned(&mut self) {
        self.shutdown().await
    }
}

impl<T: AsyncShutdown> AsyncShutdown for Option<T> {
    async fn shutdown(&mut self) {
        match self {
            None => {}
            Some(v) => v.shutdown().await,
        }
    }
}

impl<T: AsyncShutdown + ?Sized> AsyncShutdown for &mut T {
    async fn shutdown(&mut self) {
        T::shutdown(*self).await
    }
}

impl<T: AsyncShutdownDyn + ?Sized> AsyncShutdown for Box<T> {
    async fn shutdown(&mut self) {
        self.as_mut().shutdown_pinned().await
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {

    use crate::{AsyncShutdown, AsyncShutdownDyn};

    struct TestAsyncShutdown {}

    impl AsyncShutdown for TestAsyncShutdown {
        async fn shutdown(&mut self) {}
    }

    struct TestAsyncShutdownDyn {}

    impl AsyncShutdown for TestAsyncShutdownDyn {
        async fn shutdown(&mut self) {}
    }

    #[tokio::test]
    async fn test_compile() {
        let mut a = TestAsyncShutdown {};
        a.shutdown().await;
        let b = a.shutdown_pinned();
        b.await;

        let mut c = TestAsyncShutdownDyn {};
        let d = &mut c as &mut dyn AsyncShutdownDyn;
        d.shutdown_pinned().await;
    }
}
