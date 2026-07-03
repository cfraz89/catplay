#[doc(hidden)]
pub struct EventSelectPtr<T>(*mut T);

impl<T> EventSelectPtr<T> {
    #[inline(always)]
    pub fn new(value: &mut T) -> Self {
        Self(value)
    }

    #[inline(always)]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn into_mut<'a>(self) -> &'a mut T {
        unsafe { &mut *self.0 }
    }
}

impl<T> Copy for EventSelectPtr<T> {}

impl<T> Clone for EventSelectPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

unsafe impl<T: Send> Send for EventSelectPtr<T> {}
unsafe impl<T: Send> Sync for EventSelectPtr<T> {}

#[macro_export]
macro_rules! event_select {
    ( future; $($branches:tt)+ ) => {{
        $crate::__event_select! { $($branches)+ }
    }};

    ( $($branches:tt)+ ) => {{
        $crate::__event_select! { $($branches)+ }.await
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __event_select {
    ( $($branches:tt)+ ) => {{
        $crate::__event_select_parse! {
            [(__es_sleep_01 __es_fut_01) (__es_sleep_02 __es_fut_02) (__es_sleep_03 __es_fut_03) (__es_sleep_04 __es_fut_04) (__es_sleep_05 __es_fut_05)
             (__es_sleep_06 __es_fut_06) (__es_sleep_07 __es_fut_07) (__es_sleep_08 __es_fut_08) (__es_sleep_09 __es_fut_09) (__es_sleep_10 __es_fut_10)
             (__es_sleep_11 __es_fut_11) (__es_sleep_12 __es_fut_12) (__es_sleep_13 __es_fut_13) (__es_sleep_14 __es_fut_14) (__es_sleep_15 __es_fut_15)
             (__es_sleep_16 __es_fut_16) (__es_sleep_17 __es_fut_17) (__es_sleep_18 __es_fut_18) (__es_sleep_19 __es_fut_19) (__es_sleep_20 __es_fut_20)]
            []
            $($branches)+,
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __event_select_parse {
    (
        [$($slots:tt)*]
        [ $( ($sleeper:expr, $handler:expr), )+ ]
    ) => {
        $crate::__event_select_impl! {
            [$($slots)*]
            []
            $( ($sleeper, $handler) ),+
        }
    };

    (
        [$($slots:tt)*]
        [ $( ($sleeper:expr, $handler:expr), )* ]
        $sleeper0:expr => $handler0:expr,
        $($rest:tt)*
    ) => {
        $crate::__event_select_parse! {
            [$($slots)*]
            [ $( ($sleeper, $handler), )* ($sleeper0, $handler0), ]
            $($rest)*
        }
    };

    (
        [$($slots:tt)*]
        [ $( ($sleeper:expr, $handler:expr), )* ]
        $sleeper0:expr,
        $($rest:tt)*
    ) => {
        $crate::__event_select_parse! {
            [$($slots)*]
            [ $( ($sleeper, $handler), )* ($sleeper0, Some), ]
            $($rest)*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __event_select_poll_branch {
    ($cx:expr, $slot:ident, $handler:expr) => {{
        let __state = {
            let __fut = $slot.1.as_mut().expect("event_select: empty future slot");
            // SAFETY:
            // - future lives in a stable local slot inside the async block
            // - while it is Pending, we never move or replace it
            // - replacement happens only after Poll::Ready(None)
            match core::future::Future::poll(unsafe { core::pin::Pin::new_unchecked(__fut) }, $cx) {
                core::task::Poll::Ready(Some(__tok)) => Some(Ok(($handler)(__tok))),
                core::task::Poll::Ready(None) => Some(Err(())),
                core::task::Poll::Pending => None,
            }
        };

        match __state {
            Some(Ok(__ret)) => return core::task::Poll::Ready(__ret),
            Some(Err(())) => {
                $slot.1 = None;
                $slot.1 = Some($crate::EventSleeper::sleep(unsafe { $slot.0.into_mut() }));
            }
            None => {}
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __event_select_impl {
    (
        [($sleep_next:ident $fut_next:ident) $(($sleep_rest:ident $fut_rest:ident))*]
        [ $( ($sleep_slot:ident, $fut_slot:ident, $sleeper:expr, $handler:expr), )* ]
        ($sleeper0:expr, $handler0:expr),
        $( ($sleeperN:expr, $handlerN:expr) ),+
    ) => {
        $crate::__event_select_impl! {
            [$(($sleep_rest $fut_rest))*]
            [ $( ($sleep_slot, $fut_slot, $sleeper, $handler), )* ($sleep_next, $fut_next, $sleeper0, $handler0), ]
            $( ($sleeperN, $handlerN) ),+
        }
    };

    (
        [($sleep_next:ident $fut_next:ident) $(($sleep_rest:ident $fut_rest:ident))*]
        [ $( ($sleep_slot:ident, $fut_slot:ident, $sleeper:expr, $handler:expr), )* ]
        ($sleeper0:expr, $handler0:expr)
    ) => {{
        async {
            use core::future::poll_fn;
            use core::task::Poll;

            let seed = 0usize;
            let __branch_count = 1usize $(+ { let _ = stringify!($fut_slot); 1usize })*;
            let __start = seed % __branch_count;

            $(
                let mut $sleep_slot = &mut ($sleeper);
                let mut $fut_slot = {
                    let __sleeper = $crate::EventSelectPtr::new(&mut $sleep_slot);
                    let __future = $crate::EventSleeper::sleep(unsafe { __sleeper.into_mut() });
                    (__sleeper, Some(__future))
                };
            )*
            let mut $sleep_next = &mut ($sleeper0);
            let mut $fut_next = {
                let __sleeper = $crate::EventSelectPtr::new(&mut $sleep_next);
                let __future = $crate::EventSleeper::sleep(unsafe { __sleeper.into_mut() });
                (__sleeper, Some(__future))
            };

            poll_fn(|cx| {
                let mut __branch = 0usize;

                $(
                    if __branch >= __start {
                        $crate::__event_select_poll_branch!(cx, $fut_slot, $handler);
                    }
                    __branch += 1;
                )*

                if __branch >= __start {
                    $crate::__event_select_poll_branch!(cx, $fut_next, $handler0);
                }
                __branch = 0usize;

                $(
                    if __branch < __start {
                        $crate::__event_select_poll_branch!(cx, $fut_slot, $handler);
                    }
                    __branch += 1;
                )*

                if __branch < __start {
                    $crate::__event_select_poll_branch!(cx, $fut_next, $handler0);
                }

                Poll::Pending
            }).await
        }
    }};

    (
        []
        [ $( ($sleep_slot:ident, $fut_slot:ident, $sleeper:expr, $handler:expr) ),* ]
        $($rest:tt)+
    ) => {
        compile_error!("event_select! supports at most 20 branches");
    };
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use crate::{EventSleeper, EventToken};

    struct Noop;

    impl crate::EventSleeper for Noop {
        async fn sleep(&mut self) -> Option<EventToken> {
            Some(EventToken(1))
        }
    }

    struct Disabled;

    impl crate::EventSleeper for Disabled {
        async fn sleep(&mut self) -> Option<EventToken> {
            None
        }
    }

    struct Test {
        child: Noop,
    }

    impl EventSleeper for Test {
        async fn sleep(&mut self) -> Option<EventToken> {
            event_select!(self.child => Some)
        }
    }

    async fn test_compile() {}

    #[tokio::test]
    async fn test() {
        let mut sleeper1 = Noop;
        let mut sleeper2 = Noop;
        let mut sleeper3 = Noop;
        let mut sleeper4 = Noop;
        let _ = event_select!(sleeper1 => |_: EventToken| {1});
        let _ = event_select!(sleeper1 => |_| {1}, sleeper2 => |_| {2});
        let _ = event_select!(sleeper1);
        let _ = event_select!(sleeper1 => Some);
        let _ = event_select!(sleeper1, sleeper2 => |_| { Some(EventToken(2)) });
        let _ = event_select!(sleeper1, sleeper2, sleeper3);
        let _ = tokio::select! {
            t = event_select!(future; sleeper1, sleeper2 => Some, sleeper3, sleeper4) => t,
        };
    }

    #[tokio::test]
    async fn none_branch_does_not_wake_select() {
        let mut disabled = Disabled;

        tokio::select! {
            _ = event_select!(future; disabled) => panic!("None branch should not wake"),
            _ = tokio::task::yield_now() => {}
        }
    }

    #[tokio::test]
    async fn none_branch_does_not_block_ready_branch() {
        let mut disabled = Disabled;
        let mut sleeper = Noop;

        assert_eq!(event_select!(disabled, sleeper), Some(EventToken(1)));
    }
}
