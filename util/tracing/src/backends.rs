use std::str::FromStr;
use std::time::Duration;

use failure::ResultExt;
use humthreads::Builder;
use humthreads::ThreadScope;
use opentracingrust::tracers::NoopTracer;
use opentracingrust::FinishedSpan;
use opentracingrust::Tracer;
use opentracingrust_zipkin::HttpCollector;
use opentracingrust_zipkin::HttpCollectorOpts;
use opentracingrust_zipkin::KafkaCollector;
use opentracingrust_zipkin::ZipkinEndpoint;
use opentracingrust_zipkin::ZipkinTracer;
use slog::Logger;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

use super::config::ZipkinConfig;
use super::ErrorKind;
use super::Opts;
use super::Result;

/// Creates a noop tracer that discards all spans.
pub fn noop() -> Result<Tracer> {
    let (tracer, _receiver) = NoopTracer::new();
    Ok(tracer)
}

/// Creates a zipkin tracer that sends spans over kafka.
pub fn zipkin(config: ZipkinConfig, opts: Opts) -> Result<Tracer> {
    // Initialise tracer and collector.
    let (tracer, receiver) = ZipkinTracer::new();
    let endpoint = ZipkinEndpoint::new(None, None, Some(opts.service_name.to_string()), None);
    let mut collector = match config {
        ZipkinConfig::HTTP(config) => {
            let mut headers = reqwest::header::HeaderMap::new();
            for (key, value) in config.headers.into_iter() {
                let key = reqwest::header::HeaderName::from_str(&key).with_context(|_| {
                    ErrorKind::Config(format!(
                        "invalid header name '{}' for Zipkin's HTTP transport",
                        key
                    ))
                })?;
                let value = reqwest::header::HeaderValue::from_str(&value).with_context(|_| {
                    ErrorKind::Config(format!(
                        "invalid header value '{}' for Zipkin's HTTP transport",
                        value
                    ))
                })?;
                headers.insert(key, value);
            }
            let options = HttpCollectorOpts::new(config.url.as_str(), endpoint)
                .flush_count(config.flush_count)
                .flush_timeout(
                    config
                        .flush_timeout_millis
                        .map(Duration::from_millis)
                        .unwrap_or(opts.flush_timeout),
                )
                .headers(headers);
            let collector = HttpCollector::new(options);
            ZipkinCollector::Http(Box::new(collector))
        }
        ZipkinConfig::Kafka(config) => {
            let collector = KafkaCollector::new(endpoint, config.topic, config.kafka);
            ZipkinCollector::Kafka(Box::new(collector))
        }
    };

    // Setup background thread to collect and ship spans.
    let logger = opts.logger.clone();
    let recv_timeout = opts.flush_timeout;
    let thread = Builder::new("r:u:t:zipkin:collector")
        .full_name("replicante:util:zipkin:collector")
        .spawn(move |scope| {
            scope.activity("waiting for spans to collect");
            while !scope.should_shutdown() {
                let span = match receiver.recv_timeout(recv_timeout) {
                    Ok(span) => Some(span),
                    Err(error) if error.is_timeout() => None,
                    Err(error) => {
                        capture_fail!(
                            &error,
                            logger,
                            "Error receiving distributed tracing span";
                            "tracer" => "zipkin",
                            failure_info(&error),
                        );
                        // Shutdown the reporter thread, which in turn will terminate the process.
                        break;
                    }
                };
                zipkin_process(&scope, &logger, &mut collector, span);
            }
        })
        .with_context(|_| ErrorKind::ThreadSpawn("span collector"))?;
    opts.upkeep.register_thread(thread);
    Ok(tracer)
}

/// Pass a span to the configured collector.
fn zipkin_process(
    scope: &ThreadScope,
    logger: &Logger,
    collector: &mut ZipkinCollector,
    span: Option<FinishedSpan>,
) {
    let _guard = scope.scoped_activity("processing received span");
    match collector {
        ZipkinCollector::Http(ref mut collector) => {
            if let Some(span) = span {
                collector.collect(span);
            }
            if let Err(error) = collector.lazy_flush() {
                capture_fail!(
                    &error,
                    logger,
                    "Error collecting distributed tracer span";
                    "collector" => "http",
                    "tracer" => "zipkin",
                    failure_info(&error),
                );
            }
        }
        ZipkinCollector::Kafka(ref mut collector) => {
            if let Some(span) = span {
                if let Err(error) = collector.collect(span) {
                    let error = failure::SyncFailure::new(error);
                    capture_fail!(
                        &error,
                        logger,
                        "Error collecting distributed tracer span";
                        "collector" => "kafka",
                        "tracer" => "zipkin",
                        failure_info(&error),
                    );
                }
            }
        }
    };
}

/// Container for the configured zipkin collector.
enum ZipkinCollector {
    Http(Box<HttpCollector>),
    Kafka(Box<KafkaCollector>),
}

#[cfg(test)]
mod tests {
    mod noop {
        use super::super::noop;

        #[test]
        fn factory() {
            let _tracer = noop().expect("Failed to configure NoopTracer");
        }
    }
}
