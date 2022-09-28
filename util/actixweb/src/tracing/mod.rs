use std::future::ready;
use std::future::Ready;
use std::result::Result;
use std::sync::Arc;

use actix_web::dev::forward_ready;
use actix_web::dev::Service;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::dev::Transform;
use actix_web::Error;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use opentracingrust::Span;
use opentracingrust::Tracer;
use slog::error;
use slog::Logger;

mod carriers;

pub use self::carriers::HeadersCarrier;

/// Access the request's tracing span.
pub fn with_request_span<B, R>(request: &mut HttpRequest, block: B) -> R
where
    B: FnOnce(Option<&mut Span>) -> R,
{
    let mut exts = request.extensions_mut();
    let span = exts.get_mut::<Span>();
    block(span)
}

/// Actix Web middleware to inject an `opentracingrust::Span` on each request.
pub struct TracingMiddleware {
    logger: Logger,
    name: Option<String>,
    tracer: Arc<Tracer>,
}

impl TracingMiddleware {
    /// Inject spans using the request path as then name.
    pub fn new(logger: Logger, tracer: Arc<Tracer>) -> TracingMiddleware {
        TracingMiddleware {
            logger,
            name: None,
            tracer,
        }
    }

    /// Inject spans using the given name.
    pub fn with_name<S>(logger: Logger, tracer: Arc<Tracer>, name: S) -> TracingMiddleware
    where
        S: Into<String>,
    {
        let name = Some(name.into());
        TracingMiddleware {
            logger,
            name,
            tracer,
        }
    }
}

// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for TracingMiddleware
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
            logger: self.logger.clone(),
            name: self.name.clone(),
            service,
            tracer: Arc::clone(&self.tracer),
        }))
    }
}

/// Inner middleware to process requests on behalf of `TracingMiddleware`.
pub struct MiddlewareService<S> {
    logger: Logger,
    name: Option<String>,
    service: S,
    tracer: Arc<Tracer>,
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

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let logger = self.logger.clone();
        let name = match self.name.as_ref() {
            None => req.path(),
            Some(name) => name.as_str(),
        };
        let mut span = self.tracer.span(name);

        // Extend the span with a parent and some request attributes.
        match HeadersCarrier::extract(req.headers_mut(), &self.tracer) {
            Ok(Some(context)) => span.child_of(context),
            Ok(None) => (),
            Err(error) => {
                let error = anyhow::anyhow!(error);
                sentry::integrations::anyhow::capture_anyhow(&error);
                error!(
                    logger,
                    "Unable to extract trace context from request headers";
                    "error" => %error,
                );
            }
        };
        span.tag("http.route.method", req.method().as_str());
        span.tag("http.route.uri", req.uri().to_string());
        for (param, value) in req.match_info().iter() {
            span.tag(&format!("http.route.param.{}", param), value);
        }

        // Send the request and handle the span on response.
        let tracer = self.tracer.clone();
        req.extensions_mut().insert(span);
        let response = self.service.call(req);
        Box::pin(async move {
            let mut response = response.await?;
            let span: Option<Span> = response.request().extensions_mut().remove();
            if let Some(span) = span {
                let result = HeadersCarrier::inject(
                    span.context(),
                    response.response_mut().headers_mut(),
                    &tracer,
                );
                if let Err(error) = result {
                    let error = anyhow::anyhow!(error);
                    sentry::integrations::anyhow::capture_anyhow(&error);
                    error!(
                        logger,
                        "Failed to inject trace context into response headers";
                        "error" => %error,
                    );
                }

                if let Err(error) = span.finish() {
                    let error = anyhow::anyhow!(error.to_string());
                    sentry::integrations::anyhow::capture_anyhow(&error);
                    error!(
                        logger,
                        "Failed to finish request tracing span";
                        "error" => %error,
                    );
                }
            }
            Ok(response)
        })
    }
}
