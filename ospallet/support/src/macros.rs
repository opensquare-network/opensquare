pub const RUNTIME_TARGET: &str = "runtime";

#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => (
        frame_support::debug::error!(target: $target, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::error!(target: RUNTIME_TARGET, $($arg)+);
    )
}

#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)+) => (
        frame_support::debug::warn!(target: $target, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::warn!(target: RUNTIME_TARGET, $($arg)+);
    )
}

#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => (
        frame_support::debug::info!(target: $target, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::info!(target: RUNTIME_TARGET, $($arg)+);
    )
}

#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => (
        frame_support::debug::debug!(target: $target, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::debug!(target: RUNTIME_TARGET, $($arg)+);
    )
}

#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => (
        frame_support::debug::trace!(target: $target, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::trace!(target: RUNTIME_TARGET, $($arg)+);
    )
}
