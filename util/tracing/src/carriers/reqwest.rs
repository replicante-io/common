use std::collections::HashMap;

use opentracingrust::ExtractFormat;
use opentracingrust::InjectFormat;
use opentracingrust::MapCarrier;
use opentracingrust::Result as OTResult;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;

/// Implement the MapCarrier trait for Reqwest's HeaderMap.
///
/// # Examples
///
/// Inject a span context:
///
/// ```ignore
/// use replicante_util_tracing::carriers::reqwest::HeadersCarrier;
///
/// let mut headers = HeaderMap::new();
/// HeadersCarrier::inject(span.context(), &mut headers, &tracer);
/// ```
///
/// Optionally extract a context:
///
/// ```ignore
/// use replicante_util_tracing::carriers::iron::HeadersCarrier;
///
/// let mut response = Response::new();
/// HeadersCarrier::extract(span.context(), &response.headers, &tracer);
/// ```
pub struct HeadersCarrier<'a> {
    headers: &'a mut HeaderMap,
    // This is horrible, I am sorry.
    // The MapCarrier items function returns String which we don't have
    // because of how Header iteration works.
    // To work around this, we store a view of the iterator in a compatible
    // format so we can return references to valid memory.
    iter_stage: HashMap<String, String>,
}

impl<'a> HeadersCarrier<'a> {
    /// Mutably borrow a response so it can be serialised.
    pub fn new(headers: &'a mut HeaderMap) -> HeadersCarrier<'a> {
        let mut carrier = HeadersCarrier {
            iter_stage: HashMap::new(),
            headers,
        };
        carrier.prepare_iter();
        carrier
    }

    /// Inject a `SpanContext` into the given Iron headers.
    pub fn inject(context: &SpanContext, headers: &mut HeaderMap, tracer: &Tracer) -> OTResult<()> {
        let mut carrier = HeadersCarrier::new(headers);
        let format = InjectFormat::HttpHeaders(Box::new(&mut carrier));
        tracer.inject(context, format)?;
        Ok(())
    }

    /// Checks the headers for a span context and extract it if possible.
    pub fn extract(headers: &mut HeaderMap, tracer: &Tracer) -> OTResult<Option<SpanContext>> {
        let carrier = HeadersCarrier::new(headers);
        let format = ExtractFormat::HttpHeaders(Box::new(&carrier));
        tracer.extract(format)
    }

    // Again ... sorry.
    /// Fill the the iter_stage internal variable.
    fn prepare_iter(&mut self) {
        let items: HashMap<String, String> = {
            self.headers
                .iter()
                .map(|(header, value)| {
                    let header = header.as_str().into();
                    let value = value
                        .to_str()
                        .expect("failed to conver header value to string")
                        .into();
                    (header, value)
                })
                .collect()
        };
        self.iter_stage = items;
    }
}

impl<'a> MapCarrier for HeadersCarrier<'a> {
    fn items(&self) -> Vec<(&String, &String)> {
        self.iter_stage.iter().collect()
    }

    fn get(&self, key: &str) -> Option<String> {
        match self.headers.get(key) {
            Some(value) => {
                let value = value
                    .to_str()
                    .expect("failed to conver header value to string")
                    .into();
                Some(value)
            }
            None => None,
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        let key = HeaderName::from_bytes(key.as_bytes())
            .expect("failed to convert string into header name");
        let value =
            HeaderValue::from_str(value).expect("failed to convert string into header value");
        self.headers.insert(key, value);
        self.prepare_iter();
    }
}
