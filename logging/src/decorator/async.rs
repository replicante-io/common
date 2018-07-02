use slog::Drain;
use slog::Logger;
use slog::Never;
use slog::SendSyncRefUnwindSafeDrain;
use slog::SendSyncUnwindSafeDrain;
use slog_async::Async;

use super::Config;
use super::Opts;
use super::into_logger;



/// Optionally wrap the drain into an [`Async`] drain.
///
/// [`Async`]: slog_async/struct.Async.html
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
pub fn async<D>(config: Config, opts: &Opts, drain: D) -> Logger
    where D: SendSyncUnwindSafeDrain<Ok = (), Err = Never>,
          D: 'static + SendSyncRefUnwindSafeDrain<Ok = (), Err = Never>, 
{
    if config.async {
        into_logger(opts, Async::new(drain).build().ignore_res())
    } else {
        into_logger(opts, drain)
    }
}
