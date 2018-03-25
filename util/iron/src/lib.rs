extern crate iron;
#[cfg(test)]
extern crate iron_test;
#[cfg(test)]
extern crate router;

extern crate prometheus;
#[macro_use]
extern crate slog;


mod metrics;

pub use self::metrics::MetricsMiddleware;
