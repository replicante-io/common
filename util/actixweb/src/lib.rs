use std::future::Future;
use std::pin::Pin;

mod config;
mod descriptor;
mod logging;
mod metrics;
mod tracing;

pub mod errors;

pub use self::config::AppConfig;
pub use self::config::AppConfigContext;
pub use self::descriptor::APIFlags;
pub use self::descriptor::RootDescriptor;
pub use self::logging::LoggingMiddleware;
pub use self::metrics::MetricsCollector;
pub use self::metrics::MetricsExporter;
pub use self::metrics::MetricsMiddleware;
pub use self::tracing::with_request_span;
pub use self::tracing::TracingMiddleware;

/// Type alias for futures returned by middleware.
// (from futures_util but I did not want the whole crate)
pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
