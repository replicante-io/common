use opentracingrust::Log;
use opentracingrust::Span;

/// Re-implement `FailSpan` for `Fail` errors :-(
pub fn fail_span<'a, F, S>(error: F, span: S) -> F
where
    F: failure::Fail,
    S: Into<Option<&'a mut Span>>,
{
    if let Some(span) = span.into() {
        span.tag("error", true);
        span.log(
            Log::new()
                .log("event", "error")
                .log("message", error.to_string()),
        );
    }
    error
}

/// Error information returned by functions in case of errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("unable to spawn {0} thread")]
    ThreadSpawn(&'static str),
}
