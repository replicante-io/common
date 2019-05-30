use std::collections::HashMap;
use std::sync::Arc;

use iron::method;
use iron::Chain;
use iron::Handler;
use opentracingrust::Tracer;
use slog::Logger;

mod tracing;

pub use self::tracing::request_span;
use self::tracing::TracedHandler;

/// A builder object for an `iron-router` [`Router`].
///
/// [`Router`]: router/struct.Router.html
pub struct Router {
    flags: HashMap<&'static str, bool>,
    inner: ::iron_router::Router,
    logger: Logger,
    tracer: Option<Arc<Tracer>>,
}

impl Router {
    /// Wraps a new [`Router`] for manipulation.
    ///
    /// [`Router`]: router/struct.Router.html
    pub fn new<T>(flags: HashMap<&'static str, bool>, logger: Logger, tracer: T) -> Router
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let inner = ::iron_router::Router::new();
        let tracer = tracer.into();
        Router {
            flags,
            inner,
            logger,
            tracer,
        }
    }

    /// Convert this `Router` into an iron [`Chain`].
    ///
    /// [`Chain`]: iron/middleware/struct.Chain.html
    pub fn build(self) -> Chain {
        Chain::new(self.inner)
    }

    /// Returns a "veiw" on the router to register endpoints under a specific root.
    pub fn for_root<R: RootDescriptor>(&mut self, root: &R) -> RootedRouter {
        let enabled = root.enabled(&self.flags);
        let logger = &self.logger;
        let prefix = root.prefix();
        let router = &mut self.inner;
        let tracer = if root.trace() {
            self.tracer.clone()
        } else {
            None
        };
        RootedRouter {
            enabled,
            logger,
            prefix,
            router,
            tracer,
        }
    }
}

/// Interface for types that describe endpoint roots.
///
/// Useful to codify in a sensible way which possible paths are exposed by an API.
///
/// ## Example
/// ```
///# use std::collections::HashMap;
/// use replicante_util_iron::RootDescriptor;
///
/// enum APIVersion {
///     V1,
///     V2,
///     V3,
/// }
///
/// impl RootDescriptor for APIVersion {
///     fn enabled(&self, flags: &HashMap<&'static str, bool>) -> bool {
///         match self {
///             APIVersion::V1 => match flags.get("v1") {
///                 Some(flag) => *flag,
///                 None => match flags.get("legacy") {
///                     Some(flag) => *flag,
///                     None => true,
///                 },
///             },
///             APIVersion::V2 => match flags.get("v2") {
///                 Some(flag) => *flag,
///                 None => match flags.get("legacy") {
///                     Some(flag) => *flag,
///                     None => true,
///                 },
///             },
///             APIVersion::V3 => true,
///         }
///     }
///
///     fn prefix(&self) -> &'static str {
///         match self {
///             APIVersion::V1 => "/api/v1",
///             APIVersion::V2 => "/some/other/path",
///             APIVersion::V3 => "/v3",
///         }
///     }
/// }
/// ```
pub trait RootDescriptor {
    /// Check if a root should be enabled according to the given flags.
    fn enabled(&self, flags: &HashMap<&'static str, bool>) -> bool;

    /// Return the URI prefix for a root.
    fn prefix(&self) -> &'static str;

    /// Trace requests to a root, if tracing is enabled.
    ///
    /// Tracing of roots is on by default but can be turned off for low-value and
    /// high-rate roots (like introspection or debugging) by returning `false`.
    fn trace(&self) -> bool {
        true
    }
}

/// Specialised router to mount endpoints under a fixed root.
///
/// The root's prefix is automatically prepended to the URI handlers are
/// registered with as well as the the Iron `::router::Router` id.
pub struct RootedRouter<'a> {
    enabled: bool,
    logger: &'a Logger,
    prefix: &'static str,
    router: &'a mut ::iron_router::Router,
    tracer: Option<Arc<Tracer>>,
}

impl<'a> RootedRouter<'a> {
    /// Like route, but specialized to the `Get` method.
    pub fn get<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut RootedRouter<'a> {
        self.route(method::Get, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut RootedRouter<'a> {
        self.route(method::Post, glob, handler, route_id)
    }

    /// Wrapper for [`Router::route`] with additional features.
    ///
    /// [`Router::route`]: router/struct.Router.html#method.route
    pub fn route<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        method: method::Method,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut RootedRouter<'a> {
        if !self.enabled {
            return self;
        }
        let glob = self.prefix.to_string() + glob.as_ref();
        let route_id = self.prefix.to_string() + route_id.as_ref();
        match self.tracer.clone() {
            None => self.router.route(method, glob, handler, route_id),
            Some(tracer) => {
                let handler =
                    TracedHandler::new(tracer, glob.clone(), self.logger.clone(), handler);
                self.router.route(method, glob, handler, route_id)
            }
        };
        self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use iron::method;
    use iron::status;
    use iron::Headers;
    use iron::IronResult;
    use iron::Request;
    use iron::Response;
    use iron_test::request;
    use iron_test::response;
    use slog::o;
    use slog::Discard;
    use slog::Logger;

