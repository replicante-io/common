extern crate opentracingrust;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde_yaml;


mod config;

pub use self::config::Config;
