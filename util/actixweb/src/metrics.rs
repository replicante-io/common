use std::future::ready;
use std::future::Ready;
use std::time::Duration;
use std::time::Instant;

use actix_web::dev::forward_ready;
use actix_web::dev::Service;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::dev::Transform;
use actix_web::Error;
use actix_web::HttpResponse;
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
#[derive(Clone)]
pub struct MetricsExporter {
    registry: Registry,
}

impl MetricsExporter {
    pub fn with_registry(registry: Registry) -> MetricsExporter {
        MetricsExporter { registry }
    }
}

impl actix_web::Handler<()> for MetricsExporter {
    type Output = HttpResponse;
    type Future = Ready<Self::Output>;

    fn call(&self, _: ()) -> Self::Future {
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        let response = HttpResponse::Ok()
            .append_header((actix_web::http::header::CONTENT_TYPE, encoder.format_type()))
            .body(buffer);
        ready(response)
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
impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MiddlewareService {
            metrics: self.metrics.clone(),
            service,
        }))
    }
}

/// Inner middleware to process requests on behalf of `MetricsMiddleware`.
pub struct MiddlewareService<S> {
    metrics: MetricsCollector,
    service: S,
}

impl<S, B> Service<ServiceRequest> for MiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = crate::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
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

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test::call_service;
    use actix_web::test::init_service;
    use actix_web::test::TestRequest;
    use actix_web::web;
    use actix_web::App;
    use prometheus::Registry;

    use super::MetricsExporter;

    #[actix_rt::test]
    async fn metrics_exporter_returns_200() {
        let registry = Registry::new();
        let exporter = MetricsExporter::with_registry(registry);
        let service = web::resource("/").to(exporter);
        let mut app = init_service(App::new().service(service)).await;
        let request = TestRequest::with_uri("https://server:1234/").to_request();
        let response = call_service(&mut app, request).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
