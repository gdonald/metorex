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
    /// Bytes an earlier read took from a stream and did not use. A pipe
    /// cannot be wound back, so what was read ahead is kept here for the
    /// next read to hand over first.
    waiting: HashMap<u64, Vec<u8>>,
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

    /// Read from a stream, handing over first whatever an earlier read took
    /// and did not use. The answer is what `read` would have answered, with a
    /// negative number leaving errno as `read` set it.
    fn take(&mut self, handle: u64, number: RawFd, buffer: &mut [u8]) -> isize {
        if let Some(held) = self.waiting.get_mut(&handle)
            && !held.is_empty()
        {
            let taken = held.len().min(buffer.len());
            buffer[..taken].copy_from_slice(&held[..taken]);
            held.drain(..taken);
            return taken as isize;
        }
        // SAFETY: `buffer` names a run of its own length this call only
        // writes into.
        unsafe {
            libc::read(
                number,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
            )
        }
    }

    /// Hand bytes back to a stream. A stream that can be wound back is wound
    /// back, so where it stands is what it reports. A pipe cannot be, so what
    /// was read ahead is kept for the next read instead.
    fn put_back(&mut self, handle: u64, number: RawFd, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        if self.waiting.get(&handle).is_none_or(|held| held.is_empty()) {
            // SAFETY: `number` is a descriptor this program holds open.
            let wound = unsafe { libc::lseek(number, -(bytes.len() as i64), libc::SEEK_CUR) };
            if wound >= 0 {
                return;
            }
        }
        let held = self.waiting.entry(handle).or_default();
        let mut carried = bytes.to_vec();
        carried.extend_from_slice(held);
        *held = carried;
    }

    /// Forget what was read ahead, which a stream moved by hand or closed no
    /// longer stands behind.
    fn forget_read_ahead(&mut self, handle: u64) {
        self.waiting.remove(&handle);
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

/// How a file is opened the way the count says: 0 reads, 1 writes from the
/// start, 2 adds to the end, 3 reads and writes what is already there, 4 reads
/// and writes from the start, and 5 reads and adds to the end.
fn options_for(count: i64) -> std::fs::OpenOptions {
    let mut options = std::fs::OpenOptions::new();
    match count {
        0 => options.read(true),
        2 => options.append(true).create(true),
        // Reading and writing leaves what the file already holds where it
        // is, which is what `r+` asks for. A file that is not there is not
        // brought into being: `r+` reads as well as writes.
        3 => options.read(true).write(true).truncate(false),
        4 => options.read(true).write(true).create(true).truncate(true),
        5 => options.read(true).append(true).create(true),
        _ => options.write(true).create(true).truncate(true),
    };
    options
}

fn open_for(path: &str, count: i64) -> std::io::Result<std::fs::File> {
    options_for(count).open(path)
}

/// What the operating system calls this failure, without the number it
/// carries: `Bad file descriptor` rather than `Bad file descriptor (os error
/// 9)`.
fn strerror_text(problem: &std::io::Error) -> String {
    let spelled = problem.to_string();
    match spelled.find(" (os error") {
        Some(at) => spelled[..at].to_string(),
        None => spelled,
    }
}

/// The Errno class a failure belongs to.
fn errno_class(problem: &std::io::Error) -> &'static str {
    match problem.raw_os_error() {
        Some(libc::EPIPE) => "Errno::EPIPE",
        Some(libc::EBADF) => "Errno::EBADF",
        Some(libc::EAGAIN) => "Errno::EAGAIN",
        Some(libc::ENOTTY) => "Errno::ENOTTY",
        Some(libc::ENXIO) => "Errno::ENXIO",
        Some(libc::ENODEV) => "Errno::ENODEV",
        Some(libc::ESPIPE) => "Errno::ESPIPE",
        Some(libc::EPERM) => "Errno::EPERM",
        Some(libc::EINVAL) => "Errno::EINVAL",
        Some(libc::ENOENT) => "Errno::ENOENT",
        Some(libc::EACCES) => "Errno::EACCES",
        Some(libc::EISDIR) => "Errno::EISDIR",
        Some(libc::ENOTDIR) => "Errno::ENOTDIR",
        Some(libc::EEXIST) => "Errno::EEXIST",
        Some(libc::ELOOP) => "Errno::ELOOP",
        Some(libc::ENAMETOOLONG) => "Errno::ENAMETOOLONG",
        _ => "IOError",
    }
}

