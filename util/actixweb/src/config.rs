use std::collections::HashMap;
use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web::ServiceConfig;
use actix_web::Scope;

/// Type alias for AppConfig functions to improve code readability.
type AppConfigFn<T> = Arc<dyn Fn(&mut AppConfigContext<T>) + Send + Sync>;

/// On-demand Actix Web `App` configuration with functions and closures.
#[derive(Clone)]
pub struct AppConfig<T> {
    configs: Vec<AppConfigFn<T>>,
}

impl<T> AppConfig<T> {
    /// Run all the register handles to configure the given app.
    pub fn configure(&mut self, app: &mut ServiceConfig, context: &T) {
        // Build the AppConfigContext container to pass to configuration callbacks.
        let mut scopes = AppConfigScopes::default();
        let mut config_context = AppConfigContext {
            app,
            context,
            scopes: &mut scopes,
        };

        // Configure the service and its scopes.
        for config in &self.configs {
            config(&mut config_context);
        }

        // Configure the application with all of its scopes.
        scopes.configure(app);
    }

    /// Register an app configuration function to be run later.
    pub fn register<F>(&mut self, config: F)
    where
        F: Fn(&mut AppConfigContext<T>) + 'static + Send + Sync,
    {
        self.configs.push(Arc::new(config));
    }
}

impl<T> Default for AppConfig<T> {
    fn default() -> Self {
        let configs = Vec::new();
        AppConfig { configs }
    }
}

/// Configuration context provided by `AppConfig` to callbacks.
///
/// Can be used to access:
///
///   * The `actix_web::web::ServiceConfig` instance being configured.
///   * The user provided context.
///   * Additional configuration methods such as `AppConfigContext::scoped_service`.
pub struct AppConfigContext<'context, T> {
    pub app: &'context mut ServiceConfig,
    pub context: &'context T,
    scopes: &'context mut AppConfigScopes,
}

impl<'context, T> AppConfigContext<'context, T> {
    /// Register an `actix_web::dev::HttpServiceFactory` into a shared `actix_web::Scope`.
    ///
    /// This method is intended for use when many services share a known set of prefixes.
    /// Instead of generating resources or services that repeat the full prefix every time
    /// it is possible to group all these services under one or more `actix_web::Scope`s
    /// that manage the `path` prefix for all services once.
    ///
    /// The `actix_web::Scope`s are managed internally so they can be used by all
    /// `AppConfigContext::scoped_service` invokations that use the same `path`.
    /// Scopes are created the first time they are needed and are not directly accessible.
    ///
    /// # Scopes order and prefixes
    /// Actix Web route matching documentation: https://actix.rs/docs/url-dispatch/
    ///
    /// As stated there, routes are matched in order of registstration in their parent Scope/App.
    /// To ensure order is consistent across application restarts and order of callbacks invokation:
    ///
    ///   * Scopes are sorted by `path`, alphabetically.
    ///   * Scopes are reversed to support prefixes.
    ///
    /// Prefixes are scopes with a `path` starting with the `path` used by another scope:
    /// For example:
    ///
    ///   * `/api`.
    ///   * `/api/v1` (prefixed by `/api`).
    ///   * `/api/v2` (prefixed by `/api`).
    ///
    /// must be registered in reverse order or the `/api` scope would match everything
    /// and all requests to paths under `/api/v1` or `/api/v2` would fail to route correctly.
    ///
    ///
    /// # Panic
    /// To avoid paths not matching due to how `actix_web::Scope`s are visited
    /// path variables are not allowed in the scoped `path`.
    ///
    /// This prevents variables from matching "over" other paths or prefixes.
    pub fn scoped_service<F>(&mut self, path: &str, factory: F)
    where
        F: HttpServiceFactory + 'static,
    {
        if path.contains('{') {
            panic!("path variables are not suppored in scoped_service");
        }
        let (key, scope) = match self.scopes.map.remove_entry(path) {
            Some(entry) => entry,
            None => {
                let key = path.to_string();
                let scope = actix_web::web::scope(path);
                (key, scope)
            }
        };
        let scope = scope.service(factory);
        self.scopes.map.insert(key, scope);
    }
}

/// Container for `actix_web::Scope`s shared among configuration callbacks.
#[derive(Default)]
struct AppConfigScopes {
    map: HashMap<String, Scope>,
}

