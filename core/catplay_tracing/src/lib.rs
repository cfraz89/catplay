pub mod hexdump;
#[cfg(feature = "std")]
pub mod logger;
#[cfg(feature = "std")]
pub mod lowprio_queue;
pub mod tracer;

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "std")]
macro_rules! __alog_internal {
    (target: $target:expr, $level:expr, $($arg:tt)+) => {{
        $crate::logger::enqueue_async_log(
            $crate::logger::macro_now_instant(),
            $level,
            $target,
            module_path!(),
            file!(),
            line!(),
            move || $crate::logger::macro_format_owned(::core::format_args!($($arg)+)),
        );
    }};
    ($level:expr, $($arg:tt)+) => {{
        $crate::__alog_internal!(target: module_path!(), $level, $($arg)+)
    }};
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "std"))]
macro_rules! __alog_internal {
    (target: $target:expr, $level:expr, $($arg:tt)+) => {{
        ::log::log!(target: $target, $level, $($arg)+);
    }};
    ($level:expr, $($arg:tt)+) => {{
        $crate::__alog_internal!(target: module_path!(), $level, $($arg)+)
    }};
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "std")]
macro_rules! __tlog_internal {
    ($logger:expr, target: $target:expr, $level:expr, $($arg:tt)+) => {{
        $crate::logger::enqueue_async_log_for_logger(
            $logger,
            $crate::logger::macro_now_instant(),
            $level,
            $target,
            module_path!(),
            file!(),
            line!(),
            move || $crate::logger::macro_format_owned(::core::format_args!($($arg)+)),
        );
    }};
    ($logger:expr, $level:expr, $($arg:tt)+) => {{
        $crate::__tlog_internal!($logger, target: module_path!(), $level, $($arg)+)
    }};
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "std"))]
macro_rules! __tlog_internal {
    ($logger:expr, target: $target:expr, $level:expr, $($arg:tt)+) => {{
        let _ = &$logger;
        ::log::log!(target: $target, $level, $($arg)+);
    }};
    ($logger:expr, $level:expr, $($arg:tt)+) => {{
        $crate::__tlog_internal!($logger, target: module_path!(), $level, $($arg)+)
    }};
}

#[macro_export]
macro_rules! aerror {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::__alog_internal!(target: $target, ::log::Level::Error, $($arg)+)
    };
    ($($arg:tt)+) => {
        $crate::__alog_internal!(::log::Level::Error, $($arg)+)
    };
}

#[macro_export]
macro_rules! awarn {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::__alog_internal!(target: $target, ::log::Level::Warn, $($arg)+)
    };
    ($($arg:tt)+) => {
        $crate::__alog_internal!(::log::Level::Warn, $($arg)+)
    };
}

#[macro_export]
macro_rules! ainfo {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::__alog_internal!(target: $target, ::log::Level::Info, $($arg)+)
    };
    ($($arg:tt)+) => {
        $crate::__alog_internal!(::log::Level::Info, $($arg)+)
    };
}

#[macro_export]
macro_rules! adebug {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::__alog_internal!(target: $target, ::log::Level::Debug, $($arg)+)
    };
    ($($arg:tt)+) => {
        $crate::__alog_internal!(::log::Level::Debug, $($arg)+)
    };
}

#[macro_export]
macro_rules! atrace {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::__alog_internal!(target: $target, ::log::Level::Trace, $($arg)+)
    };
    ($($arg:tt)+) => {
        $crate::__alog_internal!(::log::Level::Trace, $($arg)+)
    };
}

#[macro_export]
macro_rules! terror {
    ($logger:expr, target: $target:expr, $($arg:tt)+) => {
        $crate::__tlog_internal!($logger, target: $target, ::log::Level::Error, $($arg)+)
    };
    ($logger:expr, $($arg:tt)+) => {
        $crate::__tlog_internal!($logger, ::log::Level::Error, $($arg)+)
    };
}

