use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

use actix_service::Service;
use actix_service::Transform;
use actix_web::dev::Extensions;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::Error;
use actix_web::HttpRequest;
use futures::future::ok;
use futures::future::Ready;
use opentracingrust::Span;
use opentracingrust::Tracer;
use slog::Logger;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

mod carriers;

pub use self::carriers::HeadersCarrier;

/// Access the request's tracing span.
#[deprecated(
    since = "0.2.0",
    note = "use replicante_util_actixweb::with_request_span"
)]
pub fn request_span(req: &mut Extensions) -> &mut Span {
    req.get_mut::<Span>()
        .expect("request is missing Span extention")
}

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
impl<S, B> Transform<S> for TracingMiddleware
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
            logger: self.logger.clone(),
            name: self.name.clone(),
            service,
            tracer: Arc::clone(&self.tracer),
        })
    }
}

/// Inner middleware to process requests on behalf of `TracingMiddleware`.
pub struct MiddlewareService<S> {
    logger: Logger,
    name: Option<String>,
    service: S,
    tracer: Arc<Tracer>,
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

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let logger = self.logger.clone();
        let name = match self.name.as_ref() {
            None => req.path(),
            Some(name) => name.as_str(),
        };
        let mut span = self.tracer.span(name);

        // Extend the span with a parent and some request attributes.
        match HeadersCarrier::extract(&mut req.headers_mut(), &self.tracer) {
            Ok(Some(context)) => span.child_of(context),
            Ok(None) => (),
            Err(error) => {
                capture_fail!(
                    &error,
                    logger,
                    "Unable to extract trace context from request headers";
                    failure_info(&error),
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
        req.head_mut().extensions_mut().insert(span);
        let response = self.service.call(req);
        Box::pin(async move {
            let mut response = response.await?;
            let span: Option<Span> = response.request().extensions_mut().remove();
            if let Some(span) = span {
                let result = HeadersCarrier::inject(
                    span.context(),
                    &mut response.response_mut().headers_mut(),
                    &tracer,
                );
                if let Err(error) = result {
                    capture_fail!(
                        &error,
                        logger,
                        "Failed to inject trace context into response headers";
                        failure_info(&error),
                    );
                }

                if let Err(error) = span.finish() {
                    let error = failure::SyncFailure::new(error);
                    capture_fail!(
                        &error,
                        logger,
                        "Failed to finish request tracing span";
                        failure_info(&error),
                    );
                }
            }
            Ok(response)
        })
    }
}
