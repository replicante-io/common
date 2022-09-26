use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crossbeam_channel::unbounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::Select;
use crossbeam_channel::Sender;
use humthreads::ErrorKind as HumthreadsErrorKind;
use humthreads::MapThread;
use humthreads::Thread;
use signal_hook::SigId;
use slog::debug;
use slog::o;
use slog::warn;
use slog::Discard;
use slog::Logger;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

/// Block the calling thread until shutdown is requested.
///
/// Shutdown is requested when:
///
///   * The process receives SIGINT.
///   * A registered thread panics.
///   * A required thread exists (optional threads are allowed to exit gracefully).
///
/// # Shutdown Flow
///
///  1. Request all registered threads to shutdown.
///  2. Execute all on_shutdown callbacks.
///  3  Wait for all registered threads to exit.
///
/// Threads and handlers are iterated on in registration order.
///
/// # Signal Handling
/// When a process is sent SIGINT the shutdown flow begins.
/// The process is allowed to take as long as it wants to shutdown.
///
/// If a second SIGINT is received while the process is shutting down
/// it will instead exit immediately.
///
/// # Example
/// ```no_run
/// # use replicante_util_upkeep::Upkeep;
/// let mut up = Upkeep::new();
/// up.on_shutdown(|| println!("Bye :wave:"));
/// up.keepalive();
/// ```
pub struct Upkeep {
    callbacks: Vec<Box<dyn Fn()>>,
    logger: Logger,
    registered_signals: Vec<SigId>,
    signal_flag: Arc<AtomicBool>,
    signal_receiver: Receiver<()>,
    signal_sender: Option<Sender<()>>,
    threads: Vec<ThreadMeta>,
}

impl Upkeep {
    pub fn new() -> Upkeep {
        let (signal_sender, signal_receiver) = unbounded();
        let signal_sender = Some(signal_sender);
        Upkeep {
            callbacks: Vec::new(),
            logger: Logger::root(Discard, o!()),
            registered_signals: Vec::new(),
            signal_flag: Arc::new(AtomicBool::new(false)),
            signal_receiver,
            signal_sender,
            threads: Vec::new(),
        }
    }

    /// Block the calling thread waiting for the process to shutdown.
    ///
    /// # Returns
    /// This method returns `true` if the process shuts down cleanly.
    pub fn keepalive(&mut self) -> bool {
        // Use crossbeam_channel::Select to poll for signals or thread exists:
        //
        //   - Generate a Select set to wait on.
        //   - Use the ready API to wait (select API seems to deadlock unless with timeout).
        //   - When a thread joins remove it from the vector.
        let mut clean_exit = true;
        loop {
            let mut set = self.select_set();
            let index = set.ready();
            match index {
                0 => {
                    warn!(self.logger, "Shutdown: signal received");
                    break;
                }
                n => {
                    let thread = &self.threads[n - 1];
                    let paniced = match thread.handle.join() {
                        Ok(()) => false,
                        Err(error) => match error.kind() {
                            HumthreadsErrorKind::Join(_) => {
                                capture_fail!(
                                    &error,
                                    self.logger,
                                    "Thread paniced";
                                    failure_info(&error),
                                );
                                clean_exit = false;
                                true
                            }
                            _ => false,
                        },
                    };
                    if paniced {
                        warn!(self.logger, "Shutdown: thread paniced");
                        break;
                    }
                    if thread.required {
                        warn!(self.logger, "Shutdown: thread exited");
                        break;
                    }
                }
            };

            // Can reach here only if an optional thread exited without a panic.
            drop(set);
            self.threads.remove(index - 1);
        }

        self.shutdown();
        self.join_threads() && clean_exit
    }

    /// Register a callback to be executed when a shutdown request is received.
    pub fn on_shutdown<F>(&mut self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.callbacks.push(Box::new(callback))
    }

    /// Register signal handers for SIGINT and SIGTERM.
    pub fn register_signal(&mut self) -> Result<(), ::std::io::Error> {
        let sender = match self.signal_sender.take() {
            Some(sender) => sender,
            None => return Ok(()),
        };
        let signals = vec![
            signal_hook::consts::signal::SIGINT,
            signal_hook::consts::signal::SIGTERM,
        ];
        for signal in signals.into_iter() {
            let signal_flag = Arc::clone(&self.signal_flag);
            let signal_sender = sender.clone();
            let callback = move || {
                if signal_flag.load(Ordering::Relaxed) {
                    ::std::process::exit(1);
                }
                signal_flag.store(true, Ordering::Relaxed);
                let _ = signal_sender.send(());
            };
            let signal_id = unsafe { signal_hook::low_level::register(signal, callback) }?;
            self.registered_signals.push(signal_id);
        }
        Ok(())
    }

