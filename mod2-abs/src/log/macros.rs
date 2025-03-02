//! Macros for generating log messages.


#[macro_export]
macro_rules! critical {
    ($threshold:expr, $($arg:tt)+) => {
        {
            $crate::log::init_logger();
            $crate::tracing::event!(
                $crate::tracing::Level::ERROR,
                critical = true,
                threshold = $threshold,
                message = format_args!($($arg)+)
            );
        }
    };
    ($($arg:tt)+) => {
        {
            $crate::log::init_logger();
            $crate::tracing::event!(
                $crate::tracing::Level::ERROR,
                critical = true,
                threshold = 0,
                message = format_args!($($arg)+)
            );
        }
    };
}

#[macro_export]
macro_rules! error {
    ($threshold:expr, $($arg:tt)+) => {
        {
            $crate::log::init_logger();
            $crate::tracing::event!(
                $crate::tracing::Level::ERROR,
                threshold = $threshold,
                message = format_args!($($arg)+)
            );
        }
    };
    ($($arg:tt)+) => {
        {
            $crate::log::init_logger();
            $crate::tracing::event!(
                $crate::tracing::Level::ERROR,
                threshold = 0,
                message = format_args!($($arg)+)
            );
        }
    };
}

#[macro_export]
macro_rules! warning {
    ($threshold:expr, $($arg:tt)+) => {
        {
            $crate::log::init_logger();
            $crate::tracing::event!(
                $crate::tracing::Level::WARN,
                threshold = $threshold,
                message = format_args!($($arg)+)
            );
        }
    };
    ($($arg:tt)+) => {
        {
            $crate::log::init_logger();
            $crate::tracing::event!(
                $crate::tracing::Level::WARN,
                threshold = 0,
                message = format_args!($($arg)+)
            );
        }
    };
}

#[macro_export]
macro_rules! info {
    ($threshold:expr, $($arg:tt)+) => {
        {
            $crate::log::init_logger();
            $crate::tracing::event!(
                $crate::tracing::Level::INFO,
                threshold = $threshold,
                message = format_args!($($arg)+)
            );
        }
    };
    ($($arg:tt)+) => {
        {
            $crate::log::init_logger();
            $crate::tracing::event!(
                $crate::tracing::Level::INFO,
                threshold = 0,
                message = format_args!($($arg)+)
            );
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($threshold:expr, $($arg:tt)+) => {
        {
            $crate::log::init_logger();
            $crate::tracing::event!(
                $crate::tracing::Level::DEBUG,
                threshold = $threshold,
                message = format_args!($($arg)+)
            );
        }
    };
    ($($arg:tt)+) => {
        {
            $crate::log::init_logger();
            $crate::tracing::event!(
                $crate::tracing::Level::DEBUG,
                threshold = 0,
                message = format_args!($($arg)+)
            );
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($threshold:expr, $($arg:tt)+) => {
        {
            $crate::log::init_logger();
            $crate::tracing::event!(
                $crate::tracing::Level::TRACE,
                threshold = $threshold,
                message = format_args!($($arg)+)
            );
        }
    };
    ($($arg:tt)+) => {
        {
            $crate::log::init_logger();
            $crate::tracing::event!(
                $crate::tracing::Level::TRACE,
                threshold = 0,
                message = format_args!($($arg)+)
            );
        }
    };
}


// The following makes the macros importable directly from the `log` module.
pub use {critical, error, warning, info, debug, trace};