/// The error a stream operation reports, named the way the operating system
/// names it.
fn stream_error(problem: &std::io::Error, what: &str, position: Position) -> MetorexError {
    let named = errno_class(problem);
    crate::vm::errors::simple_exception(named, &format!("{what}: {problem}"), position)
}

/// Wait for a descriptor to have something to read, or to reach its end,
/// until the deadline. Answers whether it did.
fn wait_until_readable(number: RawFd) -> bool {
    let mut watched = libc::pollfd {
        fd: number,
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: `watched` is one pollfd, which is the count passed, and a
    // negative timeout waits for as long as it takes.
    let ready = unsafe { libc::poll(&mut watched, 1, -1) };
    ready > 0 || std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted
}

fn closed_error(position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("IOError", "closed stream", position)
}

impl VirtualMachine {
    /// Whether a broken pipe is the operating system's to answer for, which
    /// is what `Signal.trap('PIPE', 'SYSTEM_DEFAULT')` asks for.
    fn leaves_a_broken_pipe_to_the_system(&self) -> bool {
        matches!(
            self.signal_handlers.get("PIPE"),
            Some(Object::String(held)) if *held.as_str() == *"SYSTEM_DEFAULT"
        )
    }

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
        // Where the text names a path, a binary one names the file its bytes
        // spell.
        let path = match arguments.get(2) {
            Some(Object::String(held)) => super::file_methods::path_text(held),
            _ => String::new(),
        };
        // A separator whose characters stand for bytes names those bytes
        // rather than the ones its text is spelled with.
        let text_bytes = match arguments.get(2) {
            Some(Object::String(held)) if held.holds_bytes() => {
                crate::vm::native_methods::string_methods::binary_bytes(held)
            }
            _ => text.as_bytes().to_vec(),
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
                // than waiting, which is what `nonblock?` reports for them,
                // and that a program it starts does not inherit.
                for end in [reader.as_raw_fd(), writer.as_raw_fd()] {
                    // SAFETY: both descriptors came from `pipe` just above.
                    unsafe {
                        let flags = libc::fcntl(end, libc::F_GETFL);
                        libc::fcntl(end, libc::F_SETFL, flags | libc::O_NONBLOCK);
                        libc::fcntl(end, libc::F_SETFD, libc::FD_CLOEXEC);
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
            // A lock on the whole file, which another process asking for one
            // waits on or is refused.
            "flock" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                // SAFETY: `number` is a descriptor this program holds open.
                let held = unsafe { libc::flock(number, count as libc::c_int) };
                if held < 0 {
                    let failure = std::io::Error::last_os_error();
                    // A lock asked for without waiting is refused rather than
                    // raising, which Ruby reports as false.
                    if failure.raw_os_error() == Some(libc::EWOULDBLOCK) {
                        return Ok(Object::Bool(false));
                    }
                    return Err(stream_error(&failure, "flock", position));
                }
                Ok(Object::Int(0))
            }
            // `fcntl` asks the operating system about a descriptor, or sets
            // one of the flags it keeps.
            // Ask the device behind the descriptor to do something, with a
            // buffer it reads from and writes back into. The filled buffer
            // is answered beside what the call returned.
            "ioctl" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                let request = count as libc::c_ulong;
                let mut buffer = crate::vm::native_methods::pack_format::string_to_bytes(&text);
                // A request that names no buffer carries a number instead,
                // which is what the caller sent in place of one.
                let answered = if buffer.is_empty() {
                    let held = match arguments.get(4) {
                        Some(Object::Int(held)) => *held as libc::c_int,
                        _ => 0,
                    };
                    // SAFETY: the descriptor is one this program holds open.
                    unsafe { libc::ioctl(number, request, held) }
                } else {
                    // SAFETY: the buffer outlives the call and the request
                    // decides how much of it is read and written.
                    unsafe { libc::ioctl(number, request, buffer.as_mut_ptr()) }
                };
                if answered < 0 {
                    return Err(stream_error(
                        &std::io::Error::last_os_error(),
                        "ioctl",
                        position,
                    ));
                }
                Ok(Object::array(vec![
                    Object::Int(i64::from(answered)),
                    crate::vm::native_methods::pack_format::bytes_to_string(&buffer),
                ]))
            }
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
                self.open_streams.forget_read_ahead(handle);
                Ok(Object::Nil)
            }
            "write" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                let bytes = text_bytes.clone();
                // SAFETY: `bytes` names a run this call only reads from.
                let written = unsafe {
                    libc::write(number, bytes.as_ptr() as *const libc::c_void, bytes.len())
                };
                if written < 0 {
                    let trouble = std::io::Error::last_os_error();
                    // Writing where nothing is left to read is a signal, and
                    // a program that leaves that signal to the operating
                    // system ends by it rather than hearing about it.
                    if trouble.raw_os_error() == Some(libc::EPIPE)
                        && self.leaves_a_broken_pipe_to_the_system()
                    {
                        crate::vm::native_functions::die_of_a_broken_pipe();
                    }
                    return Err(stream_error(&trouble, "write", position));
                }
                Ok(Object::Int(written as i64))
            }
            "read" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                let wanted = if count > 0 { count as usize } else { 65536 };
                let mut buffer = vec![0u8; wanted];
                // A stream with nothing to hand over yet is waited on, and
                // waiting is where every other thread gets its turn.
                let read = loop {
                    // A descriptor that blocks would hold every thread up
                    // until it had something, so it is asked first whether it
                    // has, and the other threads run while it has not.
                    if self.other_threads_are_waiting() && !descriptor_is_ready(number) {
                        self.wait_for_other_threads(position);
                        if self.open_streams.number_of(handle).is_none() {
                            return Err(crate::vm::errors::simple_exception(
                                "IOError",
                                "stream closed in another thread",
                                position,
                            ));
                        }
                        continue;
                    }
                    let held = self
                        .open_streams
                        .take(handle, number, &mut buffer[..wanted]);
                    if held >= 0 {
                        break held;
                    }
                    let problem = std::io::Error::last_os_error();
                    // A stream another thread closed while this one waited is
                    // closed, whatever the operating system says about the
                    // descriptor it left behind.
                    if problem.raw_os_error() == Some(libc::EBADF)
                        || self.open_streams.number_of(handle).is_none()
                    {
                        return Err(crate::vm::errors::simple_exception(
                            "IOError",
                            "stream closed in another thread",
                            position,
                        ));
                    }
                    if problem.raw_os_error() != Some(libc::EAGAIN) {
                        return Err(stream_error(&problem, "read", position));
                    }
                    if self.other_threads_are_waiting() {
                        self.wait_for_other_threads(position);
                        if self.open_streams.number_of(handle).is_none() {
                            return Err(crate::vm::errors::simple_exception(
                                "IOError",
                                "stream closed in another thread",
                                position,
                            ));
                        }
                        continue;
                    }
                    // Another process may still write, such as a child, so
                    // the descriptor is waited on until it has something. A
                    // signal that ends the wait early is handled first.
                    if !wait_until_readable(number) {
                        return Err(stream_error(&problem, "read", position));
                    }
                    self.deliver_pending_signals(position)?;
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
                let opened = self.open_taking_turns(&path, options_for(count), position);
                match opened {
                    Ok(file) => {
                        // SAFETY: the descriptor came from the file just
                        // opened and is owned from here on.
                        let owned = unsafe { OwnedFd::from_raw_fd(file.into_raw_fd()) };
                        Ok(Object::Int(self.open_streams.keep(owned) as i64))
                    }
                    Err(problem) => Err(stream_error(&problem, &format!("open({path})"), position)),
                }
            }
            // A file opened with the flags a program named as a number. The
            // file was already brought into being or refused for existing,
            // so O_EXCL is not asked again.
            "open_flags" => {
                use std::os::unix::fs::OpenOptionsExt as _;
                let flags = count as libc::c_int;
                let mut options = std::fs::OpenOptions::new();
                match flags & libc::O_ACCMODE {
                    held if held == libc::O_WRONLY => options.write(true),
                    held if held == libc::O_RDWR => options.read(true).write(true),
                    _ => options.read(true),
                };
                options.custom_flags(flags & !libc::O_ACCMODE & !libc::O_EXCL & !libc::O_CREAT);
                match self.open_taking_turns(&path, options, position) {
                    Ok(file) => {
                        // SAFETY: the descriptor came from the file just
                        // opened and is owned from here on.
                        let owned = unsafe { OwnedFd::from_raw_fd(file.into_raw_fd()) };
                        Ok(Object::Int(self.open_streams.keep(owned) as i64))
                    }
                    Err(problem) => Err(stream_error(&problem, &format!("open({path})"), position)),
                }
            }
            // The descriptor a path names, handed back by number rather
            // than under a handle. Whoever asked for it owns it from here,
            // which is what `IO.sysopen` promises.
            "sysopen" => {
                let opened = open_for(&path, count);
                match opened {
                    Ok(file) => Ok(Object::Int(i64::from(file.into_raw_fd()))),
                    Err(problem) => Err(stream_error(&problem, &format!("open({path})"), position)),
                }
            }
            // One line: everything up to and including the separator the
            // text names, or the whole of what is left where it names
            // nothing. A count above zero stops the line at that many
            // bytes, and the fifth argument, when it is 1, leaves the
            // newlines before a paragraph behind.
            "readline" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                // The separator is text the program wrote, so it stands for
                // the bytes that text is spelled with.
                let separator = text_bytes.clone();
                // The fifth argument carries how a paragraph is read: bit
                // one steps over the newlines standing before it, and bit
                // two steps over the ones standing after it.
                let mode = match arguments.get(4) {
                    Some(Object::Int(held)) => *held,
                    _ => 0,
                };
                let skipping = mode & 1 != 0;
                let paragraph = mode & 2 != 0;
                let limit = if count > 0 {
                    Some(count as usize)
                } else {
                    None
                };
                let mut collected: Vec<u8> = Vec::new();
                let mut buffer = [0u8; 4096];
                // The newlines standing before a paragraph belong to the
                // one already read, so they are stepped over here.
                if skipping {
                    loop {
                        let read = self.open_streams.take(handle, number, &mut buffer[..1]);
                        if read <= 0 {
                            break;
                        }
                        if buffer[0] != b'\n' {
                            self.open_streams.put_back(handle, number, &buffer[..1]);
                            break;
                        }
                    }
                }
                loop {
                    let mut wanted = buffer.len();
                    if let Some(limit) = limit {
                        wanted = wanted.min(limit.saturating_sub(collected.len()));
                    }
                    if wanted == 0 {
                        break;
                    }
                    let read = self
                        .open_streams
                        .take(handle, number, &mut buffer[..wanted]);
                    if read < 0 {
                        let problem = std::io::Error::last_os_error();
                        if problem.raw_os_error() == Some(libc::EAGAIN) {
                            // Nothing to read yet. A line is worth waiting
                            // for, and waiting is where every other thread
                            // gets its turn.
                            if self.other_threads_are_waiting() {
                                self.wait_for_other_threads(position);
                                if self.open_streams.number_of(handle).is_none() {
                                    break;
                                }
                                continue;
                            }
                            // Another process may still write, such as a
                            // forked child, so the descriptor is waited on
                            // until it has something.
                            if wait_until_readable(number) {
                                self.deliver_pending_signals(position)?;
                                continue;
                            }
                            break;
                        }
                        return Err(stream_error(&problem, "read", position));
                    }
                    if read == 0 {
                        break;
                    }
                    let had = collected.len();
                    collected.extend_from_slice(&buffer[..read as usize]);
                    if !separator.is_empty() {
                        let from = had.saturating_sub(separator.len() - 1);
                        if let Some(at) = collected[from..]
                            .windows(separator.len())
                            .position(|run| run == separator.as_slice())
                        {
                            let ends = from + at + separator.len();
                            let mut extra: Vec<u8> = collected[ends..].to_vec();
                            // The newlines standing after a paragraph belong
                            // to it, so the next read starts at the line
                            // opening the paragraph that follows.
                            if paragraph {
                                let kept = extra.iter().take_while(|held| **held == b'\n').count();
                                extra.drain(..kept);
                                while extra.is_empty() {
                                    let read =
                                        self.open_streams.take(handle, number, &mut buffer[..1]);
                                    if read <= 0 {
                                        break;
                                    }
                                    if buffer[0] != b'\n' {
                                        extra.push(buffer[0]);
                                        break;
                                    }
                                }
                            }
                            self.open_streams.put_back(handle, number, &extra);
                            collected.truncate(ends);
                            break;
                        }
                    }
                    if let Some(limit) = limit
                        && collected.len() >= limit
                    {
                        break;
                    }
                }
                // A count stops the line at a whole character rather than
                // in the middle of one, so the bytes finishing the last
                // character are read too.
                // A limit that cuts a character short is read past, up to the
                // longest run Ruby reads before it gives the character up.
                const EXTRA_LIMIT: usize = 16;
                let mut extra_read = 0;
                while limit.is_some()
                    && extra_read < EXTRA_LIMIT
                    && std::str::from_utf8(&collected).is_err()
                {
                    let read = self.open_streams.take(handle, number, &mut buffer[..1]);
                    if read <= 0 {
                        break;
                    }
                    collected.push(buffer[0]);
                    extra_read += 1;
                }
                // Text that reads as UTF-8 is handed back as text, and
                // anything else byte by byte.
                match String::from_utf8(collected.clone()) {
                    Ok(text) => Ok(Object::string(text)),
                    Err(_) => Ok(super::pack_format::bytes_to_string(&collected)),
                }
            }
            // One character: the byte at the cursor and the bytes that
            // finish the character it starts.
            "getc" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                let mut collected: Vec<u8> = Vec::new();
                let mut byte = [0u8; 1];
                let read = self.open_streams.take(handle, number, &mut byte);
                if read <= 0 {
                    return Ok(Object::Nil);
                }
                collected.push(byte[0]);
                // How many bytes follow the one that opens a character, which
                // the encoding the stream reads in settles.
                let following = match text.as_str() {
                    "EUC-JP" => match byte[0] {
                        0x8f => 2,
                        0xa1..=0xfe => 1,
                        _ => 0,
                    },
                    "UTF-8" | "" => match byte[0] {
                        0xC0..=0xDF => 1,
                        0xE0..=0xEF => 2,
                        0xF0..=0xF7 => 3,
                        _ => 0,
                    },
                    _ => 0,
                };
                for _ in 0..following {
                    let read = self.open_streams.take(handle, number, &mut byte);
                    if read <= 0 {
                        break;
                    }
                    collected.push(byte[0]);
                }
                match String::from_utf8(collected.clone()) {
                    Ok(text) => Ok(Object::string(text)),
                    Err(_) => Ok(super::pack_format::bytes_to_string(&collected)),
                }
            }
            // The directory a descriptor names, made the one the program
            // works from.
            "fchdir" => {
                // SAFETY: the count is a descriptor number the program was
                // handed, and `fchdir` only reads it.
                if unsafe { libc::fchdir(count as libc::c_int) } < 0 {
                    let problem = std::io::Error::last_os_error();
                    let named = errno_class(&problem);
                    return Err(crate::vm::errors::simple_exception(
                        named,
                        &format!("{} - fchdir", strerror_text(&problem)),
                        position,
                    ));
                }
                Ok(Object::Int(0))
            }
            // Where the descriptor stands. The count is the offset and the
            // text says what it is measured from.
            "seek" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                let whence = match &*text {
                    "cur" => libc::SEEK_CUR,
                    "end" => libc::SEEK_END,
                    _ => libc::SEEK_SET,
                };
                // SAFETY: `number` is a descriptor this program holds open.
                self.open_streams.forget_read_ahead(handle);
                let moved = unsafe { libc::lseek(number, count as libc::off_t, whence) };
                if moved < 0 {
                    return Err(stream_error(
                        &std::io::Error::last_os_error(),
                        "seek",
                        position,
                    ));
                }
                Ok(Object::Int(moved as i64))
            }
            // How many bytes the descriptor stands over.
            "size" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                // SAFETY: `held` is written by `fstat` before it is read,
                // and `number` is a descriptor this program holds open.
                let counted = unsafe {
                    let mut held: libc::stat = std::mem::zeroed();
                    if libc::fstat(number, &mut held) < 0 {
                        return Err(stream_error(
                            &std::io::Error::last_os_error(),
                            "fstat",
                            position,
                        ));
                    }
                    held.st_size
                };
                Ok(Object::Int(counted as i64))
            }
            // The file cut down to, or filled out to, the count of bytes.
            "truncate" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                // SAFETY: `number` is a descriptor this program holds open.
                if unsafe { libc::ftruncate(number, count as libc::off_t) } < 0 {
                    return Err(stream_error(
                        &std::io::Error::last_os_error(),
                        "truncate",
                        position,
                    ));
                }
                Ok(Object::Int(0))
            }
            // How the descriptor was opened: 0 reads, 1 writes, 2 does both.
            // A number the operating system holds nothing under is refused
            // the way every other question about it is.
            "accmode" => {
                let Some(number) = self.open_streams.number_of(handle) else {
                    return Err(closed_error(position));
                };
                // SAFETY: asking about a descriptor reads nothing and writes
                // nothing, whether or not it is still open.
                let flags = unsafe { libc::fcntl(number, libc::F_GETFL) };
                if flags < 0 {
                    return Err(stream_error(
                        &std::io::Error::last_os_error(),
                        "fcntl",
                        position,
                    ));
                }
                Ok(Object::Int(i64::from(flags & libc::O_ACCMODE)))
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
            // Each is a constant on Object, which is what makes naming it
            // again warn that it was already set.
            if let Some(Object::Class(object_class)) = self.globals().get("Object") {
                object_class.set_class_var(constant, stream.clone());
            }
            // `$>` is where a program writes without naming a stream, which
            // is standard output under the name Ruby's punctuation gives it.
            if named == "stdout" {
                self.globals_mut().set_variable(">", stream);
            }
        }
    }
}