    /// Register a [`Thread`] for shutdown.
    ///
    /// Threads are politely asked to stop work and then joined.
    /// Threads MUST join for the process to exit correctly.
    ///
    /// If a [`Thread`] registered with this function panics or exits
    /// the shutdown procedure for all other threads begins.
    ///
    /// [`Thread`]: https://docs.rs/humthreads/0.1.2/humthreads/struct.Thread.html
    pub fn register_thread<T: Send + 'static>(&mut self, thread: Thread<T>) {
        let thread = ThreadMeta {
            handle: thread.map(|_| ()),
            required: true,
        };
        self.threads.push(thread);
    }

    /// Similar to [`Upkeep::register_thread`] but clean exists do not shutdown the process.
    ///
    /// [`Upkeep::register_thread`]: #method.register_thread
    pub fn register_thread_optional<T: Send + 'static>(&mut self, thread: Thread<T>) {
        let thread = ThreadMeta {
            handle: thread.map(|_| ()),
            required: false,
        };
        self.threads.push(thread);
    }

    /// Set the logger to be used by the `Upkeep` instance.
    pub fn set_logger(&mut self, logger: Logger) {
        self.logger = logger;
    }

    /// Wait for each thread to join.
    fn join_threads(&mut self) -> bool {
        debug!(self.logger, "Joining with registered threads");
        let mut clean_exit = true;
        for thread in self.threads.drain(..) {
            if let Err(error) = thread.handle.join() {
                if let HumthreadsErrorKind::JoinedAlready = error.kind() {
                    debug!(self.logger, "Joined thread twice");
                    continue;
                }
                capture_fail!(&error, self.logger, "Thread paniced"; failure_info(&error));
                clean_exit = false;
            }
        }
        clean_exit
    }

    /// Return a crossbeam_channel::Select set to wait for signals or threads.
    ///
    /// The returned set has the following propertied:
    ///
    ///   - idx 0 == signals receiver
    ///   - idx n == self.threads.get(n - 1)
    fn select_set<'a, 'b: 'a>(&'b self) -> Select<'a> {
        let mut set = Select::new();
        set.recv(&self.signal_receiver);
        for thread in &self.threads {
            thread.handle.select_add(&mut set);
        }
        set
    }

    /// Handle process shutdown and trigger callback notifications.
    fn shutdown(&mut self) {
        debug!(self.logger, "Requesting shutdowns for registered threads");
        for thread in &self.threads {
            thread.handle.request_shutdown();
        }
        debug!(self.logger, "Executing shutdown callbacks");
        for callback in &self.callbacks {
            callback();
        }
    }
}

impl Default for Upkeep {
    fn default() -> Upkeep {
        Upkeep::new()
    }
}

impl Drop for Upkeep {
    fn drop(&mut self) {
        for signal in self.registered_signals.drain(..) {
            signal_hook::low_level::unregister(signal);
        }
    }
}

struct ThreadMeta {
    handle: MapThread<()>,
    required: bool,
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::time::Duration;

    use humthreads::Builder;

    use super::Upkeep;

    #[test]
    fn callback() {
        let flag = Arc::new(AtomicBool::new(false));
        let mut up = Upkeep::new();
        let inner_flag = Arc::clone(&flag);
        up.on_shutdown(move || inner_flag.store(true, Ordering::Relaxed));
        up.shutdown();
        assert_eq!(true, flag.load(Ordering::Relaxed));
    }

    #[test]
    fn thread_optional() {
        let count = Arc::new(AtomicUsize::new(0));
        let inner_count = Arc::clone(&count);
        let mut up = Upkeep::new();
        let optional = Builder::new("thread_optional_two")
            .spawn(|_| ::std::thread::sleep(Duration::from_millis(10)))
            .expect("to spawn test thread");
        up.register_thread_optional(optional);
        let thread = Builder::new("thread_optional_one")
            .spawn(move |scope| {
                for _ in 0..5 {
                    ::std::thread::sleep(Duration::from_millis(10));
                    if scope.should_shutdown() {
                        break;
                    }
                    inner_count.fetch_add(1, Ordering::Relaxed);
                }
            })
            .expect("to spawn test thread");
        up.register_thread(thread);
        let clean = up.keepalive();
        assert_eq!(true, clean);
        assert_eq!(5, count.load(Ordering::Relaxed));
    }

