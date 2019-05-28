extern crate serde;
extern crate serde_derive;
extern crate slog;
extern crate slog_async;
#[cfg(feature = "journald")]
extern crate slog_journald;
extern crate slog_json;

use std::io::stdout;
use std::sync::Mutex;

use slog::o;
use slog::Drain;
use slog::FnValue;
use slog::IgnoreResult;
use slog::Logger;
use slog::Record;
#[cfg(feature = "journald")]
use slog_journald::JournaldDrain;
use slog_json::Json;

mod config;
mod decorator;
mod options;

pub use self::config::Config;
pub use self::config::LoggingLevel;
pub use self::options::Opts;

use self::config::LoggingBackend;
use self::decorator::decorate;

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
            // rustc can't infer lifetimes correctly when using Record::module.
            // Without this allow, clipply complainants that we do not use Record::module.
            #[allow(clippy::redundant_closure)]
            let drain = Json::new(stdout())
                .add_default_keys()
                .add_key_value(o!(
                    "module" => FnValue(
                        |rinfo: &Record| rinfo.module()
                    )
                ))
                .build();
            let drain = Mutex::new(drain).map(IgnoreResult::new);
            decorate(config, opts, drain)
        }
    }
}

/// Creates a fixed [`Logger`] to be used until configuration is loaded.
///
/// [`Logger`]: slog/struct.Logger.html
pub fn starter(opts: &Opts) -> Logger {
    // rustc can't infer lifetimes correctly when using Record::module.
    // Without this allow, clipply complainants that we do not use Record::module.
    #[allow(clippy::redundant_closure)]
    let drain = Json::new(stdout())
        .add_default_keys()
        .add_key_value(o!("module" => FnValue(|rinfo : &Record| rinfo.module())))
        .build();
    let drain = Mutex::new(drain).map(IgnoreResult::new);
    decorator::into_logger(opts, drain)
}