impl VirtualMachine {
    /// Open a path, letting the other threads run while the open waits. A
    /// FIFO's open waits for a process to open its other end, which may be
    /// another thread of this one, so the open is made on a thread of its own
    /// while this one hands over its turn. The descriptor then answers
    /// straight away when there is nothing to read, so a read hands over its
    /// turn the same way rather than holding up the whole program.
    fn open_taking_turns(
        &mut self,
        path: &str,
        options: std::fs::OpenOptions,
        position: Position,
    ) -> std::io::Result<std::fs::File> {
        use std::os::unix::fs::FileTypeExt as _;
        let is_fifo = std::fs::metadata(path)
            .map(|found| found.file_type().is_fifo())
            .unwrap_or(false);
        if !is_fifo || !self.other_threads_are_waiting() {
            return options.open(path);
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        let named = path.to_string();
        std::thread::spawn(move || {
            let _ = sender.send(options.open(&named));
        });
        loop {
            match receiver.try_recv() {
                Ok(opened) => {
                    if let Ok(file) = &opened {
                        let number = file.as_raw_fd();
                        // SAFETY: the descriptor came from the file just
                        // opened, which is still held.
                        unsafe {
                            let flags = libc::fcntl(number, libc::F_GETFL);
                            libc::fcntl(number, libc::F_SETFL, flags | libc::O_NONBLOCK);
                        }
                    }
                    return opened;
                }
                // The thread making the open sends before it ends, so the
                // only thing to wait for is that send.
                Err(_) => {
                    if self.other_threads_are_waiting() {
                        self.wait_for_other_threads(position);
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(2));
                    }
                }
            }
        }
    }
}

/// Whether a read from a descriptor would answer straight away: it has bytes,
/// has reached its end, or has something wrong with it to report.
fn descriptor_is_ready(number: i32) -> bool {
    let mut asked = libc::pollfd {
        fd: number,
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: `poll` reads and writes the one entry given, and a zero timeout
    // answers without waiting.
    unsafe { libc::poll(&mut asked, 1, 0) != 0 }
}
