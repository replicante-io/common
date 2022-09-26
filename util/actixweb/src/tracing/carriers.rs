use std::collections::HashMap;

use actix_web::http::header::HeaderMap;
use actix_web::http::header::HeaderName;
use actix_web::http::header::HeaderValue;
use anyhow::Context;
use anyhow::Result;
use opentracingrust::ExtractFormat;
use opentracingrust::InjectFormat;
use opentracingrust::MapCarrier;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use crate::errors::HttpError;
use crate::errors::TracingContextError;

/// Implement the MapCarrier trait for Iron's Headers.
pub struct HeadersCarrier<'a> {
    headers: &'a mut HeaderMap,
    // This is horrible, I am sorry.
    // The MapCarrier items function returns pointers which we don't have
    // because of how Iron's Header iteration works.
    // To work around this, we store a view of the iterator in a compatible
    // format so we can return references to valid memory.
    iter_stage: HashMap<String, String>,
}

impl<'a> HeadersCarrier<'a> {
    // Again ... sorry.
    /// Fill the the iter_stage internal variable.
    fn prepare_iter(&mut self) -> Result<()> {
        let mut items = HashMap::new();
        for (header, value) in self.headers.iter() {
            let header = String::from(header.as_str());
            let value = value
                .to_str()
                .with_context(|| HttpError::header_value_invalid(&header))?;
            let value = String::from(value);
            items.insert(header, value);
        }
        self.iter_stage = items;
        Ok(())
    }
}

impl<'a> HeadersCarrier<'a> {
    /// Mutably borrow a response so it can be serialised.
    pub fn new(headers: &'a mut HeaderMap) -> Result<HeadersCarrier<'a>> {
        let mut carrier = HeadersCarrier {
            iter_stage: HashMap::new(),
            headers,
        };
        carrier.prepare_iter()?;
        Ok(carrier)
    }

    /// Inject a `SpanContext` into the given Iron headers.
    pub fn inject(context: &SpanContext, headers: &mut HeaderMap, tracer: &Tracer) -> Result<()> {
        let mut carrier = HeadersCarrier::new(headers)?;
        let format = InjectFormat::HttpHeaders(Box::new(&mut carrier));
        tracer
            .inject(context, format)
            .map_err(|error| anyhow::anyhow!(error.to_string()))
            .context(TracingContextError::Inject)?;
        Ok(())
    }

    /// Checks the headers for a span context and extract it if possible.
    pub fn extract(headers: &mut HeaderMap, tracer: &Tracer) -> Result<Option<SpanContext>> {
        let carrier = HeadersCarrier::new(headers)?;
        let format = ExtractFormat::HttpHeaders(Box::new(&carrier));
        let context = tracer
            .extract(format)
            .map_err(|error| anyhow::anyhow!(error.to_string()))
            .context(TracingContextError::Extract)?;
        Ok(context)
    }
}

impl<'a> MapCarrier for HeadersCarrier<'a> {
    fn items(&self) -> Vec<(&String, &String)> {
        self.iter_stage.iter().collect()
    }

    fn get(&self, key: &str) -> Option<String> {
        match self.headers.get(key) {
            None => None,
            Some(value) => {
                // Headers are validated on creation.
                let value = value.to_str().unwrap();
                Some(String::from(value))
            }
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        let header = HeaderName::from_bytes(key.as_bytes()).unwrap();
        let value = HeaderValue::from_str(value).unwrap();
        self.headers.insert(header, value);
        self.prepare_iter().unwrap();
    }
}
