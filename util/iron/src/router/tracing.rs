use iron::typemap::Key;
use iron::Request;
use opentracingrust::Span;

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
