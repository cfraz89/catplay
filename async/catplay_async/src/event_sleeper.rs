use async_trait::async_trait;
use core::future::Future;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct EventToken(pub u64);

/// [EventSleeper] adds two-way async communication between itself and it's owner(parent), while keeping very convenient `&mut self` access for processing changes, and being runtime-agnostic.
///
/// It significantly reduces boilerplate when designing complex, hierarchical state machines split between multiple components represented by structs, removing complexity caused by - otherwise needed -
/// usage of `spawn` and a typical pattern of adding communication channels between the spawned task and it's owner represented by original `&mut self`.
///
/// The idea is for [EventSleeper] to `select!` on a set of Futures that signify that an important change in state was detected, and that requires internal reconcile
/// or communication with owner.
///
/// After `sleep` returns an [EventToken], owner can communicate with [EventSleeper] by calling struct-specific APIs like `reconcile()`, including ability
/// to pass references to structs that allow sending events(data) in reverse direction.
///
/// The [EventSleeper]'s owner is also performing `select!` between [EventSleeper]'s `sleep` and it's own set of futures - the sink's sleep can be cancelled at any moment to process incoming events.
///
/// Note that the sink shouldn't directly perform any critical state changes within `sleep` as the `sleep` future might be killed by the owner at any time, due to a new event.
///
/// **Implementation Invariants**
/// - the implementation **MAY** depend on regular calls to `sleep()` in order to make progress on it's async tasks, and the owner **SHOULD** perform regular `sleep()` calls
/// - the implementation **HAS TO** be cancel safe
/// - the implementation **MAY** continue to wake up the parent infinite number of times if parent does not drain relevant data post-sleep **OR**
///   it **MAY** deliver the wake up only once per event, creating a contract such that entering `sleep()` again **WITHOUT** draining **ALL** of the data leads to undefined behavior.
///
/// Note that the first wake up variant is completely valid if the goal is to allow consuming data at a slower rate between calls to `sleep()`, but with a guarantee that they will _eventually_ be consumed.
pub trait EventSleeper: Send {
    fn sleep(&mut self) -> impl Send + Future<Output = Option<EventToken>> {
        core::future::ready(None)
    }
}

impl<T: EventSleeper + ?Sized> EventSleeper for &mut T {
    #[inline(always)]
    async fn sleep(&mut self) -> Option<EventToken> {
        T::sleep(*self).await
    }
}

impl<T: EventSleeper> EventSleeper for Option<T> {
    #[inline(always)]
    async fn sleep(&mut self) -> Option<EventToken> {
        match self {
            None => None,
            Some(v) => v.sleep().await,
        }
    }
}

#[async_trait]
pub trait EventSleeperDyn: Send {
    async fn sleep_pinned(&mut self) -> Option<EventToken>;
}

#[async_trait]
impl<T: EventSleeper> EventSleeperDyn for T {
    async fn sleep_pinned(&mut self) -> Option<EventToken> {
        self.sleep().await
    }
}

impl<T: EventSleeperDyn + ?Sized> EventSleeper for Box<T> {
    async fn sleep(&mut self) -> Option<EventToken> {
        self.as_mut().sleep_pinned().await
    }
}

pub struct DynSleeper<'a, T: EventSleeperDyn + ?Sized>(&'a mut T);

pub fn dyn_sleeper<T: EventSleeperDyn + ?Sized>(value: &mut T) -> DynSleeper<'_, T> {
    DynSleeper(value)
}

impl<T: EventSleeperDyn + ?Sized> EventSleeper for DynSleeper<'_, T> {
    #[inline(always)]
    async fn sleep(&mut self) -> Option<EventToken> {
        self.0.sleep_pinned().await
    }
}

pub struct SleeperFut<Fut> {
    fut: Option<Fut>,
}

/// A simple bridge for structs that don't implement `EventSleeper`
/// and their APIs create `Futures` that generate events, but unlike `filling_slot`, the returned value can be discarded.
///
/// Example usage: `event_select!(self.watch.changed())` where `self.watch` is `tokio::sync::watch::Receiver<T>`.
pub fn sleeper<Fut>(fut: Fut) -> SleeperFut<Fut>
where
    Fut: Future,
{
    SleeperFut { fut: Some(fut) }
}

impl<Fut> EventSleeper for SleeperFut<Fut>
where
    Fut: Future + Send,
{
    #[inline(always)]
    async fn sleep(&mut self) -> Option<EventToken> {
        self.fut.take()?.await;
        Some(EventToken(1))
    }
}

/// Allows converting from `&self` to an instance of `EventSleeper` which requires a form of `&mut self` to be available.
/*pub*/
#[allow(unused)]
trait AsEventSleeper<'a> {
    type Output: EventSleeper;

    fn as_sleeper(input: Self) -> &'a mut Self::Output;
}

impl<'a, T: EventSleeper> AsEventSleeper<'a> for &'a mut T {
    type Output = T;

    fn as_sleeper(input: &'a mut T) -> &'a mut Self::Output {
        input
    }
}

#[cfg(test)]
mod tests {
    use super::{EventSleeper, EventToken};

    #[tokio::test]
    async fn mutable_reference_forwards_to_event_sleeper() {
        use super::AsEventSleeper;
        struct Sleeper;

        impl EventSleeper for Sleeper {
            async fn sleep(&mut self) -> Option<EventToken> {
                Some(EventToken(9))
            }
        }

        let mut sleeper1 = Sleeper;
        let mut sleeper2 = Sleeper;
        let sleeper_ref = &mut sleeper1;
        let mut optional_sleeper_ref = Some(&mut sleeper2);

        assert_eq!(sleeper_ref.sleep().await, Some(EventToken(9)));
        assert_eq!(optional_sleeper_ref.sleep().await, Some(EventToken(9)));

        AsEventSleeper::as_sleeper(sleeper_ref);
    }
}
