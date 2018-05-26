use iron::prelude::*;
use iron::Handler;
use iron::headers::ContentType;
use iron::mime::Mime;
use iron::status;

use prometheus::Encoder;
use prometheus::Registry;
use prometheus::TextEncoder;


/// An Iron Handler that exposes prometheus metrics in text format.
pub struct MetricsHandler {
    content_type: ContentType,
    registry: Registry,
}

impl MetricsHandler {
    pub fn new(registry: Registry) -> MetricsHandler {
        let encoder = TextEncoder::new();
        let content_type = encoder.format_type().parse::<Mime>().unwrap();
        MetricsHandler {
            content_type: ContentType(content_type),
            registry,
        }
    }
}

impl Handler for MetricsHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        let metric_familys = self.registry.gather();
        encoder.encode(&metric_familys, &mut buffer).unwrap();

        let mut response = Response::new();
        response.headers.set(self.content_type.clone());
        response.set_mut(buffer).set_mut(status::Ok);
        Ok(response)
    }
}


#[cfg(test)]
mod tests {
    use iron::IronResult;
    use iron::Headers;
    use iron::Response;
    use iron_test::request;
    use iron_test::response;

    use prometheus::Counter;
    use prometheus::Registry;

    use super::MetricsHandler;

    fn request_get(registry: Registry) -> IronResult<Response> {
        let handler = MetricsHandler::new(registry);
        request::get(
            "http://localhost:3000/api/v1/metrics",
            Headers::new(), &handler
        )
    }

    #[test]
    fn metrics_content_header() {
        let registry = Registry::new();
        let response = request_get(registry).unwrap();
        let value = response.headers.get_raw("Content-Type").unwrap();
        let value = String::from_utf8(value[0].clone()).unwrap();
        assert_eq!(value, "text/plain; version=0.0.4");
    }

    #[test]
    fn metrics_data() {
        let count = Counter::new("name", "desc").unwrap();
        count.inc_by(2.0);

        let registry = Registry::new();
        registry.register(Box::new(count)).unwrap();

        let response = request_get(registry).unwrap();
        let body = response::extract_body_to_bytes(response);
        let body = String::from_utf8(body).unwrap();
        assert_eq!(body, "# HELP name desc\n# TYPE name counter\nname 2\n");
    }
}
