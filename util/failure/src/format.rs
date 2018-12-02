use failure::Fail;


/// Format the given `Fail` for display to the user.
pub fn format_fail<F: Fail>(_fail: &F) -> String {
    "TODO".into()
}
