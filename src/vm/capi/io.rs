//! IO from C: the `rb_io_t` C reads an IO through, writing to an IO,
//! asking about its descriptor, and waiting until a descriptor is ready
//! while the other threads take turns.

use super::calls::call;
use super::handles::{QFALSE, QNIL, QTRUE, Value, objects_from, to_object, to_value};
use super::{called_from, interpreter, keeping_call_state, raise};
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_char;
use std::time::{Duration, Instant};

const FMODE_READABLE: i32 = 0x01;
const FMODE_WRITABLE: i32 = 0x02;
const FMODE_BINMODE: i32 = 0x04;
const FMODE_SYNC: i32 = 0x08;

const IO_READABLE: i32 = 1;
const IO_PRIORITY: i32 = 2;
const IO_WRITABLE: i32 = 4;

/// What C reads through an `rb_io_t *`.
#[repr(C)]
pub struct RbIo {
    io: Value,
    fd: i32,
    mode: i32,
    path: Value,
}

/// What C hands `rb_thread_fd_select` for each set of descriptors.
#[repr(C)]
pub struct RbFdset {
    max: i32,
    set: *mut libc::fd_set,
}

thread_local! {
    /// The `rb_io_t` handed to C for each IO.
    static HANDED: RefCell<HashMap<Value, Box<RbIo>>> = RefCell::new(HashMap::new());
}

fn io_error(message: &str) -> ! {
    raise(crate::vm::errors::simple_exception(
        "IOError",
        message,
        called_from(),
    ))
}

fn send(io: Value, name: &str, arguments: Vec<Object>) -> Value {
    to_value(&call(to_object(io), name, arguments))
}

fn is_closed(io: Value) -> bool {
    call(to_object(io), "closed?", Vec::new()).is_truthy()
}

/// The descriptor `io` holds, raising the IOError MRI raises for a closed
/// stream or for one never opened.
fn descriptor(io: Value) -> i32 {
    if is_closed(io) {
        io_error("closed stream");
    }
    let answered = super::control::caught(|| call(to_object(io), "fileno", Vec::new()));
    match answered {
        Ok(Object::Int(number)) => number as i32,
        _ => io_error("uninitialized stream"),
    }
}

