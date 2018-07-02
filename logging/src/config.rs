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
}

impl Default for Config {
    fn default() -> Config {
        Config {
            async: Config::default_async(),
            backend: LoggingBackend::default(),
            level: LoggingLevel::default(),
        }
    }
}

impl Config {
    /// Default value for `async` used by serde.
    fn default_async() -> bool { true }
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
