use failure::Fail;
use slog::error;
use slog::Logger;

use super::failure_info;

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
/// [`Fail`]: https://docs.rs/failure/0.1.5/failure/trait.Fail.html
pub fn capture_fail(logger: &Logger, error: &dyn Fail) {
    sentry::integrations::failure::capture_fail(error);
    error!(logger, "Capture unhandled failure"; failure_info(error));
}
