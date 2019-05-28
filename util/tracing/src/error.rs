use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;
use opentracingrust::Log;
use opentracingrust::Span;

use replicante_util_failure::format_fail;

/// Re-implement `FailSpan` for `Fail` errors :-(
pub fn fail_span<F: Fail>(error: F, span: &mut Span) -> F {
    span.tag("error", true);
    span.log(
        Log::new()
            .log("event", "error")
            .log("message", error.to_string())
            .log("error.object", format_fail(&error)),
    );
    error
}

/// Error information returned by functions in case of errors.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }

    fn name(&self) -> Option<&str> {
        self.kind().kind_name()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error(inner)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error(Context::new(kind))
    }
}

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "configuration error: {}", _0)]
    Config(String),

    #[fail(display = "unable to spawn {} thread", _0)]
    ThreadSpawn(&'static str),
}

impl ErrorKind {
    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::Config(_) => "Config",
            ErrorKind::ThreadSpawn(_) => "ThreadSpawn",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
