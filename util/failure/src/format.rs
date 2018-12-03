use failure::Fail;


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


#[cfg(test)]
mod test {
    use failure::Fail;
    use failure::err_msg;

    use super::format_fail;


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
        assert_eq!(msg, r#"Error: test
    Caused by: some
    Caused by: more
    Caused by: errors"#);
    }
}
