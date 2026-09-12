// File descriptors an IO object stands over. A pipe, or a descriptor handed
// in by number, is held here under a handle the Ruby side carries, the same
// way `Socket.__net__` holds its sockets.

use std::collections::HashMap;
use std::os::unix::io::{AsRawFd as _, FromRawFd as _, IntoRawFd as _, OwnedFd, RawFd};

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::core::VirtualMachine;

/// The descriptors this program holds open through an IO object.
#[derive(Default)]
pub(crate) struct OpenStreams {
    held: HashMap<u64, OwnedFd>,
    /// Descriptors named from outside, which this program did not open and
    /// must not close when the IO object goes.
    borrowed: HashMap<u64, RawFd>,
    next: u64,
}

impl OpenStreams {
    /// The descriptor a handle names, whether this program opened it or was
    /// handed it.
    fn number_of(&self, handle: u64) -> Option<RawFd> {
        self.held
            .get(&handle)
            .map(|held| held.as_raw_fd())
            .or_else(|| self.borrowed.get(&handle).copied())
    }

    fn keep(&mut self, held: OwnedFd) -> u64 {
        let named = self.next;
        self.next += 1;
        self.held.insert(named, held);
        named
    }

    fn borrow(&mut self, number: RawFd) -> u64 {
        let named = self.next;
        self.next += 1;
        self.borrowed.insert(named, number);
        named
    }
}

/// The error a stream operation reports, named the way the operating system
/// names it.
fn stream_error(problem: &std::io::Error, what: &str, position: Position) -> MetorexError {
    let named = match problem.raw_os_error() {
        Some(libc::EPIPE) => "Errno::EPIPE",
        Some(libc::EBADF) => "Errno::EBADF",
        Some(libc::EAGAIN) => "Errno::EAGAIN",
        Some(libc::ENOTTY) => "Errno::ENOTTY",
        Some(libc::EINVAL) => "Errno::EINVAL",
        Some(libc::ENOENT) => "Errno::ENOENT",
        Some(libc::EACCES) => "Errno::EACCES",
        Some(libc::EISDIR) => "Errno::EISDIR",
        Some(libc::ENOTDIR) => "Errno::ENOTDIR",
        Some(libc::EEXIST) => "Errno::EEXIST",
        Some(libc::ELOOP) => "Errno::ELOOP",
        Some(libc::ENAMETOOLONG) => "Errno::ENAMETOOLONG",
        _ => "IOError",
    };
    crate::vm::errors::simple_exception(named, &format!("{what}: {problem}"), position)
}

/// How long a read waits for something to arrive before giving up. Metorex
/// runs one thread, so a stream nothing is left to write to has nobody to
/// wait for.
const READ_LIMIT: std::time::Duration = std::time::Duration::from_millis(250);

fn closed_error(position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("IOError", "closed stream", position)
}

