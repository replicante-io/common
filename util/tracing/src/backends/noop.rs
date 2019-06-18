use failure::ResultExt;
use humthreads::Builder;
use opentracingrust::tracers::NoopTracer;
use opentracingrust::Tracer;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

use crate::ErrorKind;
use crate::Opts;
use crate::Result;

/// Creates a noop tracer that discards all spans.
pub fn noop(opts: Opts) -> Result<Tracer> {
    let (tracer, receiver) = NoopTracer::new();
    let logger = opts.logger.clone();
    let recv_timeout = opts.flush_timeout;
    let thread = Builder::new("r:u:t:noop:collector")
        .full_name("replicante:util:noop:collector")
        .spawn(move |scope| {
            scope.activity("waiting for spans to collect");
            while !scope.should_shutdown() {
                match receiver.recv_timeout(recv_timeout) {
                    Ok(_) => (),
                    Err(error) if error.is_timeout() => (),
                    Err(error) => {
                        capture_fail!(
                            &error,
                            logger,
                            "Error receiving distributed tracing span";
                            "tracer" => "noop",
                            failure_info(&error),
                        );
                        // Shutdown the reporter thread, which in turn will terminate the process.
                        break;
                    }
                };
            }
        })
        .with_context(|_| ErrorKind::ThreadSpawn("span collector"))?;
    opts.upkeep.register_thread(thread);
    Ok(tracer)
}

#[cfg(test)]
mod tests {
    use slog::o;
    use slog::Discard;
    use slog::Logger;

    use replicante_util_upkeep::Upkeep;

    use super::noop;
    use crate::Opts;

    #[test]
    fn factory() {
        let logger = Logger::root(Discard, o!());
        let mut upkeep = Upkeep::new();
        let opts = Opts::new("test", logger, &mut upkeep);
        let _tracer = noop(opts).expect("Failed to configure NoopTracer");
    }
}
