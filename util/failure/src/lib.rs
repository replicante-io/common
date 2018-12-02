extern crate failure;
extern crate slog;


mod format;
mod log;


pub use self::format::format_fail;
pub use self::log::failure_info;
