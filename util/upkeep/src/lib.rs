extern crate crossbeam_channel;
extern crate humthreads;
#[cfg(test)]
extern crate nix;
extern crate signal_hook;
#[macro_use]
extern crate slog;

extern crate replicante_util_failure;

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
use slog::Discard;
use slog::Logger;

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
/// ```norun
///# fn main() {
///   let mut up = Upkeep::new();
///   up.on_shutdown(|| println!("Bye :wave:"));
///   up.wait();
///# }
/// ```
pub struct Upkeep {
    callbacks: Vec<Box<Fn() -> ()>>,
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
        //   - Use the select API to wait.
        //   - When a thread joins remove it from the vector.
        let mut clean_exit = true;
        loop {
            let mut set = self.select_set();
            let operation = set.select();
            let index = operation.index();
            match index {
                0 => {
                    // Complete the operation to avoid panics.
                    // If we get a signal or the sender is dropped, terminate the process.
                    let _ = operation.recv(&self.signal_receiver);
                    break;
                }
                n => {
                    let thread = &self.threads[n - 1];
                    let paniced = match thread.handle.select_join(operation) {
                        Ok(()) => false,
                        Err(error) => match error.kind() {
                            HumthreadsErrorKind::Join(_) => {
                                warn!(self.logger, "Thread paniced"; failure_info(&error));
                                clean_exit = false;
                                true
                            }
                            _ => false,
                        }
                    };
                    if paniced || thread.required {
                        break;
                    }

                }
            };

            // Can reach here only if an optional thread exited without a panic.
            drop(set);
            self.threads.remove(index - 1);
        }

        // Drop the selector set to release references.
        //drop(set);
        self.shutdown();
        self.join_threads() && clean_exit
    }

    /// Register a callback to be executed when a shutdown request is received.
    pub fn on_shutdown<F>(&mut self, callback: F)
    where
        F: Fn() -> () + 'static,
    {
        self.callbacks.push(Box::new(callback))
    }

    /// Register signal handers for SIGINT and SIGTERM.
    pub fn register_signal(&mut self) -> Result<(), ::std::io::Error> {
        let sender = match self.signal_sender.take() {
            Some(sender) => sender,
            None => return Ok(()),
        };
        let signals = vec![signal_hook::SIGINT, signal_hook::SIGTERM];
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
            let signal_id = unsafe { signal_hook::register(signal, callback) }?;
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

    /// Wait for each thread to join.
    fn join_threads(&mut self) -> bool {
        let mut clean_exit = true;
        for thread in self.threads.drain(..) {
            if let Err(error) = thread.handle.join() {
                if let HumthreadsErrorKind::JoinedAlready = error.kind() {
                    debug!(self.logger, "Joined thread twice");
                    continue;
                }
                warn!(self.logger, "Thread paniced"; failure_info(&error));
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
        for thread in &self.threads {
            thread.handle.request_shutdown();
        }
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
            signal_hook::unregister(signal);
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

    use nix::sys::signal::kill;
    use nix::sys::signal::SIGINT;
    use nix::unistd::Pid;

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
    fn signal() {
        let flag = Arc::new(AtomicBool::new(false));
        let mut up = Upkeep::new();
        let inner_flag = Arc::clone(&flag);
        up.register_signal().unwrap();
        up.on_shutdown(move || inner_flag.store(true, Ordering::Relaxed));
        kill(Pid::this(), SIGINT).unwrap();
        let clean = up.keepalive();
        assert_eq!(true, flag.load(Ordering::Relaxed));
        assert_eq!(true, clean);
    }

    // This test aborts the entrie tests process.
    // On one hand: yey it works! On the other: can't test really.
    //#[test]
    //fn signal_kill() {
    //    let mut up = Upkeep::new();
    //    up.register_signal().unwrap();
    //    kill(Pid::this(), SIGINT).unwrap();
    //    kill(Pid::this(), SIGINT).unwrap();
    //}

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
}
