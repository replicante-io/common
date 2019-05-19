use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

/// Supported tracing backends and their configuration.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "backend", content = "options")]
pub enum Config {
    /// The `Noop` tracer (default).
    ///
    /// A tracer that discards all spans.
    /// Used when integration with distributed tracing is not needed.
    #[serde(rename = "noop")]
    Noop,

    /// [Zipkin] tracer backend.
    ///
    /// Spans are sent to [Zipkin] over the [Kafka] collector.
    ///
    /// [Kafka]: https://kafka.apache.org/
    /// [Zipkin]: https://zipkin.io/
    #[serde(rename = "zipkin")]
    Zipkin(ZipkinConfig),
}

impl Default for Config {
    fn default() -> Config {
        Config::Noop
    }
}

/// Zipkin specific configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "transport", content = "options")]
pub enum ZipkinConfig {
    /// Zipkin HTTP transport options.
    #[serde(rename = "http")]
    HTTP(ZipkinHttp),

    /// Zipkin Kafka transport options.
    #[serde(rename = "kafka")]
    Kafka(ZipkinKafka),
}

/// Zipkin HTTP transport options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ZipkinHttp {
    /// Number of buffered spans that should trigger a flush.
    #[serde(default = "ZipkinHttp::default_flush_count")]
    pub flush_count: usize,

    /// Muximum delay between span flushes in milliseconds.
    #[serde(default)]
    pub flush_timeout_millis: Option<u64>,

    /// Custom headers to attach to POST requests.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,

    /// Target URL to post spans to.
    pub url: String,
}

impl ZipkinHttp {
    fn default_flush_count() -> usize {
        100
    }
}

/// Zipkin Kafka transport options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ZipkinKafka {
    /// List of URLs to seed the [Kafka] client.
    ///
    /// [Kafka]: https://kafka.apache.org/
    pub kafka: Vec<String>,

    /// [Kafka] topic to publish spans to (defaults to `zipkin`).
    ///
    /// [Kafka]: https://kafka.apache.org/
    #[serde(default = "ZipkinKafka::default_topic")]
    pub topic: String,
}

impl ZipkinKafka {
    fn default_topic() -> String {
        String::from("zipkin")
    }
}

#[cfg(test)]
mod tests {
    mod noop {
        use serde_yaml;

        use super::super::Config;

        #[test]
        fn deserialise() {
            let text = "backend: noop";
            let config: Config = serde_yaml::from_str(text).unwrap();
            assert_eq!(config, Config::Noop);
        }

        #[test]
        fn serialise() {
            let config = Config::Noop;
            let text = serde_yaml::to_string(&config).unwrap();
            assert_eq!(text, "---\nbackend: noop");
        }
    }

    mod zipkin {
        use serde_yaml;

        use super::super::Config;
        use super::super::ZipkinConfig;
        use super::super::ZipkinKafka;

        #[test]
        fn deserialise() {
            let text = r#"backend: zipkin
options:
  transport: kafka
  options:
    kafka:
      - def
      - ghi
    topic: test"#;
            let config: Config = serde_yaml::from_str(text).unwrap();
            assert_eq!(
                config,
                Config::Zipkin(ZipkinConfig::Kafka(ZipkinKafka {
                    kafka: vec![String::from("def"), String::from("ghi")],
                    topic: String::from("test"),
                }))
            );
        }

        #[test]
        fn deserialise_defaults() {
            let text = r#"backend: zipkin
options:
  transport: kafka
  options:
    kafka:
      - def
      - ghi"#;
            let config: Config = serde_yaml::from_str(text).unwrap();
            assert_eq!(
                config,
                Config::Zipkin(ZipkinConfig::Kafka(ZipkinKafka {
                    kafka: vec![String::from("def"), String::from("ghi")],
                    topic: String::from("zipkin"),
                }))
            );
        }

        #[test]
        #[should_panic(expected = "missing field `kafka`")]
        fn deserialise_fails() {
            let text = r#"backend: zipkin
options:
  transport: kafka
  options: {}"#;
            let _config: Config = serde_yaml::from_str(text).unwrap();
        }

        #[test]
        fn serialise() {
            let config = Config::Zipkin(ZipkinConfig::Kafka(ZipkinKafka {
                kafka: vec![String::from("def"), String::from("ghi")],
                topic: String::from("test"),
            }));
            let text = serde_yaml::to_string(&config).unwrap();
            assert_eq!(
                text,
                r#"---
backend: zipkin
options:
  transport: kafka
  options:
    kafka:
      - def
      - ghi
    topic: test"#
            );
        }
    }
}