#[macro_export]
macro_rules! twarn {
    ($logger:expr, target: $target:expr, $($arg:tt)+) => {
        $crate::__tlog_internal!($logger, target: $target, ::log::Level::Warn, $($arg)+)
    };
    ($logger:expr, $($arg:tt)+) => {
        $crate::__tlog_internal!($logger, ::log::Level::Warn, $($arg)+)
    };
}

#[macro_export]
macro_rules! tinfo {
    ($logger:expr, target: $target:expr, $($arg:tt)+) => {
        $crate::__tlog_internal!($logger, target: $target, ::log::Level::Info, $($arg)+)
    };
    ($logger:expr, $($arg:tt)+) => {
        $crate::__tlog_internal!($logger, ::log::Level::Info, $($arg)+)
    };
}

#[macro_export]
macro_rules! tdebug {
    ($logger:expr, target: $target:expr, $($arg:tt)+) => {
        $crate::__tlog_internal!($logger, target: $target, ::log::Level::Debug, $($arg)+)
    };
    ($logger:expr, $($arg:tt)+) => {
        $crate::__tlog_internal!($logger, ::log::Level::Debug, $($arg)+)
    };
}

#[macro_export]
macro_rules! ttrace {
    ($logger:expr, target: $target:expr, $($arg:tt)+) => {
        $crate::__tlog_internal!($logger, target: $target, ::log::Level::Trace, $($arg)+)
    };
    ($logger:expr, $($arg:tt)+) => {
        $crate::__tlog_internal!($logger, ::log::Level::Trace, $($arg)+)
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "std")]
macro_rules! __slog_internal {
    ($tracer:expr, $delegate_macro:ident, target: $target:expr, $($arg:tt)+) => {{
        $crate::$delegate_macro!($tracer.logger(), target: $target, $($arg)+);
    }};
    ($tracer:expr, $delegate_macro:ident, $($arg:tt)+) => {{
        $crate::$delegate_macro!($tracer.logger(), $($arg)+);
    }};
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "std"))]
macro_rules! __slog_internal {
    ($tracer:expr, $delegate_macro:ident, target: $target:expr, $($arg:tt)+) => {{
        let _ = &$tracer;
    }};
    ($tracer:expr, $delegate_macro:ident, $($arg:tt)+) => {{
        let _ = &$tracer;
    }};
}

#[macro_export]
macro_rules! sterror {
    ($tracer:expr, target: $target:expr, $($arg:tt)+) => {
        $crate::__slog_internal!($tracer, terror, target: $target, $($arg)+)
    };
    ($tracer:expr, $($arg:tt)+) => {
        $crate::__slog_internal!($tracer, terror, $($arg)+)
    };
}

#[macro_export]
macro_rules! stwarn {
    ($tracer:expr, target: $target:expr, $($arg:tt)+) => {
        $crate::__slog_internal!($tracer, twarn, target: $target, $($arg)+)
    };
    ($tracer:expr, $($arg:tt)+) => {
        $crate::__slog_internal!($tracer, twarn, $($arg)+)
    };
}

#[macro_export]
macro_rules! stinfo {
    ($tracer:expr, target: $target:expr, $($arg:tt)+) => {
        $crate::__slog_internal!($tracer, tinfo, target: $target, $($arg)+)
    };
    ($tracer:expr, $($arg:tt)+) => {
        $crate::__slog_internal!($tracer, tinfo, $($arg)+)
    };
}

#[macro_export]
macro_rules! stdebug {
    ($tracer:expr, target: $target:expr, $($arg:tt)+) => {
        $crate::__slog_internal!($tracer, tdebug, target: $target, $($arg)+)
    };
    ($tracer:expr, $($arg:tt)+) => {
        $crate::__slog_internal!($tracer, tdebug, $($arg)+)
    };
}

#[macro_export]
macro_rules! strace {
    ($tracer:expr, target: $target:expr, $($arg:tt)+) => {
        $crate::__slog_internal!($tracer, ttrace, target: $target, $($arg)+)
    };
    ($tracer:expr, $($arg:tt)+) => {
        $crate::__slog_internal!($tracer, ttrace, $($arg)+)
    };
}
