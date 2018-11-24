use std::collections::BTreeMap;


/// Logging configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Flush logs asynchronously.
    #[serde(default = "Config::default_async")]
    pub async: bool,

    /// The backend to send logs to.
    #[serde(default)]
    pub backend: LoggingBackend,

    /// The minimum logging level.
    #[serde(default)]
    pub level: LoggingLevel,

    /// Advanced level configuration by module prefix.
    ///
    /// The keys in this map are used as prefix matches against log event modules.
    /// If a match is found the mapped level is used for the event.
    /// If no match is found the `level` value is used as the filter.
    #[serde(default)]
    pub modules: BTreeMap<String, LoggingLevel>,

    /// Enable verbose debug logs.
    ///
    /// When DEBUG level is enbabled, things can get loud pretty easily.
    /// To allow DEBUG level to be more usefull, only application events are emitted at
    /// DEBUG level while dependency events are emitted at INFO level.
    ///
    /// Verbose mode can be used in cases where DEBUG level should be enabled by default
    /// on all events and not just the application logs.
    #[serde(default = "Config::default_verbose")]
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            async: Config::default_async(),
            backend: LoggingBackend::default(),
            level: LoggingLevel::default(),
            modules: BTreeMap::new(),
            verbose: false,
        }
    }
}

impl Config {
    fn default_async() -> bool { true }
    fn default_verbose() -> bool { false }
}


/// List of supported logging backends.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "name", content = "options", deny_unknown_fields)]
pub enum LoggingBackend {
    /// Log objects to systemd journal (journald).
    #[cfg(feature = "journald")]
    #[serde(rename = "journald")]
    Journald,

    /// Log JSON objects to standard output.
    #[serde(rename = "json")]
    Json,
}

impl Default for LoggingBackend {
    fn default() -> LoggingBackend {
        LoggingBackend::Json
    }
}


/// Possible logging levels.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LoggingLevel {
    /// Critical
    #[serde(rename = "critical")]
    Critical,

    /// Error
    #[serde(rename = "error")]
    Error,

    /// Warning
    #[serde(rename = "warning")]
    Warning,

    /// Info
    #[serde(rename = "info")]
    Info,

    /// Debug
    #[serde(rename = "debug")]
    Debug,
}

impl Default for LoggingLevel {
    #[cfg(debug_assertions)]
    fn default() -> LoggingLevel {
        LoggingLevel::Debug
    }

    #[cfg(not(debug_assertions))]
    fn default() -> LoggingLevel {
        LoggingLevel::Info
    }
}

impl From<LoggingLevel> for ::slog::Level {
    fn from(level: LoggingLevel) -> Self {
        match level {
            LoggingLevel::Critical => ::slog::Level::Critical,
            LoggingLevel::Error => ::slog::Level::Error,
            LoggingLevel::Warning => ::slog::Level::Warning,
            LoggingLevel::Info => ::slog::Level::Info,
            LoggingLevel::Debug => ::slog::Level::Debug,
        }
    }
}
