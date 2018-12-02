use failure::Fail;

use slog::KV;
use slog::Record;
use slog::Serializer;


/// Extract failure information to be added to structured logging.
pub fn failure_info<F: Fail>(_fail: &F) -> FailureInfo {
    FailureInfo {}
}


/// Container for extracted failure information that implements `slog::KV`.
pub struct FailureInfo {}

impl KV for FailureInfo {
   fn serialize(&self, _record: &Record, _serializer: &mut Serializer) -> ::slog::Result {
       Ok(())
   }
}
