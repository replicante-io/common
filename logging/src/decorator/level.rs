use slog::Drain;

use slog::Never;
use slog::SendSyncUnwindSafeDrain;
use slog::SendSyncRefUnwindSafeDrain;

use super::super::Config;


/// Alternative implementation of slog's [`LevelFilter`] with `Ok == ()`.
///
/// The default [`LevelFilter`] implementation wraps `D::Ok` into an [`Option`].
/// This makes it impossible to wrap a filtering drain into a [`Logger`].
///
/// [`LevelFilter`]: slog/struct.LevelFilter.html
/// [`Logger`]: slog/struct.Logger.html
/// [`Option`]: core/option/enum.Option.html
#[derive(Debug, Clone)]
pub struct LevelFilter<D: Drain>(pub D, pub ::slog::Level);
impl<D: Drain> Drain for LevelFilter<D> {
    type Ok = ();
    type Err = D::Err;
    fn log(
        &self,
        record: &::slog::Record,
        logger_values: &::slog::OwnedKVList,
    ) -> Result<Self::Ok, Self::Err> {
        if record.level().is_at_least(self.1) {
            self.0.log(record, logger_values)?;
        }
        Ok(())
    }
}


/// Configures the desired logging level.
pub fn level<D>(config: &Config, drain: D) -> LevelFilter<D>
    where D: SendSyncUnwindSafeDrain<Ok = (), Err = Never>,
          D: 'static + SendSyncRefUnwindSafeDrain<Ok = (), Err = Never>,
{
    LevelFilter(drain, config.level.clone().into())
}
