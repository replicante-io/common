extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate slog;

mod format;
mod log;

pub use self::format::format_fail;
pub use self::format::SerializableFail;
pub use self::log::failure_info;
