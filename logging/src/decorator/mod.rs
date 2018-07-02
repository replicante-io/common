use slog::Logger;
use slog::Never;
use slog::SendSyncRefUnwindSafeDrain;
use slog::SendSyncUnwindSafeDrain;

use super::Config;
use super::Opts;


mod async;
mod level;


/// Apply decorators to the drain.
pub fn decorate<D>(config: Config, opts: Opts, drain: D) -> Logger
    where D: SendSyncUnwindSafeDrain<Ok = (), Err = Never>,
          D: 'static + SendSyncRefUnwindSafeDrain<Ok = (), Err = Never>,
{
    let drain = level::level(&config, drain);
    async::async(config, opts, drain)
}


/// Converts a [`Drain`] into a [`Logger`] setting global tags. 
///
/// [`Drain`]: slog/trait.Drain.html
/// [`Logger`]: slog/struct.Logger.html
pub fn into_logger<D>(opts: Opts, drain: D) -> Logger
    where D: SendSyncUnwindSafeDrain<Ok = (), Err = Never>,
          D: 'static + SendSyncRefUnwindSafeDrain<Ok = (), Err = Never>,
{
    Logger::root(drain, o!(
        "version" => opts.version,
    ))
}