    use super::RootDescriptor;
    use super::Router;

    enum Roots {
        R1,
        R2,
        R3,
    }

    impl RootDescriptor for Roots {
        fn enabled(&self, flags: &HashMap<&'static str, bool>) -> bool {
            match self {
                Roots::R1 => match flags.get("r1") {
                    Some(flag) => *flag,
                    None => match flags.get("test") {
                        Some(flag) => *flag,
                        None => true,
                    },
                },
                Roots::R2 => match flags.get("r2") {
                    Some(flag) => *flag,
                    None => match flags.get("test") {
                        Some(flag) => *flag,
                        None => true,
                    },
                },
                Roots::R3 => true,
            }
        }

        fn prefix(&self) -> &'static str {
            match self {
                Roots::R1 => "/api/root1",
                Roots::R2 => "/api/root2",
                Roots::R3 => "/api/root3",
            }
        }
    }

    fn mock_get(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "GET")))
    }

    fn mock_post(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "POST")))
    }

    fn mock_put(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "PUT")))
    }

    #[test]
    fn attach_get() {
        let logger = Logger::root(Discard, o!());
        let mut router = Router::new(HashMap::new(), logger, None);
        {
            let mut root = router.for_root(&Roots::R1);
            root.get("", &mock_get, "");
            root.get("/subtree", &mock_get, "/subtree");
        }
        let router = router.build();
        let response =
            request::get("http://host:16016/api/root1", Headers::new(), &router).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "GET");
        let response = request::get(
            "http://host:16016/api/root1/subtree",
            Headers::new(),
            &router,
        )
        .unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "GET");
    }

    #[test]
    fn attach_post() {
        let logger = Logger::root(Discard, o!());
        let mut router = Router::new(HashMap::new(), logger, None);
        {
            let mut root = router.for_root(&Roots::R2);
            root.post("", &mock_post, "");
        }
        let router = router.build();
        let response =
            request::post("http://host:16016/api/root2", Headers::new(), "", &router).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "POST");
    }

    #[test]
    fn attach_route() {
        let logger = Logger::root(Discard, o!());
        let mut router = Router::new(HashMap::new(), logger, None);
        {
            let mut root = router.for_root(&Roots::R3);
            root.route(method::Put, "", &mock_put, "");
        }
        let router = router.build();
        let response =
            request::put("http://host:16016/api/root3", Headers::new(), "", &router).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "PUT");
    }

    #[test]
    fn filtered_routes() {
        let mut flags = HashMap::new();
        flags.insert("r1", false);
        flags.insert("r2", true);
        let logger = Logger::root(Discard, o!());
        let mut router = Router::new(flags, logger, None);
        {
            let mut root = router.for_root(&Roots::R1);
            root.get("/test", &mock_get, "/test");
        }
        {
            let mut root = router.for_root(&Roots::R2);
            root.get("/test", &mock_get, "/test");
        }
        {
            let mut root = router.for_root(&Roots::R3);
            root.get("/test", &mock_get, "/test");
        }
        let router = router.build();
        let response = request::get("http://host:16016/api/root1/test", Headers::new(), &router);
        assert_eq!(true, response.is_err());
        let response =
            request::get("http://host:16016/api/root2/test", Headers::new(), &router).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "GET");
        let response =
            request::get("http://host:16016/api/root3/test", Headers::new(), &router).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "GET");
    }

    #[test]
    fn filtered_routes_by_group() {
        let mut flags = HashMap::new();
        flags.insert("test", false);
        let logger = Logger::root(Discard, o!());
        let mut router = Router::new(flags, logger, None);
        {
            let mut root = router.for_root(&Roots::R1);
            root.get("/test", &mock_get, "/test");
        }
        {
            let mut root = router.for_root(&Roots::R2);
            root.get("/test", &mock_get, "/test");
        }
        {
            let mut root = router.for_root(&Roots::R3);
            root.get("/test", &mock_get, "/test");
        }
        let router = router.build();
        let response = request::get("http://host:16016/api/root1/test", Headers::new(), &router);
        assert_eq!(true, response.is_err());
        let response = request::get("http://host:16016/api/root2/test", Headers::new(), &router);
        assert_eq!(true, response.is_err());
        let response =
            request::get("http://host:16016/api/root3/test", Headers::new(), &router).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "GET");
    }
}
