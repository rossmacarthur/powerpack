//! Provides a way to background a process from an Alfred workflow.
//!
//! It does this by forking the process and closing the stdin/stdout/stderr file
//! descriptors for the child process. This makes sure Alfred does not block on
//! the process. Calling [`spawn`] does the following:
//!
//! - In the parent:
//!   - Returns immediately.
//! - In the child:
//!   - Detaches the stdin/stdout/stderr file descriptors.
//!   - Sets up a panic hook that logs an error on panic.
//!   - Executes the given function.
//!   - Exit the process.
//!
//! ### 💡 Note
//!
//! Depending on your Alfred workflow settings Alfred might execute your
//! workflow many times in a short space of time. It can be useful to make sure
//! only one child process is running at a time by first acquiring a file mutex
//! in the spawned function.
//!
//! # Examples
//!
//! ```no-compile
//! powerpack::detach::spawn(|| {
//!
//!     // some expensive operation that shouldn't block Alfred
//!     //
//!     // e.g. fetch and cache a remote resource
//!
//! }).expect("forked child process");
//! ```

use std::io;
use std::panic;
use std::panic::PanicInfo;
use std::process;

#[derive(Debug, Clone, Copy)]
enum Fork {
    Parent,
    Child,
}

/// Fork the current process.
fn fork() -> io::Result<Fork> {
    // SAFETY: We are handling the error correctly.
    let r = unsafe { libc::fork() };
    handle_err(r).map(|r| match r {
        0 => Fork::Child,
        _ => Fork::Parent,
    })
}

/// Close the standard file descriptors.
fn close_std_fds() -> io::Result<()> {
    // SAFETY: We are handling the error correctly.
    handle_err(unsafe { libc::close(libc::STDOUT_FILENO) })?;
    handle_err(unsafe { libc::close(libc::STDERR_FILENO) })?;
    handle_err(unsafe { libc::close(libc::STDIN_FILENO) })?;
    Ok(())
}

fn handle_err(res: i32) -> io::Result<i32> {
    match res {
        -1 => Err(io::Error::last_os_error()),
        r => Ok(r),
    }
}

fn panic_hook(info: &PanicInfo<'_>) {
    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };
    log::error!("child panicked at '{}', {}", msg, info.location().unwrap());
}

/// Execute a function in a child process.
///
/// See the [crate] level documentation for more.
pub fn spawn<F>(f: F) -> io::Result<()>
where
    F: FnOnce(),
{
    match fork()? {
        Fork::Parent => Ok(()),
        Fork::Child => match exec_child(f) {
            Ok(()) => {
                process::exit(0);
            }
            Err(err) => {
                log::error!("{:#}", err);
                process::exit(1);
            }
        },
    }
}

fn exec_child<F>(f: F) -> io::Result<()>
where
    F: FnOnce(),
{
    close_std_fds()?;
    panic::set_hook(Box::new(panic_hook));
    f();
    Ok(())
}
