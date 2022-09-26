use thiserror::Error;

/// Errors related to HTTP protocol logic.
#[derive(Error, Debug)]
pub enum HttpError {
    #[error("invalid value for HTTP header '{0}'")]
    // (header)
    HeaderValueInvalid(String),
}

impl HttpError {
    /// Error indicating an HTTP header was sent with an invalid value.
    pub fn header_value_invalid<S: Into<String>>(header: S) -> HttpError {
        HttpError::HeaderValueInvalid(header.into())
    }
}

/// Errors related to tracing contexts logic.
#[derive(Error, Debug)]
pub enum TracingContextError {
    #[error("failed to extract tracing context")]
    Extract,

    #[error("failed to inject tracing context")]
    Inject,
}
