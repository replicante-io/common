use std::task::Context;
use std::task::Poll;

use actix_service::Service;
use actix_service::Transform;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::Error;
use futures::future::ok;
use futures::future::Ready;
use slog::info;
use slog::Logger;

/// Actix Web middleware to log requests.
pub struct LoggingMiddleware {
    logger: Logger,
}

impl LoggingMiddleware {
    pub fn new(logger: Logger) -> LoggingMiddleware {
        LoggingMiddleware { logger }
    }
}

// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for LoggingMiddleware
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
            service,
        })
    }
}

/// Inner middleware to process requests on behalf of `LoggingMiddleware`.
pub struct MiddlewareService<S> {
    logger: Logger,
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
        let logger = self.logger.clone();
        let response = self.service.call(req);
        Box::pin(async move {
            let response = response.await?;
            let method = response.request().method();
            let path = response.request().path();
            let status = response.response().status();
            let error = status.is_server_error() || status.is_client_error();
            info!(
                logger,
                "Request handled";
                "success" => !error,
                "method" => %method,
                "path" => path,
                "status" => %status,
            );
            Ok(response)
        })
    }
}
