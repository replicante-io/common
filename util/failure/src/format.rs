use failure::Fail;
use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Format the given `Fail` for display to the user.
pub fn format_fail(fail: &dyn Fail) -> String {
    let mut message = String::new();
    message.push_str(&format!("Error: {}", fail));
    for cause in fail.iter_causes() {
        message.push_str(&format!("\n    Caused by: {}", cause));
    }
    let bt = match fail.find_root_cause().backtrace() {
        None => None,
        Some(ref bt) if bt.to_string() == "" => None,
        Some(bt) => Some(bt),
    };
    if let Some(bt) = bt {
        message.push_str(&format!("\n    Backtrace: {}", bt));
    }
    message
}

/// Serde serializable/deserializable "view" of a `Fail` error.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct SerializableFail {
    /// Error message.
    pub error: String,

    /// Layers of errors that ultimately caused this error.
    pub layers: Vec<String>,

    /// Optional formatted backtrace to aid debugging.
    pub trace: Option<String>,

    /// Identifier of the reported error variant.
    #[serde(default)]
    pub variant: Option<String>,
}

impl<E: Fail> From<&E> for SerializableFail {
    fn from(error: &E) -> SerializableFail {
        let layers = <dyn Fail>::iter_chain(error)
            .map(ToString::to_string)
            .collect();
        let trace = match error.backtrace().map(ToString::to_string) {
            None => None,
            Some(ref bt) if bt.is_empty() => None,
            Some(bt) => Some(bt),
        };
        let variant = error.name().map(ToString::to_string);
        SerializableFail {
            error: error.to_string(),
            layers,
            trace,
            variant,
        }
    }
}

#[cfg(test)]
mod test {
    use failure::err_msg;
    use failure::Fail;

    use super::format_fail;
    use super::SerializableFail;

    #[test]
    fn flat_error() {
        let error = err_msg("test");
        let msg = format_fail(error.as_ref());
        assert_eq!(msg, "Error: test");
    }

    #[test]
    fn nested_errors() {
        let error = err_msg("errors")
            .context("more")
            .context("some")
            .context("test");
        let msg = format_fail(&error);
        assert_eq!(
            msg,
            r#"Error: test
    Caused by: some
    Caused by: more
    Caused by: errors"#
        );
    }

    #[test]
    fn serializable_fail() {
        let error = err_msg("test").context("chained").context("failures");
        let error = SerializableFail::from(&error);
        assert_eq!(error.error, "failures");
        assert_eq!(
            error.layers,
            vec![
                String::from("failures"),
                String::from("chained"),
                String::from("test"),
            ]
        );
        assert_eq!(error.trace, None);
    }
}