impl AppConfigScopes {
    /// Consume this object and configure all known scopes as services.
    fn configure(self, app: &mut ServiceConfig) {
        let mut scopes: Vec<(String, Scope)> = self.map.into_iter().collect();
        scopes.sort_by(|a, b| a.0.cmp(&b.0));
        scopes.reverse();
        for (_, scope) in scopes.into_iter() {
            app.service(scope);
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::test::call_service;
    use actix_web::test::init_service;
    use actix_web::test::TestRequest;
    use actix_web::web;
    use actix_web::App;
    use actix_web::HttpResponse;
    use actix_web::Responder;

    use super::AppConfig;

    async fn static_200() -> impl Responder {
        "static 200".to_string()
    }

    async fn static_400() -> impl Responder {
        HttpResponse::BadRequest()
    }

    async fn static_500() -> impl Responder {
        HttpResponse::InternalServerError()
    }

    #[actix_rt::test]
    async fn configure_app() {
        let mut conf = AppConfig::default();
        conf.register(|conf| {
            let resource = web::resource("/res1").route(web::get().to(static_200));
            conf.app.service(resource);
        });
        conf.register(|conf| {
            let resource = web::resource("/res2").route(web::get().to(static_200));
            conf.app.service(resource);
        });
        let resource = web::resource("/res3").route(web::get().to(static_200));
        let app = App::new()
            .configure(|app| conf.configure(app, &()))
            .service(resource);
        let mut app = init_service(app).await;

        let req = TestRequest::get().uri("/res1").to_request();
        let res = call_service(&mut app, req).await;
        assert_eq!(res.status().as_u16(), 200);
        let req = TestRequest::get().uri("/res2").to_request();
        let res = call_service(&mut app, req).await;
        assert_eq!(res.status().as_u16(), 200);
        let req = TestRequest::get().uri("/res3").to_request();
        let res = call_service(&mut app, req).await;
        assert_eq!(res.status().as_u16(), 200);
    }

    #[actix_rt::test]
    async fn scopes() {
        let mut conf = AppConfig::default();
        conf.register(|conf| {
            let resource = web::resource("/res1").route(web::get().to(static_200));
            conf.scoped_service("/scope1", resource);
        });
        conf.register(|conf| {
            let resource = web::resource("/res2").route(web::get().to(static_200));
            conf.scoped_service("/scope1", resource);
        });
        conf.register(|conf| {
            let resource = web::resource("/res3").route(web::get().to(static_200));
            conf.scoped_service("/scope2", resource);
        });
        let app = App::new().configure(|app| conf.configure(app, &()));
        let mut app = init_service(app).await;

        let req = TestRequest::get().uri("/scope1/res1").to_request();
        let res = call_service(&mut app, req).await;
        assert_eq!(res.status().as_u16(), 200);
        let req = TestRequest::get().uri("/scope1/res2").to_request();
        let res = call_service(&mut app, req).await;
        assert_eq!(res.status().as_u16(), 200);
        let req = TestRequest::get().uri("/scope2/res3").to_request();
        let res = call_service(&mut app, req).await;
        assert_eq!(res.status().as_u16(), 200);
    }

    #[actix_rt::test]
    async fn scopes_with_overlapping_prefix() {
        let mut conf = AppConfig::default();
        conf.register(|conf| {
            let resource = web::resource("/res").route(web::get().to(static_200));
            conf.scoped_service("/scope/prefix", resource);
        });
        conf.register(|conf| {
            let resource = web::resource("/res").route(web::get().to(static_400));
            conf.scoped_service("/scope/prefix/overlap", resource);
        });
        conf.register(|conf| {
            let resource = web::resource("/{variable}").route(web::get().to(static_500));
            conf.scoped_service("/scope/prefix/overlap", resource);
        });
        let app = App::new().configure(|app| conf.configure(app, &()));
        let mut app = init_service(app).await;

        let req = TestRequest::get().uri("/scope/prefix/res").to_request();
        let res = call_service(&mut app, req).await;
        assert_eq!(res.status().as_u16(), 200);
        let req = TestRequest::get()
            .uri("/scope/prefix/overlap/res")
            .to_request();
        let res = call_service(&mut app, req).await;
        assert_eq!(res.status().as_u16(), 400);
        let req = TestRequest::get()
            .uri("/scope/prefix/overlap/variable")
            .to_request();
        let res = call_service(&mut app, req).await;
        assert_eq!(res.status().as_u16(), 500);
    }

    #[test]
    #[should_panic(expected = "path variables are not suppored in scoped_service")]
    fn scopes_should_not_allow_variable() {
        let mut conf = AppConfig::default();
        conf.register(|conf| {
            let resource = web::resource("/res").route(web::get().to(static_200));
            conf.scoped_service("/path/with/{variable}", resource);
        });
        App::new().configure(|app| conf.configure(app, &()));
    }
}
