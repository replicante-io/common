/// Supported tracing backends and their configuration.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "backend", content = "options", deny_unknown_fields)]
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
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct ZipkinConfig {
    /// Value for the [Zipkin] service name field.
    /// 
    /// [Zipkin]: https://zipkin.io/
    pub service_name: String,

    /// List of URLs to seed the [Kafka] client.
    ///
    /// [Kafka]: https://kafka.apache.org/
    pub kafka: Vec<String>,

    /// [Kafka] topic to publish spans to (defaults to `zipkin`).
    ///
    /// [Kafka]: https://kafka.apache.org/
    #[serde(default = "ZipkinConfig::default_topic")]
    pub topic: String,
}

impl ZipkinConfig {
    fn default_topic() -> String { String::from("zipkin") }
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

        #[test]
        fn deserialise() {
            let text = r#"backend: zipkin
options:
    service_name: abc
    kafka:
        - def
        - ghi
    topic: test"#;
            let config: Config = serde_yaml::from_str(text).unwrap();
            assert_eq!(config, Config::Zipkin(ZipkinConfig {
                service_name: String::from("abc"),
                kafka: vec![String::from("def"), String::from("ghi")],
                topic: String::from("test"),
            }));
        }

        #[test]
        fn deserialise_defaults() {
            let text = r#"backend: zipkin
options:
    service_name: abc
    kafka:
        - def
        - ghi"#;
            let config: Config = serde_yaml::from_str(text).unwrap();
            assert_eq!(config, Config::Zipkin(ZipkinConfig {
                service_name: String::from("abc"),
                kafka: vec![String::from("def"), String::from("ghi")],
                topic: String::from("zipkin"),
            }));
        }

        #[test]
        #[should_panic(expected = "missing field `kafka`")]
        fn deserialise_fails() {
            let text = r#"backend: zipkin
options:
    service_name: abc"#;
            let _config: Config = serde_yaml::from_str(text).unwrap();
        }

        #[test]
        fn serialise() {
            let config = Config::Zipkin(ZipkinConfig {
                service_name: String::from("abc"),
                kafka: vec![String::from("def"), String::from("ghi")],
                topic: String::from("test"),
            });
            let text = serde_yaml::to_string(&config).unwrap();
            assert_eq!(text, r#"---
backend: zipkin
options:
  service_name: abc
  kafka:
    - def
    - ghi
  topic: test"#);
        }
    }
}
