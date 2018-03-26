extern crate iron;
#[cfg(test)]
extern crate iron_test;
#[cfg(test)]
extern crate router;

extern crate opentracingrust;
extern crate prometheus;

#[macro_use]
extern crate slog;

use iron::Request;
use iron::Response;


mod logging;
mod metrics;
mod tracing;

pub use self::logging::middleware::RequestLogger;
pub use self::metrics::expose::MetricsHandler;
pub use self::metrics::observe::MetricsMiddleware;
pub use self::tracing::carrier::HeadersCarrier;


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
    response.status.expect("Response instance does not have a status set").to_u16().to_string()
}
