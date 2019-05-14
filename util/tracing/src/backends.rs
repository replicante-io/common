use failure::format_err;
use opentracingrust::tracers::NoopTracer;
use opentracingrust::utils::ReporterThread;
use opentracingrust::Tracer;
use opentracingrust_zipkin::KafkaCollector;
use opentracingrust_zipkin::ZipkinEndpoint;
use opentracingrust_zipkin::ZipkinTracer;
use slog::Logger;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

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
            // TODO: once error implements std::Error drop the format.
            let error = format_err!("{:?}", error);
            capture_fail!(
                error.as_fail(),
                logger,
                "ZipkinTracer failed to report span";
                failure_info(error.as_fail()),
            );
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
