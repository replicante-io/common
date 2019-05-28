extern crate failure;
extern crate sentry;
extern crate serde;
extern crate serde_derive;
extern crate slog;

mod capture;
mod format;
mod log;

#[doc(hidden)]
pub use self::capture::capture_fail_inner;
pub use self::format::format_fail;
pub use self::format::SerializableFail;
pub use self::log::failure_info;
