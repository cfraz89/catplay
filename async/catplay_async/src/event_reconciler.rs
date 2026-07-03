use async_trait::async_trait;

/// [EventReconciler] performs state reconcilation on `&mut self` after `sleep()`, effectively creating a `sleep-reconcile` error-resistant reconciliation loop, controlled by parent(owner).
///
/// It's a central part of implementation's state machine - aiming to consume all data and events that happened between calls to `sleep()`, and run all sanity checks to decide if it should enter an error state.
///
/// **Implementation Invariants**
/// - call to `reconcile()` **SHOULD** always happen before first `sleep()`
/// - implementations **MAY** continue repeating a stored `Error` on next calls to `reconcile()`, this is however not a hard requirement, especially for non-Clonable Errors.
/// - implementations are **ABSOLUTELY NOT** cancel-safe. If a `reconcile()` future was cancelled, the whole struct needs to be dropped to retain sanity.
///   **This may be enforced in future with a panic.**
/// - implementations **MAY** perform async blocking work for extended amount of time(up to a couple of seconds) **OR** manage their own task queues for such blocking work
///   and treat `reconcile()` as a sanity check observing errors from such task queue
/// - using any other APIs of the implementation after `Error` is observed from `reconcile()` **MAY** lead to undefined behavior
pub trait EventReconciler: Send {
    type Error: Send;

    fn reconcile(&mut self) -> impl Send + Future<Output = Result<(), Self::Error>> {
        core::future::ready(Ok(()))
    }
}

impl<T: EventReconciler> EventReconciler for Option<T> {
    type Error = T::Error;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        match self {
            None => Ok(()),
            Some(v) => v.reconcile().await,
        }
    }
}

#[async_trait]
pub trait EventReconcilerDyn: Send {
    type Error: Send;

    async fn reconcile_pinned(&mut self) -> Result<(), Self::Error>;
}

#[async_trait]
impl<T: EventReconciler> EventReconcilerDyn for T {
    type Error = T::Error;

    async fn reconcile_pinned(&mut self) -> Result<(), Self::Error> {
        self.reconcile().await
    }
}

impl<T: EventReconcilerDyn + ?Sized> EventReconciler for Box<T> {
    type Error = T::Error;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        self.as_mut().reconcile_pinned().await
    }
}
