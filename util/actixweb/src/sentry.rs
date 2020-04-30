use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

use actix_service::Service;
use actix_service::Transform;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::Error;
use actix_web::HttpRequest;
use futures::future::ok;
use futures::future::Ready;
use sentry::internals::ScopeGuard;
use sentry::Hub;

/// Sentry `Hub` extension with actix-web methods.
pub trait ActixWebHubExt {
    /// Extract the `Hub` attached to a request.
    fn from_request(req: &HttpRequest) -> Arc<Hub>;

    /// Invoke a callback with the `Hub` attached to a request.
    fn run_from_request<F: FnOnce() -> R, R>(req: &HttpRequest, f: F) -> R;
}

impl ActixWebHubExt for Hub {
    fn from_request(req: &HttpRequest) -> Arc<Hub> {
        let exts = req.extensions();
        let context = exts.get::<SentryExtension>().unwrap();
        Arc::clone(&context.hub)
    }

    fn run_from_request<F: FnOnce() -> R, R>(req: &HttpRequest, f: F) -> R {
        let hub = Hub::from_request(req);
        Hub::run(hub, f)
    }
}

/// Actix Web middleware to integrate with sentry.
pub struct SentryMiddleware {
    current_hub: bool,
    report_code: u16,
}

impl SentryMiddleware {
    /// Create a `SentryMiddleware` capturing events if the return code is >= to `report_code`.
    ///
    /// The middleware will attach an `Hub` derived from `Hub::main()` to each request.
    pub fn new(report_code: u16) -> SentryMiddleware {
        SentryMiddleware {
            current_hub: false,
            report_code,
        }
    }

    /// Create a `SentryMiddleware` capturing events if the return code is >= to `report_code`.
    ///
    /// The middleware will attach an `Hub` derived from `Hub::current()` to each request.
    ///
    /// Generally deriving hubs from `Hub::main` is preferred to prevent differences across
    /// threads caused by changes performed to the thread local `Hub`.
    ///
    /// Using `Hub::current` is provided mainly for tests so the `Hub` created by
    /// `sentry::test::with_captured_events` is used and events can be inspected.
    pub fn with_current_hub(report_code: u16) -> SentryMiddleware {
        let mut middleware = SentryMiddleware::new(report_code);
        middleware.current_hub = true;
        middleware
    }
}

// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for SentryMiddleware
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
            current_hub: self.current_hub,
            report_code: self.report_code,
            service,
        })
    }
}

/// Inner middleware to process requests on behalf of `SentryMiddleware`.
pub struct MiddlewareService<S> {
    current_hub: bool,
    report_code: u16,
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

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        // Create a new Hub and push a scope for the request.
        let hub = if self.current_hub {
            Hub::current()
        } else {
            Hub::main()
        };
        let hub = Arc::new(Hub::new_from_top(hub));
        let scope = hub.push_scope();

        // Decorate events with request metadata.
        let request = sentry_request_context(&req);
        hub.configure_scope(move |scope| {
            scope.add_event_processor(Box::new(move |mut event| {
                event.request = Some(request.clone());
                Some(event)
            }));
        });

        // Add sentry context to the request extentions.
        let report_code = self.report_code;
        req.head_mut()
            .extensions_mut()
            .insert(SentryExtension { hub, scope });
        let response = self.service.call(req);
        Box::pin(async move {
            // Process sentry context and events if possible.
            let response = response.await?;
            let sentry: Option<SentryExtension> = response.request().extensions_mut().remove();
            if let Some(sentry) = sentry {
                // Send sentry event for 5xx/4xx responses.
                let code = response.response().status().as_u16();
                if code >= report_code {
                    let level = sentry_level_for_code(code);
                    let message = response
                        .response()
                        .error()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| format!("HTTP {}", response.response().status()));
                    sentry.hub.capture_event(sentry::protocol::Event {
                        level,
                        message: Some(message),
                        ..Default::default()
                    });
                }

                // Pop sentry context (scope before hub).
                drop(sentry.scope);
                drop(sentry.hub);
            }
            Ok(response)
        })
    }
}

/// Additional sentry context attached to a request.
struct SentryExtension {
    hub: Arc<Hub>,
    scope: ScopeGuard,
}

/// Convert an HTTP status code into a sentry event level.
fn sentry_level_for_code(code: u16) -> sentry::Level {
    match code {
        code if code < 400 => sentry::Level::Info,
        code if code < 500 => sentry::Level::Warning,
        _ => sentry::Level::Error,
    }
}

