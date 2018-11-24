use std::collections::HashMap;

use slog::Drain;
use slog::Level;
use slog::Never;
use slog::OwnedKVList;
use slog::Record;
use slog::SendSyncRefUnwindSafeDrain;
use slog::SendSyncUnwindSafeDrain;

use super::super::Config;


/// Alternative implementation of slog's [`LevelFilter`] with `Ok == ()`.
///
/// The default [`LevelFilter`] implementation wraps `D::Ok` into an [`Option`].
/// This makes it impossible to wrap a filtering drain into a [`Logger`].
///
/// [`LevelFilter`]: slog/struct.LevelFilter.html
/// [`Logger`]: slog/struct.Logger.html
/// [`Option`]: core/option/enum.Option.html
#[derive(Clone, Debug)]
pub struct LevelFilter<D: Drain> {
    default: Level,
    drain: D,
    modules: Vec<PrefixLevel>,
}

impl<D: Drain> LevelFilter<D> {
    pub fn new(drain: D, default: Level) -> LevelFilter<D> {
        LevelFilter {
            default,
            drain,
            modules: Vec::new(),
        }
    }

    fn allow(&self, record: &Record) -> bool {
        let module = record.module();
        for filter in self.modules.iter() {
            if module.starts_with(&filter.prefix) {
                return record.level().is_at_least(filter.level);
            }
        }
        record.level().is_at_least(self.default)
    }

    pub fn modules(&mut self, prefixes: HashMap<String, Level>) {
        let mut prefixes: Vec<PrefixLevel> = prefixes.into_iter()
            .map(PrefixLevel::from)
            .collect();
        prefixes.sort_unstable_by_key(|p| p.prefix.clone());
        prefixes.reverse();
        self.modules = prefixes;
    }
}

impl<D: Drain> Drain for LevelFilter<D> {
    type Ok = ();
    type Err = D::Err;

    fn log(&self, record: &Record, logger_values: &OwnedKVList) -> Result<Self::Ok, Self::Err> {
        if self.allow(record) {
            self.drain.log(record, logger_values)?;
        }
        Ok(())
    }
}


/// Prefix based levels.
#[derive(Clone, Debug, Eq, PartialEq)]
struct PrefixLevel {
    pub prefix: String,
    pub level: Level,
}

impl From<(String, Level)> for PrefixLevel {
    fn from(pair: (String, Level)) -> PrefixLevel {
        PrefixLevel {
            prefix: pair.0,
            level: pair.1,
        }
    }
}


/// Configures the desired logging level.
pub fn level<D>(config: &Config, drain: D) -> LevelFilter<D>
    where D: SendSyncUnwindSafeDrain<Ok = (), Err = Never>,
          D: 'static + SendSyncRefUnwindSafeDrain<Ok = (), Err = Never>,
{
    let mut filter = LevelFilter::new(drain, config.level.clone().into());
    filter.modules(
        config.modules.clone().into_iter().map(|(prefix, level)| (prefix, level.into())).collect()
    );
    filter
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use slog::Discard;
    use slog::Level;

    use super::LevelFilter;
    use super::PrefixLevel;

    #[test]
    fn default_emit() {
        let drain = Discard;
        let filter = LevelFilter::new(drain, Level::Warning);
        let args = format_args!("test");
        let record = record!(Level::Error, "test", &args, b!());
        let allowed = filter.allow(&record);
        assert!(allowed);
    }

    #[test]
    fn default_skip() {
        let drain = Discard;
        let filter = LevelFilter::new(drain, Level::Warning);
        let args = format_args!("test");
        let record = record!(Level::Info, "test", &args, b!());
        let allowed = filter.allow(&record);
        assert!(!allowed);
    }

    #[test]
    fn prefix_emit() {
        let drain = Discard;
        let mut filter = LevelFilter::new(drain, Level::Warning);
        let args = format_args!("test");
        let record = record!(Level::Debug, "test", &args, b!());
        filter.modules.push(PrefixLevel {
            prefix: "replicante".into(),
            level: Level::Debug,
        });
        let allowed = filter.allow(&record);
        assert!(allowed);
    }

    #[test]
    fn prefix_skip() {
        let drain = Discard;
        let mut filter = LevelFilter::new(drain, Level::Warning);
        let args = format_args!("test");
        let record = record!(Level::Warning, "test", &args, b!());
        filter.modules.push(PrefixLevel {
            prefix: "replicante".into(),
            level: Level::Error,
        });
        let allowed = filter.allow(&record);
        assert!(!allowed);
    }

    #[test]
    fn prefix_sorted_check() {
        let drain = Discard;
        let mut filter = LevelFilter::new(drain, Level::Warning);
        let args = format_args!("test");
        let record = record!(Level::Debug, "test", &args, b!());
        filter.modules.push(PrefixLevel {
            prefix: "repli".into(),
            level: Level::Error,
        });
        filter.modules.push(PrefixLevel {
            prefix: "replicante".into(),
            level: Level::Debug,
        });
        let allowed = filter.allow(&record);
        assert!(!allowed);
    }

    #[test]
    fn modules_are_sorted() {
        let drain = Discard;
        let mut filter = LevelFilter::new(drain, Level::Warning);
        let mut prefixes = HashMap::new();
        prefixes.insert("test".into(), Level::Debug);
        prefixes.insert("ac".into(), Level::Warning);
        prefixes.insert("abc".into(), Level::Info);
        prefixes.insert("a".into(), Level::Error);
        filter.modules(prefixes);
        assert_eq!(filter.modules, vec![PrefixLevel {
            prefix: "test".into(),
            level: Level::Debug,
        }, PrefixLevel {
            prefix: "ac".into(),
            level: Level::Warning,
        }, PrefixLevel {
            prefix: "abc".into(),
            level: Level::Info,
        }, PrefixLevel {
            prefix: "a".into(),
            level: Level::Error,
        }]);
    }
}
