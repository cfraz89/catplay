use std::time::Instant;

use crate::{AsyncShutdown, EventReconciler, EventSleeper};

pub trait Reconcilable: Send + AsyncShutdown {
    type Output: Clone + PartialEq + Send + Sync;

    fn on_update(&mut self, new: Self::Output) -> impl Future<Output = Self::Output> + Send;

    fn render(&mut self, prev: Self::Output, update: Instant) -> impl Future<Output = Self::Output> + Send;
}

pub struct Reconciler<T: Reconcilable> {
    child: T,
    state: T::Output,
    update: Instant,
}

impl<T: Reconcilable> Reconciler<T> {
    pub fn new(child: T, state: T::Output) -> Self {
        Self {
            child,
            state,
            update: Instant::now(),
        }
    }

    pub fn state(&self) -> &T::Output {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut T::Output {
        &mut self.state
    }

    pub fn child(&self) -> &T {
        &self.child
    }

    pub fn child_mut(&mut self) -> &mut T {
        &mut self.child
    }
}

impl<T: Reconcilable + EventSleeper> EventSleeper for Reconciler<T> {
    async fn sleep(&mut self) -> Option<crate::EventToken> {
        self.child.sleep().await
    }
}

impl<T: Reconcilable> EventReconciler for Reconciler<T> {
    type Error = ();

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        let child = &mut self.child;
        let mut dirty = true;

        while dirty {
            let update = self.update;
            let prev = { self.state.clone() };
            let mut next = { child.render(prev.clone(), update).await };

            if next != prev {
                next = child.on_update(next).await;
                self.state = next;
                self.update = Instant::now();
            } else {
                dirty = false;
            }
        }

        Ok(())
    }
}

impl<T: Reconcilable> AsyncShutdown for Reconciler<T> {
    async fn shutdown(&mut self) {
        self.child.shutdown().await;
    }
}
