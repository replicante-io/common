use failure::Fail;

use slog::Record;
use slog::Serializer;
use slog::KV;

/// Extract failure information to be added to structured logging.
pub fn failure_info(fail: &dyn Fail) -> FailureInfo {
    let trace = match <dyn Fail>::find_root_cause(fail).backtrace() {
        None => None,
        Some(ref bt) if bt.to_string() == "" => None,
        Some(bt) => Some(bt.to_string()),
    };
    FailureInfo {
        cause: fail
            .cause()
            .map(|cause| cause.find_root_cause().to_string()),
        cause_name: fail
            .cause()
            .map(|cause| cause.find_root_cause())
            .and_then(Fail::name)
            .map(String::from),
        layers: <dyn Fail>::iter_chain(fail).count(),
        message: fail.to_string(),
        name: fail.name().map(String::from),
        trace,
    }
}

/// Container for extracted failure information that implements `slog::KV`.
pub struct FailureInfo {
    cause: Option<String>,
    cause_name: Option<String>,
    layers: usize,
    message: String,
    name: Option<String>,
    trace: Option<String>,
}

impl KV for FailureInfo {
    fn serialize(&self, _record: &Record, serializer: &mut dyn Serializer) -> ::slog::Result {
        if let Some(cause) = self.cause.as_ref() {
            serializer.emit_str("error_cause", cause)?;
        }
        serializer.emit_usize("error_layers", self.layers)?;
        serializer.emit_str("error_message", &self.message)?;
        if let Some(cause_name) = self.cause_name.as_ref() {
            serializer.emit_str("error_cause_name", cause_name)?;
        }
        if let Some(name) = self.name.as_ref() {
            serializer.emit_str("error_name", name)?;
        }
        if let Some(trace) = self.trace.as_ref() {
            serializer.emit_str("error_trace", trace)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use failure::err_msg;
    use failure::Fail;

    use super::failure_info;

    #[test]
    fn flat_error() {
        let error = err_msg("test");
        let info = failure_info(error.as_ref());
        assert_eq!(info.cause, None);
        assert_eq!(info.layers, 1);
        assert_eq!(info.message, "test");
    }

    #[test]
    fn nested_errors() {
        let error = err_msg("errors")
            .context("more")
            .context("some")
            .context("test");
        let info = failure_info(&error);
        assert_eq!(info.cause, Some("errors".into()));
        assert_eq!(info.layers, 4);
        assert_eq!(info.message, "test");
    }
}
