extern crate router as iron_router;

use iron::Request;
use iron::Response;

mod error;
mod logging;
mod metrics;
mod router;
mod sentry;

pub use self::error::into_ironerror;
pub use self::error::otr_into_ironerror;
pub use self::logging::middleware::RequestLogger;
pub use self::metrics::expose::MetricsHandler;
pub use self::metrics::observe::MetricsMiddleware;
#[cfg(feature = "with_test_support")]
pub use self::router::mock_request_span;
pub use self::router::request_span;
pub use self::router::RootDescriptor;
pub use self::router::RootedRouter;
pub use self::router::Router;
pub use self::sentry::SentryMiddleware;

/// Extracts the request method as a string.
fn request_method(request: &Request) -> String {
    request.method.to_string()
}

/// Extracts the request path as a string.
fn request_path(request: &Request) -> String {
    format!("/{}", request.url.path().join("/"))
}

/// Extracts the response status code as a string.
///
/// # Panics
/// If the response does not have a status set.
fn response_status(response: &Response) -> String {
    response
        .status
        .expect("Response instance does not have a status set")
        .to_u16()
        .to_string()
}
