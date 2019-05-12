use failure::Fail;

/// Capture a [`Fail`] and prevent propagation.
///
/// The main purpose is to provide a standard and reusable logic for dealing with
/// errors that should be reported but not propagated.
///
/// [`Fail`]s are:
///
///   * Reported to sentry (if enabled)
///   * Logged to the provided logger.
///
/// # Example
/// ```ignore
/// use replicante_util_failure::capture_fail;
/// use replicante_util_failure::failure_info;
///
/// capture_fail!(&error, logger, "logging: {}", "test");
/// capture_fail!(&error, logger, "logging: {}", "test"; "key" => ?"value");
/// capture_fail!(&error, logger, "logging: {}", "test"; failure_info(&error));
/// ```
///
/// [`Fail`]: https://docs.rs/failure/0.1.5/failure/trait.Fail.html
#[macro_export]
macro_rules! capture_fail(
    ($error:expr, $($args:tt)+) => {
        $crate::capture_fail_inner($error);
        slog::error!($($args)+);
    }
);

/// Helper function called from the `capture_fail` macro for extra processing.
#[doc(hidden)]
pub fn capture_fail_inner(error: &dyn Fail) {
    sentry::integrations::failure::capture_fail(error);
}

#[cfg(test)]
mod tests {
    use slog::o;
    use slog::Discard;
    use slog::Logger;

    use super::super::failure_info;

    #[test]
    fn capture_with_log() {
        let logger = Logger::root(Discard, o!());
        let error = failure::err_msg("test");
        capture_fail!(error.as_fail(), logger, "logging: {}", "test");
        capture_fail!(error.as_fail(), logger, "logging: {}", "test"; failure_info(error.as_fail()));
        capture_fail!(error.as_fail(), logger, "logging: {}", "test"; "key" => ?"value");
    }
}
