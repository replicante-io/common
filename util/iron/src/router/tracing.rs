use std::sync::Arc;

use iron::typemap::Key;
use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use opentracingrust::Span;
use opentracingrust::Tracer;
use router::Router;
use slog::Logger;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_tracing::carriers::iron::HeadersCarrier;

/// Attach a span to each request before passing it to the handler.
///
/// Useful to test any handler that expects a span to be attached to requests
/// and extracted with `request_span`.
#[cfg(feature = "with_test_support")]
pub fn mock_request_span<H: Handler>(tracer: Arc<Tracer>, handler: H) -> impl Handler {
    move |request: &mut Request| -> IronResult<Response> {
        let span = tracer.span("mock_request_span");
        request.extensions.insert::<IronSpan>(span);
        let response = handler.handle(request);
        if let Some(span) = request.extensions.remove::<IronSpan>() {
            let _ = span.finish();
        }
        response
    }
}

/// Access the request's tracing span.
pub fn request_span<'a>(req: &'a mut Request) -> &'a mut Span {
    req.extensions
        .get_mut::<IronSpan>()
        .expect("request is missing the IronSpan extention")
}

/// Private Iron extention key to attach spans to requests.
struct IronSpan;

impl Key for IronSpan {
    type Value = Span;
}

/// Iron handler that decorates another handler with a trace span.
pub struct TracedHandler<H: Handler> {
    glob: String,
    handler: H,
    logger: Logger,
    tracer: Arc<Tracer>,
}

impl<H: Handler> TracedHandler<H> {
    pub fn new(tracer: Arc<Tracer>, glob: String, logger: Logger, handler: H) -> TracedHandler<H> {
        TracedHandler {
            glob,
            handler,
            logger,
            tracer,
        }
    }
}

impl<H: Handler> Handler for TracedHandler<H> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let mut span = self.tracer.span(&self.glob);
        match HeadersCarrier::extract(&mut req.headers, &self.tracer) {
            Ok(Some(context)) => span.child_of(context),
            Ok(None) => (),
            Err(error) => {
                let error = failure::SyncFailure::new(error);
                capture_fail!(
                    &error,
                    self.logger,
                    "Unable to extract trace context from request headers";
                    failure_info(&error),
                );
            }
        };
        if let Some(params) = req.extensions.get::<Router>() {
            for (param, value) in params.iter() {
                span.tag(&format!("http.route.param.{}", param), value);
            }
        }
        req.extensions.insert::<IronSpan>(span);
        let mut response = self.handler.handle(req);
        if let Some(span) = req.extensions.remove::<IronSpan>() {
            let headers = match response {
                Ok(ref mut response) => &mut response.headers,
                Err(ref mut error) => &mut error.response.headers,
            };
            if let Err(error) = HeadersCarrier::inject(span.context(), headers, &self.tracer) {
                let error = failure::SyncFailure::new(error);
                capture_fail!(
                    &error,
                    self.logger,
                    "Failed to inject trace context into response headers";
                    failure_info(&error),
                );
            }
            if let Err(error) = span.finish() {
                let error = failure::SyncFailure::new(error);
                capture_fail!(
                    &error,
                    self.logger,
                    "Failed to finish request tracing span";
                    failure_info(&error),
                );
            }
        }
        response
    }
}
