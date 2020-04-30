use std::future::Future;
use std::pin::Pin;

mod descriptor;
mod error;
mod logging;
mod metrics;
mod sentry;
mod tracing;

pub use self::descriptor::APIFlags;
pub use self::descriptor::RootDescriptor;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::logging::LoggingMiddleware;
pub use self::metrics::MetricsCollector;
pub use self::metrics::MetricsExporter;
pub use self::metrics::MetricsMiddleware;
pub use self::sentry::ActixWebHubExt;
pub use self::sentry::SentryMiddleware;
pub use self::tracing::request_span;
pub use self::tracing::TracingMiddleware;

/// Type alias for futures returned by middleweres to keep clippy happy.
type BoxedFuture<R, E> = Pin<Box<dyn Future<Output = std::result::Result<R, E>>>>;
