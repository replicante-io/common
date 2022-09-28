use std::future::ready;
use std::future::Ready;

use actix_web::dev::forward_ready;
use actix_web::dev::Service;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::dev::Transform;
use actix_web::Error;
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
impl<S, B> Transform<S, ServiceRequest> for LoggingMiddleware
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
            service,
        }))
    }
}

/// Inner middleware to process requests on behalf of `LoggingMiddleware`.
pub struct MiddlewareService<S> {
    logger: Logger,
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
