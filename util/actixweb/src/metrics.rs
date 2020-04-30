use std::task::Context;
use std::task::Poll;
use std::time::Duration;
use std::time::Instant;

use actix_service::Service;
use actix_service::Transform;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use futures::future::ok;
use futures::future::Ready;
use prometheus::CounterVec;
use prometheus::Encoder;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;
use prometheus::TextEncoder;
use slog::debug;
use slog::Logger;

/// Set of metrics tracked by the `MetricsMiddleware` for actix web.
#[derive(Clone)]
pub struct MetricsCollector {
    duration: HistogramVec,
    errors: CounterVec,
}

impl MetricsCollector {
    /// Create a new set of metrics with the given prefix.
    pub fn new<S>(prefix: S) -> MetricsCollector
    where
        S: AsRef<str>,
    {
        let prefix = prefix.as_ref();
        let duration = HistogramVec::new(
            HistogramOpts::new(
                format!("{}_endpoint_duration", prefix).as_str(),
                "Duration (in seconds) of HTTP endpoints",
            ),
            &["method", "path", "status"],
        )
        .expect("unable to configure API duration histogram");
        let errors = CounterVec::new(
            Opts::new(
                format!("{}_endpoint_errors", prefix).as_str(),
                "Number of errors encountered while handling requests",
            ),
            &["method", "path", "status"],
        )
        .expect("unable to configure API errors counter");
        MetricsCollector { duration, errors }
    }

    /// Register this set of metrics with the registry.
    pub fn register(&self, logger: &Logger, registry: &Registry) {
        if let Err(error) = registry.register(Box::new(self.duration.clone())) {
            debug!(logger, "Failed to register MetricsMiddleware::duration"; "error" => ?error);
        }
        if let Err(error) = registry.register(Box::new(self.errors.clone())) {
            debug!(logger, "Failed to register MetricsMiddleware::errors"; "error" => ?error);
        }
    }
}

/// ActixWeb `Responder` to export prometheus metrics.
pub struct MetricsExporter {
    registry: Registry,
}

impl MetricsExporter {
    pub fn new(registry: Registry) -> MetricsExporter {
        MetricsExporter { registry }
    }
}

impl Responder for MetricsExporter {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        let metric_familys = self.registry.gather();
        encoder.encode(&metric_familys, &mut buffer).unwrap();
        let response = HttpResponse::Ok()
            .header(actix_web::http::header::CONTENT_TYPE, encoder.format_type())
            .body(buffer);
        ok(response)
    }
}

/// Actix Web middleware to capture request metrics.
pub struct MetricsMiddleware {
    metrics: MetricsCollector,
}

impl MetricsMiddleware {
    pub fn new(metrics: MetricsCollector) -> MetricsMiddleware {
        MetricsMiddleware { metrics }
    }
}

// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for MetricsMiddleware
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MiddlewareService {
            metrics: self.metrics.clone(),
            service,
        })
    }
}

/// Inner middleware to process requests on behalf of `MetricsMiddleware`.
pub struct MiddlewareService<S> {
    metrics: MetricsCollector,
    service: S,
}

impl<S, B> Service for MiddlewareService<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = crate::BoxedFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, ctx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let metrics = self.metrics.clone();
        let request_start = Instant::now();
        let response = self.service.call(req);
        Box::pin(async move {
            let response = response.await?;
            let duration = duration_to_seconds(request_start.elapsed());
            let method = response.request().method().as_str();
            let path = response.request().path();
            let status = response.response().status();
            metrics
                .duration
                .with_label_values(&[method, path, status.as_str()])
                .observe(duration);
            if response.response().error().is_some() {
                metrics
                    .errors
                    .with_label_values(&[method, path, status.as_str()])
                    .inc();
            }
            Ok(response)
        })
    }
}

/// Convert a [request] duration to seconds.
fn duration_to_seconds(duration: Duration) -> f64 {
    let nanos = f64::from(duration.subsec_nanos()) / 1e9;
    duration.as_secs() as f64 + nanos
}
