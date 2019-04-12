use opentracingrust::tracers::NoopTracer;
use opentracingrust::utils::ReporterThread;
use opentracingrust::Tracer;

use opentracingrust_zipkin::KafkaCollector;
use opentracingrust_zipkin::ZipkinEndpoint;
use opentracingrust_zipkin::ZipkinTracer;

use slog::Logger;

use super::config::ZipkinConfig;
use super::Result;
use super::TracerExtra;

/// Creates a noop tracer that discards all spans.
pub fn noop() -> Result<(Tracer, TracerExtra)> {
    let (tracer, receiver) = NoopTracer::new();
    let reporter = ReporterThread::new(receiver, NoopTracer::report);
    Ok((tracer, TracerExtra::ReporterThread(reporter)))
}

/// Creates a zipkin tracer that sends spans over kafka.
pub fn zipkin(config: ZipkinConfig, logger: Logger) -> Result<(Tracer, TracerExtra)> {
    let mut collector = KafkaCollector::new(
        ZipkinEndpoint::new(None, None, Some(config.service_name), None),
        config.topic,
        config.kafka,
    );
    let (tracer, receiver) = ZipkinTracer::new();
    let reporter = ReporterThread::new(receiver, move |span| {
        if let Err(error) = collector.collect(span) {
            error!(logger, "ZipkinTracer failed to report span"; "error" => ?error);
        }
    });
    Ok((tracer, TracerExtra::ReporterThread(reporter)))
}

#[cfg(test)]
mod tests {
    mod noop {
        use std::time::Duration;

        use super::super::noop;
        use super::super::TracerExtra;

        #[test]
        fn factory() {
            let (_tracer, extra) = noop().expect("Failed to configure NoopTracer");
            match extra {
                TracerExtra::ReporterThread(mut reporter) => {
                    reporter.stop_delay(Duration::from_millis(10))
                }
                _ => panic!("unexpected extra payload"),
            };
        }
    }
}
