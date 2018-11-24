extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate slog;
extern crate slog_async;
#[cfg(feature = "journald")]
extern crate slog_journald;
extern crate slog_json;


use std::io::stdout;
use std::sync::Mutex;

use slog::Drain;
use slog::IgnoreResult;
use slog::Logger;

use slog::FnValue;
use slog::Record;

#[cfg(feature = "journald")]
use slog_journald::JournaldDrain;
use slog_json::Json;


mod config;
mod decorator;
mod options;

pub use config::Config;
pub use config::LoggingLevel;
pub use options::Opts;

use config::LoggingBackend;
use decorator::decorate;


/// Creates a [`Logger`] based on the given configuration.
///
/// This is the first function in a list of generic functions.
/// The intermediate configuration stages, while all compatible with the [`Drain`] trait,
/// have different concrete types.
/// Using generic functions allows code reuse without repeatedly boxing intermediate steps.
///
/// [`Drain`]: slog/trait.Drain.html
/// [`Logger`]: slog/struct.Logger.html
pub fn configure(config: Config, opts: &Opts) -> Logger {
    match config.backend {
        #[cfg(feature = "journald")]
        LoggingBackend::Journald => decorate(config, opts, JournaldDrain.ignore_res()),
        LoggingBackend::Json => {
            let drain = Json::new(stdout())
                .add_default_keys()
                .add_key_value(o!("module" => FnValue(|rinfo : &Record| rinfo.module())))
                .build();
            let drain = Mutex::new(drain).map(IgnoreResult::new);
            decorate(config, opts, drain)
        },
    }
}


/// Creates a fixed [`Logger`] to be used until configuration is loaded.
///
/// [`Logger`]: slog/struct.Logger.html
pub fn starter(opts: &Opts) -> Logger {
    let drain = Json::new(stdout())
        .add_default_keys()
        .add_key_value(o!("module" => FnValue(|rinfo : &Record| rinfo.module())))
        .build();
    let drain = Mutex::new(drain).map(IgnoreResult::new);
    decorator::into_logger(opts, drain)
}