impl VirtualMachine {
    /// The bridge the Ruby side of IO reaches its descriptors through, called
    /// as `IO.__stream__(action, handle, text, count)`.
    pub(crate) fn stream_action(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(Object::String(action)) = arguments.first() else {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::AtLeast(1),
                arguments.len(),
                position,
            ));
        };
        let handle = match arguments.get(1) {
            Some(Object::Int(held)) => *held as u64,
            _ => 0,
        };
        let text = match arguments.get(2) {
            Some(Object::String(held)) => held.as_str().to_string(),
            _ => String::new(),
        };
        let count = match arguments.get(3) {
            Some(Object::Int(held)) => *held,
            _ => 0,
        };
        match &*action.as_str() {
            // Two joined descriptors: what is written to the second is read
            // from the first.
            "pipe" => {
                let mut ends: [libc::c_int; 2] = [0, 0];
                // SAFETY: `ends` is a two-element array, which is what
                // `pipe` fills in.
                let made = unsafe { libc::pipe(ends.as_mut_ptr()) };
                if made != 0 {
                    return Err(stream_error(
                        &std::io::Error::last_os_error(),
                        "pipe",
                        position,
                    ));
                }
                // SAFETY: both descriptors came from `pipe` and are owned
                // from here on.
                let (reader, writer) =
                    unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) };
                // Ruby hands back pipe ends that answer straight away rather
                // than waiting, which is what `nonblock?` reports for them.
                for end in [reader.as_raw_fd(), writer.as_raw_fd()] {
                    // SAFETY: both descriptors came from `pipe` just above.
                    unsafe {
                        let flags = libc::fcntl(end, libc::F_GETFL);
                        libc::fcntl(end, libc::F_SETFL, flags | libc::O_NONBLOCK);
                    }
                }
                let first = self.open_streams.keep(reader);
                let second = self.open_streams.keep(writer);
                Ok(Object::array(vec![
                    Object::Int(first as i64),
                    Object::Int(second as i64),
                ]))
            }
            // A descriptor named by number, which this program did not open.
            "adopt" => Ok(Object::Int(self.open_streams.borrow(count as RawFd) as i64)),
            // Another handle on the same descriptor, closed on its own.
            "dup" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                // SAFETY: `number` is a descriptor this program holds open.
                let copied = unsafe { libc::dup(number) };
                if copied < 0 {
                    return Err(stream_error(
                        &std::io::Error::last_os_error(),
                        "dup",
                        position,
                    ));
                }
                // SAFETY: `copied` came from `dup` and is owned from here on.
                let owned = unsafe { OwnedFd::from_raw_fd(copied) };
                Ok(Object::Int(self.open_streams.keep(owned) as i64))
            }
            // Point this stream's descriptor at what another stream holds,
            // so everything already reading or writing through the number
            // reaches the new place.
            "reopen" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                let Some(source) = self.open_streams.number_of(count as u64) else {
                    return Err(closed_error(position));
                };
                // SAFETY: both are descriptors this program holds open.
                if unsafe { libc::dup2(source, number) } < 0 {
                    return Err(stream_error(
                        &std::io::Error::last_os_error(),
                        "reopen",
                        position,
                    ));
                }
                Ok(Object::Int(i64::from(number)))
            }
            // `fcntl` asks the operating system about a descriptor, or sets
            // one of the flags it keeps.
            "fcntl" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                let command = count as libc::c_int;
                let argument = match arguments.get(4) {
                    Some(Object::Int(held)) => *held as libc::c_int,
                    _ => 0,
                };
                // SAFETY: `number` is a descriptor this program holds open,
                // and the command decides whether the argument is read.
                let answered = unsafe { libc::fcntl(number, command, argument) };
                if answered < 0 {
                    return Err(stream_error(
                        &std::io::Error::last_os_error(),
                        "fcntl",
                        position,
                    ));
                }
                Ok(Object::Int(i64::from(answered)))
            }
            "fileno" => match self.open_streams.number_of(handle) {
                Some(number) => Ok(Object::Int(i64::from(number))),
                None => Err(closed_error(position)),
            },
            "open?" => Ok(Object::Bool(self.open_streams.number_of(handle).is_some())),
            // Whether the descriptor is still one the operating system
            // holds, which a second handle on a closed one is not.
            "live?" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Ok(Object::Bool(false));
                };
                // SAFETY: asking about a descriptor reads nothing and writes
                // nothing, whether or not it is still open.
                Ok(Object::Bool(
                    unsafe { libc::fcntl(number, libc::F_GETFD) } >= 0,
                ))
            }
            "close" => {
                self.open_streams.held.remove(&handle);
                self.open_streams.borrowed.remove(&handle);
                Ok(Object::Nil)
            }
            "write" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                let bytes = super::pack_format::string_to_bytes(&text);
                // SAFETY: `bytes` names a run this call only reads from.
                let written = unsafe {
                    libc::write(number, bytes.as_ptr() as *const libc::c_void, bytes.len())
                };
                if written < 0 {
                    return Err(stream_error(
                        &std::io::Error::last_os_error(),
                        "write",
                        position,
                    ));
                }
                Ok(Object::Int(written as i64))
            }
            "read" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                let wanted = if count > 0 { count as usize } else { 65536 };
                let mut buffer = vec![0u8; wanted];
                // A stream that answers straight away says so rather than
                // waiting, and what a program means by a read is to wait for
                // what is coming, so the read is tried again for a while.
                let deadline = std::time::Instant::now() + READ_LIMIT;
                let read = loop {
                    // SAFETY: `buffer` names a run of `wanted` bytes this
                    // call only writes into.
                    let held = unsafe {
                        libc::read(number, buffer.as_mut_ptr() as *mut libc::c_void, wanted)
                    };
                    if held >= 0 {
                        break held;
                    }
                    let problem = std::io::Error::last_os_error();
                    if problem.raw_os_error() != Some(libc::EAGAIN)
                        || std::time::Instant::now() >= deadline
                    {
                        return Err(stream_error(&problem, "read", position));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(2));
                };
                buffer.truncate(read as usize);
                Ok(super::pack_format::bytes_to_string(&buffer))
            }
            // Whether the descriptor is a terminal, which decides how a
            // program reading it behaves.
            "tty?" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                // SAFETY: `number` is a descriptor this program holds open.
                Ok(Object::Bool(unsafe { libc::isatty(number) } == 1))
            }
            // Whether reads answer straight away rather than waiting.
            "nonblock?" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                // SAFETY: `number` is a descriptor this program holds open.
                let flags = unsafe { libc::fcntl(number, libc::F_GETFL) };
                Ok(Object::Bool(flags >= 0 && flags & libc::O_NONBLOCK != 0))
            }
            "nonblock=" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                // SAFETY: `number` is a descriptor this program holds open.
                let flags = unsafe { libc::fcntl(number, libc::F_GETFL) };
                if flags < 0 {
                    return Err(stream_error(
                        &std::io::Error::last_os_error(),
                        "fcntl",
                        position,
                    ));
                }
                let wanted = if count != 0 {
                    flags | libc::O_NONBLOCK
                } else {
                    flags & !libc::O_NONBLOCK
                };
                // SAFETY: `number` is a descriptor this program holds open.
                unsafe { libc::fcntl(number, libc::F_SETFL, wanted) };
                Ok(Object::Nil)
            }
            // Whether the descriptor is handed to a child process.
            "close_on_exec?" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                // SAFETY: `number` is a descriptor this program holds open.
                let flags = unsafe { libc::fcntl(number, libc::F_GETFD) };
                Ok(Object::Bool(flags >= 0 && flags & libc::FD_CLOEXEC != 0))
            }
            "close_on_exec=" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                // SAFETY: `number` is a descriptor this program holds open.
                let flags = unsafe { libc::fcntl(number, libc::F_GETFD) };
                if flags < 0 {
                    return Err(stream_error(
                        &std::io::Error::last_os_error(),
                        "fcntl",
                        position,
                    ));
                }
                let wanted = if count != 0 {
                    flags | libc::FD_CLOEXEC
                } else {
                    flags & !libc::FD_CLOEXEC
                };
                // SAFETY: `number` is a descriptor this program holds open.
                unsafe { libc::fcntl(number, libc::F_SETFD, wanted) };
                Ok(Object::Nil)
            }
            // Whether the descriptor has something to read, or room to
            // write, right now. The count says which side is asked about.
            "ready?" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                let mut watched = libc::pollfd {
                    fd: number,
                    events: if count != 0 {
                        libc::POLLOUT
                    } else {
                        libc::POLLIN
                    },
                    revents: 0,
                };
                // SAFETY: `watched` is one entry, which is what the count
                // handed alongside it says.
                let answered = unsafe { libc::poll(&mut watched, 1, 0) };
                Ok(Object::Bool(answered > 0 && watched.revents != 0))
            }
            // Wait until the descriptor is ready, or until the wait runs out.
            // The count carries the milliseconds to wait, where a negative
            // count waits without limit.
            "wait" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                let mut watched = libc::pollfd {
                    fd: number,
                    events: match &*text {
                        "write" => libc::POLLOUT,
                        _ => libc::POLLIN,
                    },
                    revents: 0,
                };
                // SAFETY: `watched` is one entry, which is what the count
                // handed alongside it says.
                let answered = unsafe { libc::poll(&mut watched, 1, count as libc::c_int) };
                Ok(Object::Bool(answered > 0 && watched.revents != 0))
            }
            // The descriptor a path names, opened for reading or for writing.
            "open" => {
                // The count says how the file is opened: 0 reads, 1 writes
                // from the start, 2 adds to the end, and 3 does both.
                let opened = match count {
                    0 => std::fs::File::open(&text),
                    2 => std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(&text),
                    // Reading and writing leaves what the file already holds
                    // where it is, which is what `r+` asks for.
                    3 => std::fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .truncate(false)
                        .open(&text),
                    _ => std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&text),
                };
                match opened {
                    Ok(file) => {
                        // SAFETY: the descriptor came from the file just
                        // opened and is owned from here on.
                        let owned = unsafe { OwnedFd::from_raw_fd(file.into_raw_fd()) };
                        Ok(Object::Int(self.open_streams.keep(owned) as i64))
                    }
                    Err(problem) => Err(stream_error(&problem, &format!("open({text})"), position)),
                }
            }
            _ => Ok(Object::Nil),
        }
    }
}

impl VirtualMachine {
    /// Build the three streams the program starts with, over the descriptors
    /// the operating system opened for it. Done once the prelude has defined
    /// IO, since each one is an IO of its own.
    pub(crate) fn open_standard_streams(&mut self) {
        let Some(Object::Class(io_class)) = self.globals().get("IO") else {
            return;
        };
        for (number, named, constant) in [
            (0_i64, "stdin", "STDIN"),
            (1, "stdout", "STDOUT"),
            (2, "stderr", "STDERR"),
        ] {
            let built = self.send_to_object(
                Object::Class(std::rc::Rc::clone(&io_class)),
                "__standard__",
                vec![Object::Int(number), Object::string(named.to_string())],
                Position::new(0, 0, 0),
            );
            let Ok(stream) = built else {
                continue;
            };
            self.globals_mut().set(constant, stream.clone());
            self.globals_mut().set_variable(named, stream.clone());
            // The name is reached as a constant as well as through the
            // globals, so the scope the program runs in holds it too.
            self.environment_mut().define(constant.to_string(), stream);
        }
    }
}
