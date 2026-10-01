// The sockets a program holds open, and how long a wait on one runs.

use super::*;

impl VirtualMachine {
    /// The descriptor behind a handle, whichever of the kinds of socket it
    /// names.
    pub(crate) fn socket_descriptor(&self, handle: u64) -> Option<i32> {
        use std::os::unix::io::AsRawFd as _;
        if let Some(held) = self.open_sockets.listeners.get(&handle) {
            return Some(held.as_raw_fd());
        }
        if let Some(held) = self.open_sockets.streams.get(&handle) {
            return Some(held.as_raw_fd());
        }
        if let Some(held) = self.open_sockets.unix_listeners.get(&handle) {
            return Some(held.as_raw_fd());
        }
        if let Some(held) = self.open_sockets.unix_streams.get(&handle) {
            return Some(held.as_raw_fd());
        }
        if let Some(held) = self.open_sockets.datagrams.get(&handle) {
            return Some(held.as_raw_fd());
        }
        self.open_sockets
            .unix_datagrams
            .get(&handle)
            .map(|held| held.as_raw_fd())
    }
}

/// The listeners and connections a program holds open, each named by a
/// number the Ruby object carries.
#[derive(Default)]
pub(crate) struct OpenSockets {
    pub(crate) listeners: std::collections::HashMap<u64, std::net::TcpListener>,
    pub(crate) streams: std::collections::HashMap<u64, std::net::TcpStream>,
    pub(crate) unix_listeners: std::collections::HashMap<u64, std::os::unix::net::UnixListener>,
    pub(crate) unix_streams: std::collections::HashMap<u64, std::os::unix::net::UnixStream>,
    pub(crate) datagrams: std::collections::HashMap<u64, std::net::UdpSocket>,
    pub(crate) unix_datagrams: std::collections::HashMap<u64, std::os::unix::net::UnixDatagram>,
    pub(crate) next: u64,
}

/// How long a read or an accept waits before giving up. Metorex runs one
/// thread, so a socket that would block forever has nobody left to write to
/// it, and waiting is what a deadlock looks like from the inside.
pub(crate) const WAIT_LIMIT: std::time::Duration = std::time::Duration::from_secs(2);
/// The longest a wait may run altogether. A machine with more work on it than
/// cores runs each thread in fits and starts, so a wait that keeps handing
/// turns over is given more time. This is where that stops, so a wait nothing
/// can ever end still ends.
pub(crate) const WAIT_CEILING: std::time::Duration = std::time::Duration::from_secs(20);
/// How long one read on a socket waits before handing control back. A read
/// waits for as long as `WAIT_LIMIT` altogether, in steps this short, so
/// whatever else the program has to run gets a turn in between.
pub(crate) const POLL_LIMIT: std::time::Duration = std::time::Duration::from_millis(5);

/// Wait for the next connection to a listener, giving up once the limit is
/// past. `std` names no timeout for accepting, so the listener is asked over
/// and over until one arrives or the time runs out.
pub(crate) trait Accepts {
    type Stream;
    fn try_once(&self) -> std::io::Result<Self::Stream>;
    fn set_waiting(&self, blocking: bool) -> std::io::Result<()>;
}

impl Accepts for std::net::TcpListener {
    type Stream = (std::net::TcpStream, std::net::SocketAddr);

    fn try_once(&self) -> std::io::Result<Self::Stream> {
        self.accept()
    }

    fn set_waiting(&self, blocking: bool) -> std::io::Result<()> {
        self.set_nonblocking(!blocking)
    }
}

impl Accepts for std::os::unix::net::UnixListener {
    type Stream = (
        std::os::unix::net::UnixStream,
        std::os::unix::net::SocketAddr,
    );

    fn try_once(&self) -> std::io::Result<Self::Stream> {
        self.accept()
    }

    fn set_waiting(&self, blocking: bool) -> std::io::Result<()> {
        self.set_nonblocking(!blocking)
    }
}

/// The error a wait that came to nothing raises, which is what Ruby raises
/// for a socket that had nothing to give.
pub(crate) fn timed_out(position: Position) -> MetorexError {
    crate::vm::errors::simple_exception(
        "Errno::EAGAIN",
        "Resource temporarily unavailable - read would block",
        position,
    )
}

