use std::collections::BTreeMap;

use iron::AfterMiddleware;
use iron::IronError;
use iron::IronResult;
use iron::Request;
use iron::Response;

use sentry::capture_event;
use sentry::protocol::Event as SentryEvent;
use sentry::protocol::Request as SentryRequest;

/// Convert an HTTP status code into a severity level.
fn event_level(code: u16) -> sentry::Level {
    match code {
        code if code < 400 => sentry::Level::Info,
        code if code < 500 => sentry::Level::Warning,
        _ => sentry::Level::Error,
    }
}

/// Generate a sentry request context from an iron request.
fn request_context(request: &Request) -> SentryRequest {
    let mut headers = BTreeMap::new();
    for header in request.headers.iter() {
        headers.insert(header.name().to_string(), header.value_string());
    }
    SentryRequest {
        headers,
        method: Some(request.method.to_string()),
        url: Some(request.url.clone().into()),
        ..Default::default()
    }
}

/// Iron middleware that sends non-success responses to sentry.
///
/// * Responses with a status < 400 are ignored (2xx & 3xx).
/// * Responses with a status < 500 are warnings (4xx).
/// * Responses with all other codes are errors (5xx).
///
/// It is worth noting that handlers that fail with an IronError that carries
/// a successful response (status < 400) will also not be logged.
pub struct SentryMiddlewere {
    status_above: u16,
}

impl SentryMiddlewere {
    pub fn new(status_above: u16) -> SentryMiddlewere {
        SentryMiddlewere { status_above }
    }
}

impl AfterMiddleware for SentryMiddlewere {
    fn after(&self, request: &mut Request, response: Response) -> IronResult<Response> {
        let code = response
            .status
            .expect("response must have a status")
            .to_u16();
        // Skip success responses.
        if code < self.status_above {
            return Ok(response);
        }

        // Capture an event.
        let level = event_level(code);
        let context = request_context(request);
        capture_event(SentryEvent {
            level,
            request: Some(context),
            ..Default::default()
        });

        // Move on to the next handler.
        Ok(response)
    }

    fn catch(&self, request: &mut Request, error: IronError) -> IronResult<Response> {
        let code = error
            .response
            .status
            .expect("response must have a status")
            .to_u16();
        // Skip success responses.
        if code < 400 {
            return Err(error);
        }

        // Capture an event.
        let level = event_level(code);
        let context = request_context(request);
        capture_event(SentryEvent {
            level,
            request: Some(context),
            ..Default::default()
        });

        // Move on to the next handler.
        Err(error)
    }
}

impl Default for SentryMiddlewere {
    fn default() -> SentryMiddlewere {
        SentryMiddlewere::new(400)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use iron::headers::Origin;
    use iron::method;
    use iron::status;
    use iron::Chain;
    use iron::Headers;
    use iron::IronError;
    use iron::Request;
    use iron::Response;
    use iron_test::request;

    use sentry::test::with_captured_events;

    use super::SentryMiddlewere;

    #[derive(Debug)]
    struct MockError;
    impl ::std::fmt::Display for MockError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            ::std::fmt::Debug::fmt(self, f)
        }
    }
    impl ::std::error::Error for MockError {
        fn description(&self) -> &str {
            "MockError"
        }
    }

    fn make_chain(
        middleware: SentryMiddlewere,
        code: status::Status,
        message: &'static str,
    ) -> Chain {
        let mut chain = Chain::new(move |req: &mut Request| match req.method {
            method::Get => Ok(Response::with((code, message))),
            _ => {
                let response = Response::with((code, message));
                let err = IronError {
                    response,
                    error: Box::new(MockError),
                };
                Err(err)
            }
        });
        chain.link_after(middleware);
        chain
    }

    #[test]
    fn after_200() {
        let middleware = SentryMiddlewere::default();
        let chain = make_chain(middleware, status::Ok, "OK Response");
        let headers = Headers::new();
        let events = with_captured_events(|| {
            request::get("http://host:16016/some/endpoint", headers, &chain).unwrap();
        });
        assert_eq!(0, events.len());
    }

    #[test]
    fn after_404() {
        let middleware = SentryMiddlewere::default();
        let chain = make_chain(middleware, status::NotFound, "NaN");
        let mut headers = Headers::new();
        headers.set(Origin::new("http", "host", Some(16916)));
        let mut events = with_captured_events(|| {
            request::get("http://host:16016/some/endpoint", headers, &chain).unwrap();
        });
        let event = events.remove(0);
        assert_eq!(sentry::Level::Warning, event.level);
        let context = event.request.expect("no request context found");
        assert_eq!("/some/endpoint", context.url.unwrap().path());
        assert_eq!("GET", context.method.unwrap());
        assert_eq!(context.headers, {
            let mut headers = BTreeMap::new();
            headers.insert("Content-Length".into(), "0".into());
            headers.insert("Origin".into(), "http://host:16916".into());
            headers.insert("User-Agent".into(), "iron-test".into());
            headers
        });
    }

    #[test]
    fn after_500() {
        let middleware = SentryMiddlewere::default();
        let chain = make_chain(middleware, status::InternalServerError, "");
        let headers = Headers::new();
        let mut events = with_captured_events(|| {
            request::get("http://host:16016/some/endpoint", headers, &chain).unwrap();
        });
        let event = events.remove(0);
        assert_eq!(sentry::Level::Error, event.level);
    }

    #[test]
    fn catch_200() {
        let middleware = SentryMiddlewere::default();
        let chain = make_chain(middleware, status::Ok, "NaN");
        let headers = Headers::new();
        let events = with_captured_events(|| {
            let err = request::put("http://host:16016/some/endpoint", headers, "", &chain);
            assert_eq!(true, err.is_err());
        });
        assert_eq!(0, events.len());
    }

    #[test]
    fn catch_404() {
        let middleware = SentryMiddlewere::default();
        let chain = make_chain(middleware, status::NotFound, "NaN");
        let mut headers = Headers::new();
        headers.set(Origin::new("http", "host", Some(16916)));
        let mut events = with_captured_events(|| {
            let err = request::put("http://host:16016/some/endpoint", headers, "", &chain);
            assert_eq!(true, err.is_err());
        });
        let event = events.remove(0);
        assert_eq!(sentry::Level::Warning, event.level);
    }

    #[test]
    fn catch_500() {
        let middleware = SentryMiddlewere::default();
        let chain = make_chain(middleware, status::InternalServerError, "");
        let headers = Headers::new();
        let mut events = with_captured_events(|| {
            let err = request::put("http://host:16016/some/endpoint", headers, "", &chain);
            assert_eq!(true, err.is_err());
        });
        let event = events.remove(0);
        assert_eq!(sentry::Level::Error, event.level);
    }
}
