use slog::Drain;
use slog::Logger;
use slog::Never;
use slog::SendSyncRefUnwindSafeDrain;
use slog::SendSyncUnwindSafeDrain;
use slog_async::Async;

use super::into_logger;
use super::Config;
use super::Opts;

/// Optionally wrap the drain into an [`Async`] drain.
///
/// [`Async`]: slog_async/struct.Async.html
#[allow(clippy::needless_pass_by_value)]
pub fn async_flush<D>(config: Config, opts: &Opts, drain: D) -> Logger
where
    D: SendSyncUnwindSafeDrain<Ok = (), Err = Never>,
    D: 'static + SendSyncRefUnwindSafeDrain<Ok = (), Err = Never>,
{
    if config.async_flush {
        into_logger(opts, Async::new(drain).build().ignore_res())
    } else {
        into_logger(opts, drain)
    }
}
