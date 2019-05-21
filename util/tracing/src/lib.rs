extern crate failure;
extern crate humthreads;
extern crate opentracingrust;
extern crate opentracingrust_zipkin;
extern crate reqwest;
extern crate serde;
extern crate serde_derive;
#[cfg(test)]
extern crate serde_yaml;
extern crate slog;

extern crate replicante_util_failure;
extern crate replicante_util_upkeep;

use std::time::Duration;

use opentracingrust::Tracer;
use slog::Logger;

use replicante_util_upkeep::Upkeep;

mod backends;
mod config;
mod error;

pub use self::config::Config;
pub use self::error::fail_span;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;

/// Additional options passed to tracer configuration.
pub struct Opts<'a> {
    flush_timeout: Duration,
    logger: Logger,
    service_name: &'a str,
    upkeep: &'a mut Upkeep,
}

impl<'a> Opts<'a> {
    pub fn new<S>(service_name: S, logger: Logger, upkeep: &'a mut Upkeep) -> Opts<'a>
    where
        S: Into<&'a str>,
    {
        Opts {
            flush_timeout: Duration::from_secs(1),
            logger,
            service_name: service_name.into(),
            upkeep,
        }
    }

    /// Set the muximum delay between span flushes.
    ///
    /// Some tracers' collectors allow this option to be set through the configuration.
    /// In that case, the value from the user configuration overrides this option.
    pub fn flush_timeout(mut self, timeout: Duration) -> Opts<'a> {
        self.flush_timeout = timeout;
        self
    }
}

/// Creates a new tracer based on the given configuration.
pub fn tracer(config: Config, opts: Opts) -> Result<Tracer> {
    match config {
        Config::Noop => self::backends::noop(),
        Config::Zipkin(config) => self::backends::zipkin(config, opts),
    }
}