/// The access mode the operating system holds `fd` open with, as FMODE
/// flags.
fn access_flags(fd: i32) -> i32 {
    // SAFETY: F_GETFL reads the flags of a descriptor and changes nothing.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    match flags & libc::O_ACCMODE {
        libc::O_RDONLY => FMODE_READABLE,
        libc::O_WRONLY => FMODE_WRITABLE,
        _ => FMODE_READABLE | FMODE_WRITABLE,
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_descriptor(io: Value) -> i32 {
    descriptor(io)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_mode(io: Value) -> i32 {
    let mut mode = access_flags(descriptor(io));
    if call(to_object(io), "binmode?", Vec::new()).is_truthy() {
        mode |= FMODE_BINMODE;
    }
    if call(to_object(io), "sync", Vec::new()).is_truthy() {
        mode |= FMODE_SYNC;
    }
    mode
}

/// The `rb_io_t` for `io`, filled from it as it stands. A closed IO gets
/// one with no descriptor, which `rb_io_check_closed` refuses.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_io_struct(io: Value) -> *mut RbIo {
    let (fd, mode) = if is_closed(io) {
        (-1, 0)
    } else {
        (descriptor(io), rb_io_mode(io))
    };
    let path = send(io, "path", Vec::new());
    HANDED.with(|handed| {
        let mut handed = handed.borrow_mut();
        let held = handed
            .entry(io)
            .or_insert_with(|| Box::new(RbIo { io, fd, mode, path }));
        held.fd = fd;
        held.mode = mode;
        held.path = path;
        &mut **held as *mut RbIo
    })
}

fn handed_io(fptr: *mut RbIo) -> Value {
    // SAFETY: C hands back a pointer `rb_metorex_io_struct` answered, which
    // stays where it is for the life of the program.
    unsafe { (*fptr).io }
}

/// `io`, refusing a frozen one as MRI does before it touches an IO.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_taint_check(io: Value) -> Value {
    super::objects::rb_check_frozen(io);
    io
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_check_closed(fptr: *mut RbIo) {
    if is_closed(handed_io(fptr)) {
        io_error("closed stream");
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_check_readable(fptr: *mut RbIo) {
    rb_io_check_closed(fptr);
    if rb_io_mode(handed_io(fptr)) & FMODE_READABLE == 0 {
        io_error("not opened for reading");
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_check_writable(fptr: *mut RbIo) {
    rb_io_check_closed(fptr);
    if rb_io_mode(handed_io(fptr)) & FMODE_WRITABLE == 0 {
        io_error("not opened for writing");
    }
}

fn add_descriptor_flag(fd: i32, get: i32, set: i32, flag: i32) {
    // SAFETY: reading and writing back a descriptor's flags changes only
    // the one flag asked for.
    unsafe {
        let flags = libc::fcntl(fd, get);
        libc::fcntl(fd, set, flags | flag);
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_set_nonblock(fptr: *mut RbIo) {
    let fd = descriptor(handed_io(fptr));
    add_descriptor_flag(fd, libc::F_GETFL, libc::F_SETFL, libc::O_NONBLOCK);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_fd_fix_cloexec(fd: i32) {
    add_descriptor_flag(fd, libc::F_GETFD, libc::F_SETFD, libc::FD_CLOEXEC);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_cloexec_open(
    path: *const c_char,
    flags: i32,
    mode: libc::mode_t,
) -> i32 {
    // SAFETY: C hands over a NUL-terminated path.
    unsafe { libc::open(path, flags | libc::O_CLOEXEC, libc::c_uint::from(mode)) }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_cloexec_fcntl_dupfd(fd: i32, minimum: i32) -> i32 {
    // SAFETY: F_DUPFD_CLOEXEC makes a new descriptor and changes nothing else.
    unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, minimum) }
}

/// A copy of `fd` marked close-on-exec, above the standard streams.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_cloexec_dup(fd: i32) -> i32 {
    rb_cloexec_fcntl_dupfd(fd, 3)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_path(io: Value) -> Value {
    send(io, "path", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_closed_p(io: Value) -> Value {
    if is_closed(io) { QTRUE } else { QFALSE }
}

/// `io` itself when it is an IO, what its `to_io` answers, or nil.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_check_io(io: Value) -> Value {
    const T_FILE: i32 = 0x0b;
    let type_name = c"IO";
    let method = c"to_io";
    super::objects::rb_check_convert_type(io, T_FILE, type_name.as_ptr(), method.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_addstr(io: Value, string: Value) -> Value {
    send(io, "<<", vec![to_object(string)]);
    io
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_printf(count: i32, values: *const Value, io: Value) -> Value {
    send(io, "printf", objects_from(i64::from(count), values))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_print(count: i32, values: *const Value, io: Value) -> Value {
    send(io, "print", objects_from(i64::from(count), values))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_puts(count: i32, values: *const Value, io: Value) -> Value {
    send(io, "puts", objects_from(i64::from(count), values))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_write(io: Value, string: Value) -> Value {
    send(io, "write", vec![to_object(string)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_close(io: Value) -> Value {
    send(io, "close", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_binmode(io: Value) -> Value {
    send(io, "binmode", Vec::new())
}

/// For each IO event, the `poll` event that asks for it and the `poll`
/// events that report it, as MRI reads them: a hang-up counts as readable,
/// and an error as readable and writable.
const EVENT_SETS: [(i32, i16, i16); 3] = [
    (
        IO_READABLE,
        libc::POLLIN,
        libc::POLLIN | libc::POLLRDNORM | libc::POLLRDBAND | libc::POLLHUP | libc::POLLERR,
    ),
    (IO_PRIORITY, libc::POLLPRI, libc::POLLPRI),
    (
        IO_WRITABLE,
        libc::POLLOUT,
        libc::POLLOUT | libc::POLLWRNORM | libc::POLLWRBAND | libc::POLLERR,
    ),
];

/// The `poll` events that ask for the IO events in `events`.
fn poll_events(events: i32) -> i16 {
    EVENT_SETS
        .iter()
        .filter(|(event, _, _)| events & event != 0)
        .fold(0, |asked, (_, polled, _)| asked | polled)
}

/// The IO events of those `asked` that the `poll` events in `ready` report.
fn io_events(ready: i16, asked: i32) -> i32 {
    EVENT_SETS
        .iter()
        .filter(|(_, _, reported)| ready & reported != 0)
        .fold(0, |events, (event, _, _)| events | event)
        & asked
}

/// Asks the operating system once, without waiting, which of `waits`
/// are ready, as pairs of descriptor and IO events asked for, and answers
/// the IO events ready for each.
fn ready_now(waits: &[(i32, i32)]) -> Vec<i32> {
    let mut asked: Vec<libc::pollfd> = waits
        .iter()
        .map(|&(fd, events)| libc::pollfd {
            fd,
            events: poll_events(events),
            revents: 0,
        })
        .collect();
    // SAFETY: `asked` holds one pollfd for each of its entries.
    unsafe { libc::poll(asked.as_mut_ptr(), asked.len() as libc::nfds_t, 0) };
    asked
        .iter()
        .zip(waits)
        .map(|(polled, &(_, events))| io_events(polled.revents, events))
        .collect()
}

/// Waits until one of `waits` is ready or `timeout` passes, with the
/// other threads taking turns meanwhile and a kill of this thread ending
/// the wait. Answers the IO events ready for each, all zero after a
/// timeout.
fn wait_until_ready(waits: &[(i32, i32)], timeout: Option<Duration>) -> Vec<i32> {
    let deadline = timeout.map(|length| Instant::now() + length);
    let position = called_from();
    loop {
        let ready = ready_now(waits);
        if ready.iter().any(|&events| events != 0)
            || deadline.is_some_and(|deadline| Instant::now() >= deadline)
        {
            return ready;
        }
        keeping_call_state(|| interpreter().wait_for_other_threads(position));
        if let Err(stopped) = interpreter().raise_if_thread_killed(position) {
            raise(stopped);
        }
    }
}

/// The length a timeout VALUE stands for, or None for nil.
fn timeout_of(timeout: Value) -> Option<Duration> {
    (timeout != QNIL).then(|| Duration::from_secs_f64(super::bignums::rb_num2dbl(timeout).max(0.0)))
}

/// Waits until `io` is ready for `events`, answering the events ready, or
/// false when `timeout` passes first.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_wait(io: Value, events: Value, timeout: Value) -> Value {
    let fd = descriptor(io);
    let asked = super::numbers::rb_num2int(events) as i32;
    let ready = wait_until_ready(&[(fd, asked)], timeout_of(timeout))[0];
    if ready == 0 {
        QFALSE
    } else {
        to_value(&Object::Int(i64::from(ready)))
    }
}

/// What to do after an operation on `io` failed with `error`: answer the
/// events again after an interruption, wait for them when the operation
/// would have blocked, and answer nil for any other error.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_maybe_wait(
    error: i32,
    io: Value,
    events: Value,
    timeout: Value,
) -> Value {
    descriptor(io);
    match error {
        libc::EINTR => events,
        libc::EAGAIN => rb_io_wait(io, events, timeout),
        _ => QNIL,
    }
}

/// `rb_io_maybe_wait` for one event, raising IO::TimeoutError when the
/// timeout passes.
fn maybe_wait_for(error: i32, io: Value, event: i32, timeout: Value, named: &str) -> i32 {
    let answered = rb_io_maybe_wait(error, io, to_value(&Object::Int(i64::from(event))), timeout);
    match answered {
        QNIL => 0,
        QFALSE => raise(crate::vm::errors::simple_exception(
            "IO::TimeoutError",
            &format!("Timed out waiting for IO to become {}!", named),
            called_from(),
        )),
        ready => super::numbers::rb_num2int(ready) as i32,
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_maybe_wait_readable(error: i32, io: Value, timeout: Value) -> i32 {
    maybe_wait_for(error, io, IO_READABLE, timeout, "readable")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_maybe_wait_writable(error: i32, io: Value, timeout: Value) -> i32 {
    maybe_wait_for(error, io, IO_WRITABLE, timeout, "writable")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_thread_wait_fd(fd: i32) -> i32 {
    wait_until_ready(&[(fd, IO_READABLE)], None);
    1
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_thread_fd_writable(fd: i32) -> i32 {
    wait_until_ready(&[(fd, IO_WRITABLE)], None);
    1
}

/// The descriptors below `count` in `set`, or none for a NULL set.
fn members(set: *mut RbFdset, count: i32) -> Vec<i32> {
    if set.is_null() {
        return Vec::new();
    }
    // SAFETY: C hands over a set `rb_fd_init` made, holding an fd_set.
    let fds = unsafe { (*set).set };
    // SAFETY: `fds` is an fd_set, and each descriptor checked is below
    // the count C gave, which is within it.
    (0..count)
        .filter(|&fd| unsafe { libc::FD_ISSET(fd, fds) })
        .collect()
}

/// Leaves only `ready` in `set`.
fn keep_only(set: *mut RbFdset, ready: &[i32]) {
    if set.is_null() {
        return;
    }
    // SAFETY: as in `members`.
    let fds = unsafe { (*set).set };
    // SAFETY: `fds` is an fd_set C made, and each descriptor set was in it.
    unsafe {
        libc::FD_ZERO(fds);
        for &fd in ready {
            libc::FD_SET(fd, fds);
        }
    }
}

/// Waits until a descriptor in one of the sets is ready, leaving in each
/// set only the ones that are, and answers how many are.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_thread_fd_select(
    count: i32,
    read: *mut RbFdset,
    write: *mut RbFdset,
    except: *mut RbFdset,
    timeout: *const libc::timeval,
) -> i32 {
    let sets = [
        (read, IO_READABLE),
        (write, IO_WRITABLE),
        (except, IO_PRIORITY),
    ];
    let waits: Vec<(i32, i32)> = sets
        .iter()
        .flat_map(|&(set, event)| members(set, count).into_iter().map(move |fd| (fd, event)))
        .collect();
    let timeout = (!timeout.is_null()).then(|| {
        // SAFETY: C hands over a timeval or NULL.
        let held = unsafe { *timeout };
        Duration::from_secs(held.tv_sec.max(0) as u64)
            + Duration::from_micros(held.tv_usec.max(0) as u64)
    });
    let ready = wait_until_ready(&waits, timeout);
    for (set, event) in sets {
        let in_set: Vec<i32> = waits
            .iter()
            .zip(&ready)
            .filter(|((_, asked), events)| *asked == event && **events != 0)
            .map(|((fd, _), _)| *fd)
            .collect();
        keep_only(set, &in_set);
    }
    ready.iter().filter(|&&events| events != 0).count() as i32
}

/// The mode string `IO.for_fd` takes for `descriptor`: the access the
/// operating system holds it open with, whatever `mode` asks for, as MRI
/// does not check one against the other, and binary when `mode` says so.
fn mode_string(descriptor: i32, mode: i32) -> String {
    let access = match access_flags(descriptor) {
        FMODE_READABLE => "r",
        FMODE_WRITABLE => "w",
        _ => "r+",
    };
    let binary = if mode & FMODE_BINMODE != 0 { "b" } else { "" };
    format!("{}{}", access, binary)
}

/// A new instance of `klass` over `descriptor`, naming `path`, with the
/// timeout and encodings given.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_io_open_descriptor(
    klass: Value,
    descriptor: i32,
    mode: i32,
    path: Value,
    timeout: Value,
    encoding: *const RbIoEncoding,
) -> Value {
    let path = super::strings::string_value(to_object(path));
    let path = call(path, "dup", Vec::new());
    let arguments = vec![
        Object::Int(i64::from(descriptor)),
        Object::string(mode_string(descriptor, mode)),
    ];
    let io = call(to_object(klass), "for_fd", arguments);
    call(
        io.clone(),
        "instance_variable_set",
        vec![Object::symbol("@path"), call(path, "freeze", Vec::new())],
    );
    if timeout != QNIL {
        call(io.clone(), "timeout=", vec![to_object(timeout)]);
    }
    if !encoding.is_null() {
        // SAFETY: C hands over an rb_io_encoding or NULL.
        let held = unsafe { &*encoding };
        let external = super::encodings::encoding_object(held.external);
        let internal = super::encodings::encoding_object(held.internal);
        call(io.clone(), "set_encoding", vec![external, internal]);
    }
    to_value(&io)
}

/// What C hands `rb_io_open_descriptor` for the encodings.
#[repr(C)]
pub struct RbIoEncoding {
    internal: *const super::encodings::RbEncoding,
    external: *const super::encodings::RbEncoding,
    flags: i32,
    options: Value,
}
