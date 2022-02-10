use std::sync::Arc;
use std::time::Duration;

use opentracingrust::Tracer;
use slog::Logger;

use replicante_util_upkeep::Upkeep;

mod backends;
pub mod carriers;
mod config;
mod error;

pub use self::config::Config;
pub use self::error::fail_span;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;

/// Wrapper for easier optional `Tracer`s.
#[derive(Clone)]
pub struct MaybeTracer(Option<Arc<Tracer>>);

impl MaybeTracer {
    pub fn new<T>(tracer: T) -> MaybeTracer
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        MaybeTracer(tracer.into())
    }

    /// Execute the block if a `Tracer` is available.
    pub fn with<B, T>(&self, block: B) -> Option<T>
    where
        B: FnOnce(&Tracer) -> T,
    {
        self.0.as_ref().map(|tracer| block(tracer))
    }
}

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
        Config::Noop => self::backends::noop(opts),
        Config::Zipkin(config) => self::backends::zipkin(config, opts),
    }
}
