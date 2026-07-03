use async_trait::async_trait;

#[async_trait]
pub trait EventRefSink<T, R>: Send + 'static {
    async fn on_event<'a>(&mut self, event: &'a T) -> R;
}

#[async_trait]
pub trait EventMutSink<T, R>: Send + 'static {
    async fn on_event<'a>(&mut self, event: &'a mut T) -> R;
}

#[async_trait]
pub trait EventSink<T, R>: Send + 'static {
    async fn on_event<'a>(&mut self, event: T) -> R
    where
        T: 'a;
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use crate::{EventMutSink, EventRefSink, EventSink};

    pub struct TestEvent<'a> {
        data: &'a u8,
    }

    pub struct TestImpl {}

    #[async_trait]
    impl EventSink<TestEvent<'_>, ()> for TestImpl {
        async fn on_event<'a>(&mut self, event: TestEvent<'a>) {}
    }

    #[async_trait]
    impl EventRefSink<TestEvent<'_>, ()> for TestImpl {
        async fn on_event<'a>(&mut self, event: &'a TestEvent) {}
    }

    #[async_trait]
    impl EventMutSink<TestEvent<'_>, ()> for TestImpl {
        async fn on_event<'a>(&mut self, event: &'a mut TestEvent) {}
    }
}
