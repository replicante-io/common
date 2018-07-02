/// Additional program options given to the logging configuration.
pub struct Opts {
    /// The version string to attack to logs.
    pub version: String
}

impl Opts {
    pub fn new(version: String) -> Opts {
        Opts {
            version,
        }
    }
}