    #[test]
    fn thread_panics() {
        let flag = Arc::new(AtomicBool::new(false));
        let inner_flag = Arc::clone(&flag);
        let mut up = Upkeep::new();
        let thread = Builder::new("thread_panics")
            .spawn(move |_| {
                inner_flag.store(true, Ordering::Relaxed);
                panic!("this panic is expected");
            })
            .expect("to spawn test thread");
        up.register_thread(thread);
        let clean = up.keepalive();
        assert_eq!(true, flag.load(Ordering::Relaxed));
        assert_eq!(false, clean);
    }

    #[test]
    fn thread_shuts_down() {
        let flag = Arc::new(AtomicBool::new(false));
        let inner_flag = Arc::clone(&flag);
        let thread = Builder::new("thread_shuts_down")
            .spawn(move |scope| {
                loop {
                    ::std::thread::sleep(Duration::from_millis(10));
                    if scope.should_shutdown() {
                        break;
                    }
                }
                inner_flag.store(true, Ordering::Relaxed);
            })
            .expect("to spawn test thread");
        let mut up = Upkeep::new();
        up.register_thread(thread);
        up.shutdown();
        let clean = up.keepalive();
        assert_eq!(true, flag.load(Ordering::Relaxed));
        assert_eq!(true, clean);
    }

    // Tests below are commented out because they cause undefined behaviours.
    // Running this test as well as other can lead to a panic from the inners of stdlib threads:

    //use nix::sys::signal::kill;
    //use nix::sys::signal::SIGINT;
    //use nix::unistd::Pid;

    //#[test]
    // ```
    // thread '<unnamed>' panicked at 'assertion failed: c.borrow().is_none()', src/libstd/sys_common/thread_info.rs:37:26
    // test tests::signal ... ok
    // stack backtrace:
    //    0: std::sys::unix::backtrace::tracing::imp::unwind_backtrace
    //              at src/libstd/sys/unix/backtrace/tracing/gcc_s.rs:39
    //    1: std::sys_common::backtrace::_print
    //              at src/libstd/sys_common/backtrace.rs:70
    //    2: std::panicking::default_hook::{{closure}}
    //              at src/libstd/sys_common/backtrace.rs:58
    //              at src/libstd/panicking.rs:200
    //    3: std::panicking::default_hook
    //              at src/libstd/panicking.rs:215
    //    4: std::panicking::rust_panic_with_hook
    //              at src/libstd/panicking.rs:478
    //    5: std::panicking::begin_panic
    //              at src/libstd/panicking.rs:412
    //    6: std::sys_common::thread_info::set
    //              at src/libstd/sys_common/thread_info.rs:37
    //              at src/libstd/thread/local.rs:300
    //              at src/libstd/thread/local.rs:246
    //              at src/libstd/sys_common/thread_info.rs:37
    //    7: std::thread::Builder::spawn_unchecked::{{closure}}
    //              at /rustc/91856ed52c58aa5ba66a015354d1cc69e9779bdf/src/libstd/thread/mod.rs:466
    //    8: <F as alloc::boxed::FnBox<A>>::call_box
    //              at /rustc/91856ed52c58aa5ba66a015354d1cc69e9779bdf/src/liballoc/boxed.rs:749
    //    9: std::sys::unix::thread::Thread::new::thread_start
    //              at /rustc/91856ed52c58aa5ba66a015354d1cc69e9779bdf/src/liballoc/boxed.rs:759
    //              at src/libstd/sys_common/thread.rs:14
    //              at src/libstd/sys/unix/thread.rs:81
    //   10: start_thread
    //   11: clone
    // fatal runtime error: failed to initiate panic, error 5
    // error: process didn't exit successfully: `replicante_util_upkeep-3a7217a487d2749e` (signal: 6, SIGABRT: process abort signal)
    // ```
    //
    // Use the below command (after un-commenting this code) to see the error:
    // ```
    // for i in `seq 1 100`; do RUST_BACKTRACE=1 cargo test -p replicante_util_upkeep || break; done
    // ```
    //fn signal() {
    //    let flag = Arc::new(AtomicBool::new(false));
    //    let mut up = Upkeep::new();
    //    let inner_flag = Arc::clone(&flag);
    //    up.register_signal().unwrap();
    //    up.on_shutdown(move || inner_flag.store(true, Ordering::Relaxed));
    //    kill(Pid::this(), SIGINT).unwrap();
    //    let clean = up.keepalive();
    //    assert_eq!(true, flag.load(Ordering::Relaxed));
    //    assert_eq!(true, clean);
    //}

    // This test aborts the entrie tests process.
    // On one hand: yey it works! On the other: can't test really.
    //#[test]
    //fn signal_kill() {
    //    let mut up = Upkeep::new();
    //    up.register_signal().unwrap();
    //    kill(Pid::this(), SIGINT).unwrap();
    //    kill(Pid::this(), SIGINT).unwrap();
    //}
}
