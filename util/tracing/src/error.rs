/// Error information returned by functions in case of errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("unable to spawn {0} thread")]
    ThreadSpawn(&'static str),
}