/// The name of the Errno class that stands for what the operating system
/// reported, so a program rescues the error it would rescue in Ruby.
pub(crate) fn errno_class(problem: &std::io::Error) -> &'static str {
    match problem.raw_os_error() {
        Some(libc::ENOTCONN) => "Errno::ENOTCONN",
        Some(libc::ECONNRESET) => "Errno::ECONNRESET",
        Some(libc::EPIPE) => "Errno::EPIPE",
        Some(libc::EINVAL) => "Errno::EINVAL",
        Some(libc::EDESTADDRREQ) => "Errno::EDESTADDRREQ",
        Some(libc::EMSGSIZE) => "Errno::EMSGSIZE",
        Some(libc::EACCES) => "Errno::EACCES",
        Some(libc::EADDRINUSE) => "Errno::EADDRINUSE",
        Some(libc::EOPNOTSUPP) => "Errno::EOPNOTSUPP",
        Some(libc::EAGAIN) => "Errno::EAGAIN",
        Some(number) => crate::vm::init::errno_class_name(number).unwrap_or("Errno::ECONNREFUSED"),
        None => "Errno::ECONNREFUSED",
    }
}

/// The name of the Errno class a failed bind stands for. A bind reports
/// addresses the machine does not answer on and ones already taken apart
/// from anything else that went wrong.
pub(crate) fn bind_errno_class(problem: &std::io::Error) -> &'static str {
    match problem.raw_os_error() {
        Some(libc::EADDRNOTAVAIL) => "Errno::EADDRNOTAVAIL",
        Some(libc::EADDRINUSE) => "Errno::EADDRINUSE",
        other => errno_class(&std::io::Error::from_raw_os_error(other.unwrap_or(0))),
    }
}

/// The path the other end of a socket answers to, read from the operating
/// system. Nothing when the other end has no name of its own.
pub(crate) fn peer_path_of(descriptor: i32) -> Option<String> {
    let mut held: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    let mut size = std::mem::size_of::<libc::sockaddr_un>() as libc::socklen_t;
    let answered = unsafe {
        libc::getpeername(
            descriptor,
            &mut held as *mut libc::sockaddr_un as *mut libc::sockaddr,
            &mut size,
        )
    };
    if answered < 0 || held.sun_family != libc::AF_UNIX as libc::sa_family_t {
        return None;
    }
    // A path is held as the C library's own character type, which is signed
    // on some platforms and unsigned on others, so the cast is needed on the
    // one and stands for nothing on the other.
    #[allow(clippy::unnecessary_cast)]
    let spelled: Vec<u8> = held
        .sun_path
        .iter()
        .take_while(|byte| **byte != 0)
        .map(|byte| *byte as u8)
        .collect();
    if spelled.is_empty() {
        return None;
    }
    String::from_utf8(spelled).ok()
}

/// The address a sockaddr holds, written the way a program reads it, or
/// nothing when the pointer names no address of a family we can write.
/// The address at the other end of a point-to-point interface, which each
/// platform keeps under its own name: BSD calls it the destination address,
/// while Linux holds it in a union it shares with the broadcast address.
#[cfg(target_os = "linux")]
pub(crate) fn point_to_point_address(entry: &libc::ifaddrs) -> *const libc::sockaddr {
    entry.ifa_ifu
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn point_to_point_address(entry: &libc::ifaddrs) -> *const libc::sockaddr {
    entry.ifa_dstaddr
}

pub(crate) fn address_of_sockaddr(held: *const libc::sockaddr) -> Object {
    if held.is_null() {
        return Object::Nil;
    }
    let family = unsafe { (*held).sa_family } as libc::c_int;
    if family == libc::AF_INET {
        let held = held as *const libc::sockaddr_in;
        let bytes = unsafe { (*held).sin_addr.s_addr }.to_ne_bytes();
        return Object::string(std::net::Ipv4Addr::from(bytes).to_string());
    }
    if family == libc::AF_INET6 {
        let held = held as *const libc::sockaddr_in6;
        let bytes = unsafe { (*held).sin6_addr.s6_addr };
        return Object::string(std::net::Ipv6Addr::from(bytes).to_string());
    }
    Object::Nil
}
