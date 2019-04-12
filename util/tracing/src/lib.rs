extern crate failure;
extern crate opentracingrust;
extern crate opentracingrust_zipkin;

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde_yaml;
#[macro_use]
extern crate slog;

use opentracingrust::utils::ReporterThread;
use opentracingrust::Tracer;

use slog::Logger;

mod backends;
mod config;
mod error;

pub use self::config::Config;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;

/// Tracer-dependent additional returns options.
///
/// Returned along-side the tracer itself to pass back any thread needed my the tracer.
pub enum TracerExtra {
    /// The tracer has no extra returns to provide.
    Nothing,

    /// The tracer's `ReporterThread` that send spans to the backend.
    ReporterThread(ReporterThread),
}

/// Creates a new tracer based on the given configuration.
pub fn tracer(config: Config, logger: Logger) -> Result<(Tracer, TracerExtra)> {
    match config {
        Config::Noop => self::backends::noop(),
        Config::Zipkin(config) => self::backends::zipkin(config, logger),
    }
}
