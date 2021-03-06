use slog::o;
use slog::Logger;
use slog::Never;
use slog::SendSyncRefUnwindSafeDrain;
use slog::SendSyncUnwindSafeDrain;

use super::Config;
use super::Opts;

mod async_flush;
mod level;

/// Apply decorators to the drain.
pub fn decorate<D>(config: Config, opts: &Opts, drain: D) -> Logger
where
    D: 'static
        + SendSyncUnwindSafeDrain<Ok = (), Err = Never>
        + SendSyncRefUnwindSafeDrain<Ok = (), Err = Never>,
{
    let drain = level::level(&config, drain);
    async_flush::async_flush(config, opts, drain)
}

/// Converts a [`Drain`] into a [`Logger`] setting global tags.
///
/// [`Drain`]: slog/trait.Drain.html
/// [`Logger`]: slog/struct.Logger.html
pub fn into_logger<D>(opts: &Opts, drain: D, include_version: bool) -> Logger
where
    D: 'static
        + SendSyncUnwindSafeDrain<Ok = (), Err = Never>
        + SendSyncRefUnwindSafeDrain<Ok = (), Err = Never>,
{
    if include_version {
        Logger::root(drain, o!("version" => opts.version.clone()))
    } else {
        Logger::root(drain, o!())
    }
}