/// Convert request metadata into a sentry request object.
fn sentry_request_context(req: &ServiceRequest) -> sentry::protocol::Request {
    // Attempt to extract the full request URL.
    // Skip it if we are unable to convert it to sentry expected format.
    let url = req.match_info().get_ref().uri().to_string().parse().ok();
    let headers = req
        .headers()
        .iter()
        .map(|(key, value)| {
            let key = key.to_string();
            let value = value.to_str().unwrap_or("<bytes>").to_string();
            (key, value)
        })
        .collect();
    sentry::protocol::Request {
        headers,
        method: Some(req.method().to_string()),
        query_string: Some(req.query_string().to_string()),
        url,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use actix_web::test::call_service;
    use actix_web::test::init_service;
    use actix_web::test::TestRequest;
    use actix_web::web;
    use actix_web::App;
    use actix_web::Error;
    use actix_web::HttpResponse;
    use failure::err_msg;
    use futures::executor::block_on;
    use sentry::capture_message;
    use sentry::test::with_captured_events;
    use sentry::Hub;
    use sentry::Level;

    use super::ActixWebHubExt;
    use super::SentryMiddleware;

    async fn respond_500() -> Result<HttpResponse, Error> {
        Err(Error::from(err_msg("test")))
    }

    #[actix_rt::test]
    async fn capture_event() {
        let mut app = init_service(
            App::new()
                .wrap(SentryMiddleware::with_current_hub(500))
                .service(web::resource("/test").to(|req| {
                    Hub::run_from_request(&req, || {
                        capture_message("test", Level::Error);
                    });
                    HttpResponse::Ok()
                })),
        )
        .await;
        let request = TestRequest::with_uri("https://server:1234/test").to_request();
        let events = with_captured_events(|| {
            block_on(call_service(&mut app, request));
        });
        assert_eq!(events.len(), 1);
        let event = events.into_iter().next().unwrap();
        let request = event.request.unwrap();
        assert_eq!(request.method.unwrap(), "GET");
        assert_eq!(request.query_string.unwrap(), "");
        assert_eq!(request.url.unwrap().to_string(), "https://server:1234/test");
    }

    #[actix_rt::test]
    async fn capture_event_on_eror() {
        let mut app = init_service(
            App::new()
                .wrap(SentryMiddleware::with_current_hub(500))
                .service(web::resource("/test").to(respond_500)),
        )
        .await;
        let request = TestRequest::with_uri("https://server:1234/test").to_request();
        let events = with_captured_events(|| {
            block_on(call_service(&mut app, request));
        });
        assert_eq!(events.len(), 1);
        let event = events.into_iter().next().unwrap();
        assert_eq!(event.message.unwrap(), "test");
    }

    #[actix_rt::test]
    async fn capture_event_on_400() {
        let mut app = init_service(
            App::new()
                .wrap(SentryMiddleware::with_current_hub(400))
                .service(web::resource("/test").to(|| HttpResponse::BadRequest())),
        )
        .await;
        let request = TestRequest::with_uri("https://server:1234/test").to_request();
        let events = with_captured_events(|| {
            block_on(call_service(&mut app, request));
        });
        assert_eq!(events.len(), 1);
        let event = events.into_iter().next().unwrap();
        assert_eq!(event.message.unwrap(), "HTTP 400 Bad Request");
    }

    #[actix_rt::test]
    async fn capture_event_on_500() {
        let mut app = init_service(
            App::new()
                .wrap(SentryMiddleware::with_current_hub(500))
                .service(web::resource("/test").to(|| HttpResponse::InternalServerError())),
        )
        .await;
        let request = TestRequest::with_uri("https://server:1234/test").to_request();
        let events = with_captured_events(|| {
            block_on(call_service(&mut app, request));
        });
        assert_eq!(events.len(), 1);
        let event = events.into_iter().next().unwrap();
        assert_eq!(event.message.unwrap(), "HTTP 500 Internal Server Error");
    }

    #[actix_rt::test]
    async fn main_hub_misses_test_events() {
        let mut app = init_service(App::new().wrap(SentryMiddleware::new(500)).service(
            web::resource("/test").to(|req| {
                Hub::run_from_request(&req, || {
                    capture_message("test", Level::Error);
                });
                HttpResponse::Ok()
            }),
        ))
        .await;
        let request = TestRequest::with_uri("https://server:1234/test").to_request();
        let events = with_captured_events(|| {
            block_on(call_service(&mut app, request));
        });
        assert_eq!(events.len(), 0);
    }

    #[actix_rt::test]
    async fn skip_event_on_400() {
        let mut app = init_service(
            App::new()
                .wrap(SentryMiddleware::with_current_hub(401))
                .service(web::resource("/test").to(|| HttpResponse::BadRequest())),
        )
        .await;
        let request = TestRequest::with_uri("https://server:1234/test").to_request();
        let events = with_captured_events(|| {
            block_on(call_service(&mut app, request));
        });
        assert_eq!(events.len(), 0);
    }
}
