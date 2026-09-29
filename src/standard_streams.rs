//! The standard streams a program was started with closed. The Rust runtime
//! opens `/dev/null` in place of each before `main` runs, and Ruby leaves them
//! closed, so which were closed is noted before the runtime starts and they
//! are closed again once it has.

use std::sync::atomic::{AtomicU8, Ordering};

/// One bit for each of descriptors 0, 1 and 2 that was closed at start-up.
static CLOSED_AT_START: AtomicU8 = AtomicU8::new(0);

/// Note which standard descriptors are closed. It runs before the Rust
/// runtime does, from the list of functions run before `main`.
extern "C" fn note_closed_standard_streams() {
    let mut closed = 0;
    for descriptor in 0..3 {
        // SAFETY: `fcntl` with F_GETFD only reads the descriptor's flags.
        if unsafe { libc::fcntl(descriptor, libc::F_GETFD) } < 0 {
            closed |= 1 << descriptor;
        }
    }
    CLOSED_AT_START.store(closed, Ordering::SeqCst);
}

#[cfg(target_os = "linux")]
#[used]
#[unsafe(link_section = ".init_array")]
static NOTE_CLOSED_STANDARD_STREAMS: extern "C" fn() = note_closed_standard_streams;

#[cfg(target_os = "macos")]
#[used]
#[unsafe(link_section = "__DATA,__mod_init_func")]
static NOTE_CLOSED_STANDARD_STREAMS: extern "C" fn() = note_closed_standard_streams;

/// Close again the standard descriptors that were closed when the program
/// started, which the Rust runtime opened on `/dev/null` in the meantime.
pub fn close_the_ones_closed_at_start() {
    let closed = CLOSED_AT_START.load(Ordering::SeqCst);
    for descriptor in 0..3 {
        if closed & (1 << descriptor) != 0 {
            // SAFETY: the descriptor is one the runtime opened on /dev/null,
            // and nothing else holds it yet.
            unsafe { libc::close(descriptor) };
        }
    }
}
