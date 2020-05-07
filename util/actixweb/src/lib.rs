use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use actix_web::web::ServiceConfig;

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

/// On-demand Actix Web `App` configuration with functions and closures.
#[derive(Clone)]
pub struct AppConfig<T> {
    configs: Vec<Arc<dyn Fn(&mut ServiceConfig, &T) + Send + Sync>>,
}

impl<T> AppConfig<T> {
    /// Run all the register handles to configure the given app.
    pub fn configure(&self, app: &mut ServiceConfig, context: &T) {
        for config in &self.configs {
            config(app, context);
        }
    }

    /// Register an app configuration function to be run later.
    pub fn register<F>(&mut self, config: F)
    where
        F: Fn(&mut ServiceConfig, &T) + 'static + Send + Sync,
    {
        self.configs.push(Arc::new(config));
    }
}

impl<T> Default for AppConfig<T> {
    fn default() -> Self {
        let configs = Vec::new();
        AppConfig { configs }
    }
}
