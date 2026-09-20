// Network addresses: reading one out of the text a program writes it as, and
// writing one back out in the shape the operating system expects.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::core::VirtualMachine;

/// The four bytes an IPv4 address stands for, or None when the text is not
/// one.
fn ipv4_bytes(text: &str) -> Option<[u8; 4]> {
    let mut held = [0u8; 4];
    let mut pieces = text.split('.');
    for slot in held.iter_mut() {
        let piece = pieces.next()?;
        if piece.is_empty() || piece.len() > 3 || !piece.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *slot = piece.parse().ok()?;
    }
    if pieces.next().is_some() {
        return None;
    }
    Some(held)
}

/// The sixteen bytes an IPv6 address stands for, `::` included.
fn ipv6_bytes(text: &str) -> Option<[u8; 16]> {
    let held: std::net::Ipv6Addr = text.parse().ok()?;
    Some(held.octets())
}

/// How an address was written, which decides how everything else reads it.
enum Written {
    Four([u8; 4]),
    Six([u8; 16]),
}

fn read_address(text: &str) -> Option<Written> {
    if let Some(held) = ipv4_bytes(text) {
        return Some(Written::Four(held));
    }
    ipv6_bytes(text).map(Written::Six)
}

/// An IPv6 address written the shortest way it can be, which is how Ruby
/// spells one back out.
fn write_ipv6(bytes: [u8; 16]) -> String {
    std::net::Ipv6Addr::from(bytes).to_string()
}

impl VirtualMachine {
    /// `Socket.__address__(action, text, port)` — the one place the socket
    /// library reads and writes addresses, since their shape belongs to the
    /// operating system rather than to Ruby.
    pub(crate) fn socket_address(
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
        let text = match arguments.get(1) {
            Some(Object::String(held)) => held.as_str().to_string(),
            _ => String::new(),
        };
        let port = match arguments.get(2) {
            Some(Object::Int(held)) => *held,
            _ => 0,
        };
        let refuse =
            |what: String| crate::vm::errors::simple_exception("SocketError", &what, position);
        match &*action.as_str() {
            // Which family an address belongs to, or nothing when the text
            // names no address at all.
            "family" => Ok(match read_address(&text) {
                Some(Written::Four(_)) => Object::Int(4),
                Some(Written::Six(_)) => Object::Int(6),
                None => Object::Nil,
            }),
            // The address written back out in its shortest form.
            "normalize" => match read_address(&text) {
                Some(Written::Four(held)) => {
                    Ok(Object::string(std::net::Ipv4Addr::from(held).to_string()))
                }
                Some(Written::Six(held)) => Ok(Object::string(write_ipv6(held))),
                None => Err(refuse(format!(
                    "getaddrinfo: {text}: Name or service not known"
                ))),
            },
            // The bytes an address stands for, one to a character.
            "bytes" => match read_address(&text) {
                Some(Written::Four(held)) => Ok(super::pack_format::bytes_to_string(&held)),
                Some(Written::Six(held)) => Ok(super::pack_format::bytes_to_string(&held)),
                None => Err(refuse(format!(
                    "getaddrinfo: {text}: Name or service not known"
                ))),
            },
            // The struct the operating system takes an address in, which
            // carries a length byte on some systems and not on others.
            "sockaddr" => {
                let held = read_address(&text).ok_or_else(|| {
                    refuse(format!("getaddrinfo: {text}: Name or service not known"))
                })?;
                let mut out = Vec::new();
                match held {
                    Written::Four(address) => {
                        push_family(&mut out, libc::AF_INET as u16, 16);
                        out.extend_from_slice(&(port as u16).to_be_bytes());
                        out.extend_from_slice(&address);
                        out.extend_from_slice(&[0u8; 8]);
                    }
                    Written::Six(address) => {
                        push_family(&mut out, libc::AF_INET6 as u16, 28);
                        out.extend_from_slice(&(port as u16).to_be_bytes());
                        out.extend_from_slice(&[0u8; 4]);
                        out.extend_from_slice(&address);
                        out.extend_from_slice(&[0u8; 4]);
                    }
                }
                Ok(super::pack_format::bytes_to_string(&out))
            }
            // The address and port a struct carries, read back out.
            "unpack" => {
                let bytes = super::pack_format::string_to_bytes(&text);
                match read_sockaddr(&bytes) {
                    Some((named, held)) => Ok(Object::array(vec![
                        Object::string(named),
                        Object::Int(i64::from(held)),
                    ])),
                    None => Err(refuse("not an IP address struct".to_string())),
                }
            }
            // The names a host answers to, resolved by the system.
            // The name an address is known by, or nothing when it has none.
            "name_of" => {
                let Ok(address) = text.parse::<std::net::IpAddr>() else {
                    return Ok(Object::Nil);
                };
                let mut buffer = vec![0u8; libc::NI_MAXHOST as usize];
                let answered = match address {
                    std::net::IpAddr::V4(held) => {
                        let mut sockaddr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
                        sockaddr.sin_family = libc::AF_INET as libc::sa_family_t;
                        sockaddr.sin_addr.s_addr = u32::from_ne_bytes(held.octets());
                        unsafe {
                            libc::getnameinfo(
                                &sockaddr as *const libc::sockaddr_in as *const libc::sockaddr,
                                std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
                                buffer.as_mut_ptr() as *mut libc::c_char,
                                buffer.len() as libc::socklen_t,
                                std::ptr::null_mut(),
                                0,
                                0,
                            )
                        }
                    }
                    std::net::IpAddr::V6(_) => -1,
                };
                if answered != 0 {
                    return Ok(Object::Nil);
                }
                let named = unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr() as *const _) };
                Ok(Object::string(named.to_string_lossy().to_string()))
            }
            "resolve" => match std::net::ToSocketAddrs::to_socket_addrs(&(text.as_str(), 0u16)) {
                Ok(found) => Ok(Object::array(
                    found
                        .map(|held| Object::string(held.ip().to_string()))
                        .collect(),
                )),
                Err(_) => Err(refuse(format!(
                    "getaddrinfo: {text}: nodename nor servname provided, or not known"
                ))),
            },
            // The name this machine answers to.
            "hostname" => Ok(Object::string(
                hostname().unwrap_or_else(|| "localhost".to_string()),
            )),
            other => Err(MetorexError::runtime_error(
                format!("unknown address action {other}"),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }
}

/// The name this machine answers to, as the system reports it.
fn hostname() -> Option<String> {
    let mut buffer = [0u8; 256];
    // SAFETY: `gethostname` writes at most the length given into the buffer,
    // which outlives the call.
    let answered = unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) };
    if answered != 0 {
        return None;
    }
    let end = buffer.iter().position(|byte| *byte == 0).unwrap_or(0);
    String::from_utf8(buffer[..end].to_vec()).ok()
}

/// Write the family into a sockaddr struct. BSD systems put the struct's
/// length in front of it; Linux does not and holds the family in two bytes.
fn push_family(out: &mut Vec<u8>, family: u16, length: u8) {
    if cfg!(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    )) {
        out.push(length);
        out.push(family as u8);
    } else {
        out.extend_from_slice(&family.to_le_bytes());
    }
}

/// The address and port a sockaddr struct carries.
fn read_sockaddr(bytes: &[u8]) -> Option<(String, u16)> {
    if bytes.len() < 8 {
        return None;
    }
    let family = if cfg!(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    )) {
        u16::from(bytes[1])
    } else {
        u16::from_le_bytes([bytes[0], bytes[1]])
    };
    let port = u16::from_be_bytes([bytes[2], bytes[3]]);
    if family == libc::AF_INET as u16 {
        let held: [u8; 4] = bytes.get(4..8)?.try_into().ok()?;
        return Some((std::net::Ipv4Addr::from(held).to_string(), port));
    }
    if family == libc::AF_INET6 as u16 {
        let held: [u8; 16] = bytes.get(8..24)?.try_into().ok()?;
        return Some((write_ipv6(held), port));
    }
    None
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
const WAIT_LIMIT: std::time::Duration = std::time::Duration::from_secs(2);
/// The longest a wait may run altogether. A machine with more work on it than
/// cores runs each thread in fits and starts, so a wait that keeps handing
/// turns over is given more time. This is where that stops, so a wait nothing
/// can ever end still ends.
const WAIT_CEILING: std::time::Duration = std::time::Duration::from_secs(20);
/// How long one read on a socket waits before handing control back. A read
/// waits for as long as `WAIT_LIMIT` altogether, in steps this short, so
/// whatever else the program has to run gets a turn in between.
const POLL_LIMIT: std::time::Duration = std::time::Duration::from_millis(5);

/// Wait for the next connection to a listener, giving up once the limit is
/// past. `std` names no timeout for accepting, so the listener is asked over
/// and over until one arrives or the time runs out.
trait Accepts {
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
fn timed_out(position: Position) -> MetorexError {
    crate::vm::errors::simple_exception(
        "Errno::EAGAIN",
        "Resource temporarily unavailable - read would block",
        position,
    )
}

impl VirtualMachine {
    /// `Socket.__net__(action, handle, text, port)` — opening, accepting,
    /// reading, and writing, which belong to the operating system.
    pub(crate) fn socket_net(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        use std::io::{Read as _, Write as _};
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
        let port = match arguments.get(3) {
            Some(Object::Int(held)) => *held,
            _ => 0,
        };
        let refuse = |what: String| {
            crate::vm::errors::simple_exception("Errno::ECONNREFUSED", &what, position)
        };
        match &*action.as_str() {
            // Take a name and a port and start listening there.
            "listen" => {
                let held = std::net::TcpListener::bind((text.as_str(), port as u16)).map_err(
                    |problem| {
                        crate::vm::errors::simple_exception(
                            bind_errno_class(&problem),
                            &format!("bind({text}:{port}): {problem}"),
                            position,
                        )
                    },
                )?;
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
                let _ = held.set_nonblocking(true);
                self.open_sockets.listeners.insert(named, held);
                Ok(Object::Int(named as i64))
            }
            // Take a name and a port without yet waiting for anything to
            // arrive. A socket that is bound and no more refuses what tries
            // to reach it, which is what a connection to one is told.
            "bind_only" => {
                use std::net::ToSocketAddrs as _;
                use std::os::unix::io::FromRawFd as _;
                let wanted: std::net::SocketAddr = (text.as_str(), port as u16)
                    .to_socket_addrs()
                    .ok()
                    .and_then(|mut found| found.next())
                    .ok_or_else(|| {
                        crate::vm::errors::simple_exception(
                            "SocketError",
                            &format!("bind({text}:{port}): no such address"),
                            position,
                        )
                    })?;
                let family = match wanted {
                    std::net::SocketAddr::V4(_) => libc::AF_INET,
                    std::net::SocketAddr::V6(_) => libc::AF_INET6,
                };
                // SAFETY: the descriptor is taken over by the listener built
                // from it, which closes it when it goes.
                let held = unsafe {
                    let descriptor = libc::socket(family, libc::SOCK_STREAM, 0);
                    if descriptor < 0 {
                        let problem = std::io::Error::last_os_error();
                        return Err(crate::vm::errors::simple_exception(
                            bind_errno_class(&problem),
                            &format!("bind({text}:{port}): {problem}"),
                            position,
                        ));
                    }
                    let (address, length) = sockaddr_of(&wanted);
                    if libc::bind(descriptor, address.as_ptr().cast(), length) < 0 {
                        let problem = std::io::Error::last_os_error();
                        libc::close(descriptor);
                        return Err(crate::vm::errors::simple_exception(
                            bind_errno_class(&problem),
                            &format!("bind({text}:{port}): {problem}"),
                            position,
                        ));
                    }
                    std::net::TcpListener::from_raw_fd(descriptor)
                };
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
                let _ = held.set_nonblocking(true);
                self.open_sockets.listeners.insert(named, held);
                Ok(Object::Int(named as i64))
            }
            // Start waiting for connections on a socket already bound.
            "listen_on" => {
                let Some(descriptor) = self.socket_descriptor(handle) else {
                    return Err(refuse("listen on a closed socket".to_string()));
                };
                let backlog = if port > 0 { port as i32 } else { 5 };
                // SAFETY: the descriptor is one this program holds open.
                if unsafe { libc::listen(descriptor, backlog) } < 0 {
                    let problem = std::io::Error::last_os_error();
                    return Err(crate::vm::errors::simple_exception(
                        errno_class(&problem),
                        &format!("listen: {problem}"),
                        position,
                    ));
                }
                Ok(Object::Int(0))
            }
            // Reach a name and a port that something is listening on.
            "connect" => {
                let held = std::net::TcpStream::connect((text.as_str(), port as u16))
                    .map_err(|problem| refuse(format!("connect({text}:{port}): {problem}")))?;
                let _ = held.set_read_timeout(Some(POLL_LIMIT));
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
                let _ = held.set_nonblocking(true);
                self.open_sockets.streams.insert(named, held);
                Ok(Object::Int(named as i64))
            }
            // Where a listener or a connection sits.
            "address" => {
                let found = self
                    .open_sockets
                    .listeners
                    .get(&handle)
                    .and_then(|held| held.local_addr().ok())
                    .or_else(|| {
                        self.open_sockets
                            .streams
                            .get(&handle)
                            .and_then(|held| held.local_addr().ok())
                    });
                Ok(match found {
                    Some(held) => Object::array(vec![
                        Object::string(held.ip().to_string()),
                        Object::Int(i64::from(held.port())),
                    ]),
                    None => Object::Nil,
                })
            }
            // Where the other end of a connection sits.
            "peer" => Ok(
                match self
                    .open_sockets
                    .streams
                    .get(&handle)
                    .and_then(|held| held.peer_addr().ok())
                {
                    Some(held) => Object::array(vec![
                        Object::string(held.ip().to_string()),
                        Object::Int(i64::from(held.port())),
                    ]),
                    None => Object::Nil,
                },
            ),
            // Take the next connection something has made to a listener.
            "accept" => {
                // Waiting hands other threads a turn, and they may reach the
                // same listener, so it is looked up again each time round
                // rather than borrowed across the wait.
                let mut deadline = std::time::Instant::now() + WAIT_LIMIT;
                let waited_enough = std::time::Instant::now() + WAIT_CEILING;
                let stream = loop {
                    let listener = self
                        .open_sockets
                        .listeners
                        .get(&handle)
                        .ok_or_else(|| refuse("accept on a closed listener".to_string()))?;
                    let _ = listener.set_waiting(false);
                    let taken = listener.try_once();
                    let _ = listener.set_waiting(true);
                    match taken {
                        Ok((held, _)) => break held,
                        Err(problem) if problem.kind() == std::io::ErrorKind::WouldBlock => {
                            // A thread that could still write is given its
                            // turn, and the wait is given more time for
                            // having handed one over, up to a ceiling that a
                            // wait nothing can ever end still reaches.
                            if std::time::Instant::now() >= deadline {
                                return Err(timed_out(position));
                            }
                            self.wait_for_other_threads(position);
                            self.raise_if_thread_killed(position)?;
                            if self.other_threads_are_waiting() {
                                deadline =
                                    (std::time::Instant::now() + WAIT_LIMIT).min(waited_enough);
                            }
                        }
                        Err(problem) => {
                            return Err(crate::vm::errors::simple_exception(
                                "Errno::ECONNREFUSED",
                                &format!("accept: {problem}"),
                                position,
                            ));
                        }
                    }
                };
                let _ = stream.set_read_timeout(Some(POLL_LIMIT));
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
                let _ = stream.set_nonblocking(true);
                self.open_sockets.streams.insert(named, stream);
                Ok(Object::Int(named as i64))
            }
            "write" => {
                let stream = self
                    .open_sockets
                    .streams
                    .get_mut(&handle)
                    .ok_or_else(|| refuse("write to a closed connection".to_string()))?;
                let bytes = super::pack_format::string_to_bytes(&text);
                // `port` carries the flags a send was given. Out-of-band data
                // travels ahead of what is already queued, which the ordinary
                // write has no way to ask for.
                if port != 0 {
                    use std::os::unix::io::AsRawFd as _;
                    let sent = unsafe {
                        libc::send(
                            stream.as_raw_fd(),
                            bytes.as_ptr() as *const libc::c_void,
                            bytes.len(),
                            port as libc::c_int,
                        )
                    };
                    if sent < 0 {
                        let problem = std::io::Error::last_os_error();
                        return Err(crate::vm::errors::simple_exception(
                            errno_class(&problem),
                            &format!("send: {problem}"),
                            position,
                        ));
                    }
                    return Ok(Object::Int(sent as i64));
                }
                stream.write_all(&bytes).map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        errno_class(&problem),
                        &format!("write: {problem}"),
                        position,
                    )
                })?;
                Ok(Object::Int(bytes.len() as i64))
            }
            // Whether the socket has something to read, or room to write,
            // as the system reports it rather than by trying and undoing.
            "ready?" => {
                let Some(descriptor) = self.socket_descriptor(handle) else {
                    return Ok(Object::Bool(false));
                };
                let mut asked = libc::pollfd {
                    fd: descriptor,
                    events: if port != 0 {
                        libc::POLLOUT
                    } else {
                        libc::POLLIN
                    },
                    revents: 0,
                };
                // SAFETY: the descriptor is one this program holds open, and
                // the wait of zero asks without waiting.
                let answered = unsafe { libc::poll(&mut asked, 1, 0) };
                Ok(Object::Bool(answered > 0 && asked.revents != 0))
            }
            // Read what has arrived without waiting for more. Nothing there
            // yet is reported at once rather than handing the turn over,
            // which is what a read told not to wait asks for.
            "read_now" => {
                let wanted = if port > 0 { port as usize } else { 65536 };
                let mut buffer = vec![0u8; wanted];
                let stream = self
                    .open_sockets
                    .streams
                    .get_mut(&handle)
                    .ok_or_else(|| refuse("read from a closed connection".to_string()))?;
                match stream.read(&mut buffer) {
                    Ok(read) => {
                        buffer.truncate(read);
                        Ok(super::pack_format::bytes_to_string(&buffer))
                    }
                    Err(problem)
                        if matches!(
                            problem.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                        ) =>
                    {
                        Err(timed_out(position))
                    }
                    Err(problem) => Err(refuse(format!("read: {problem}"))),
                }
            }
            // Read what the other end sent, up to the count asked for.
            "read" => {
                let wanted = if port > 0 { port as usize } else { 65536 };
                let mut buffer = vec![0u8; wanted];
                // Nothing has arrived yet is not the end of the matter, so
                // whatever else the program has to run gets a turn and the
                // read is tried again.
                let mut deadline = std::time::Instant::now() + WAIT_LIMIT;
                let waited_enough = std::time::Instant::now() + WAIT_CEILING;
                let read = loop {
                    let stream = self
                        .open_sockets
                        .streams
                        .get_mut(&handle)
                        .ok_or_else(|| refuse("read from a closed connection".to_string()))?;
                    match stream.read(&mut buffer) {
                        Ok(held) => break held,
                        Err(problem)
                            if matches!(
                                problem.kind(),
                                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                            ) =>
                        {
                            // A thread that could still write is given its
                            // turn, and the wait is given more time for
                            // having handed one over, up to a ceiling that a
                            // wait nothing can ever end still reaches.
                            if std::time::Instant::now() >= deadline {
                                return Err(timed_out(position));
                            }
                            self.wait_for_other_threads(position);
                            self.raise_if_thread_killed(position)?;
                            if self.other_threads_are_waiting() {
                                deadline =
                                    (std::time::Instant::now() + WAIT_LIMIT).min(waited_enough);
                            }
                        }
                        Err(problem) => return Err(refuse(format!("read: {problem}"))),
                    }
                };
                buffer.truncate(read);
                Ok(super::pack_format::bytes_to_string(&buffer))
            }
            "close" => {
                self.open_sockets.listeners.remove(&handle);
                self.open_sockets.streams.remove(&handle);
                self.open_sockets.unix_listeners.remove(&handle);
                self.open_sockets.unix_streams.remove(&handle);
                self.open_sockets.datagrams.remove(&handle);
                Ok(Object::Nil)
            }
            // The number the operating system holds a socket under, which is
            // what `fileno` reports.
            // Close one direction of a connection, so the other end reads
            // the end of the stream without the socket itself closing.
            "shutdown" => {
                use std::os::unix::io::AsRawFd as _;
                let found = self
                    .open_sockets
                    .streams
                    .get(&handle)
                    .map(|held| held.as_raw_fd())
                    .or_else(|| {
                        self.open_sockets
                            .unix_streams
                            .get(&handle)
                            .map(|held| held.as_raw_fd())
                    });
                let Some(number) = found else {
                    return Err(refuse("shutdown of a closed connection".to_string()));
                };
                // SAFETY: `number` is a descriptor this program holds open.
                let answered = unsafe { libc::shutdown(number, port as libc::c_int) };
                if answered < 0 {
                    let problem = std::io::Error::last_os_error();
                    return Err(refuse(format!("shutdown: {problem}")));
                }
                Ok(Object::Int(0))
            }
            "fileno" => {
                if let Some(found) = self.socket_descriptor(handle) {
                    return Ok(Object::Int(i64::from(found)));
                }
                use std::os::unix::io::AsRawFd as _;
                let found = self
                    .open_sockets
                    .listeners
                    .get(&handle)
                    .map(|held| held.as_raw_fd())
                    .or_else(|| {
                        self.open_sockets
                            .streams
                            .get(&handle)
                            .map(|h| h.as_raw_fd())
                    })
                    .or_else(|| {
                        self.open_sockets
                            .unix_listeners
                            .get(&handle)
                            .map(|h| h.as_raw_fd())
                    })
                    .or_else(|| {
                        self.open_sockets
                            .unix_streams
                            .get(&handle)
                            .map(|h| h.as_raw_fd())
                    })
                    .or_else(|| {
                        self.open_sockets
                            .datagrams
                            .get(&handle)
                            .map(|h| h.as_raw_fd())
                    });
                Ok(match found {
                    Some(held) => Object::Int(i64::from(held)),
                    None => Object::Nil,
                })
            }
            // The socket a descriptor names, answered as the handle metorex
            // holds it under, which is what `for_fd` builds another socket
            // around.
            "handle_of_fd" => {
                use std::os::unix::io::AsRawFd as _;
                let wanted = port as i32;
                let found = self
                    .open_sockets
                    .listeners
                    .iter()
                    .map(|(held, socket)| (*held, socket.as_raw_fd()))
                    .chain(
                        self.open_sockets
                            .streams
                            .iter()
                            .map(|(held, socket)| (*held, socket.as_raw_fd())),
                    )
                    .chain(
                        self.open_sockets
                            .unix_listeners
                            .iter()
                            .map(|(held, socket)| (*held, socket.as_raw_fd())),
                    )
                    .chain(
                        self.open_sockets
                            .unix_streams
                            .iter()
                            .map(|(held, socket)| (*held, socket.as_raw_fd())),
                    )
                    .chain(
                        self.open_sockets
                            .datagrams
                            .iter()
                            .map(|(held, socket)| (*held, socket.as_raw_fd())),
                    )
                    .chain(
                        self.open_sockets
                            .unix_datagrams
                            .iter()
                            .map(|(held, socket)| (*held, socket.as_raw_fd())),
                    )
                    .find(|(_, number)| *number == wanted)
                    .map(|(held, _)| held);
                Ok(match found {
                    Some(held) => Object::Int(held as i64),
                    None => Object::Nil,
                })
            }
            // The port a named service is reached on, and the name a port is
            // known by, as the operating system's own table has them.
            // The number this platform names a socket setting, a family, or a
            // message flag by. The names differ between platforms, so a
            // program that writes one of them by hand would be writing the
            // number another platform uses.
            "constant" => Ok(socket_constant(&text)
                .map(Object::Int)
                .unwrap_or(Object::Nil)),
            // Ask the device behind the socket to do something, with a
            // buffer it reads from and writes back into.
            "ioctl" => {
                let Some(descriptor) = self.socket_descriptor(handle) else {
                    return Err(refuse("ioctl on a closed socket".to_string()));
                };
                let mut buffer = super::pack_format::string_to_bytes(&text);
                let request = port as libc::c_ulong;
                // SAFETY: the descriptor is one this program holds open, and
                // the buffer outlives the call.
                let answered = if buffer.is_empty() {
                    unsafe { libc::ioctl(descriptor, request, 0) }
                } else {
                    unsafe { libc::ioctl(descriptor, request, buffer.as_mut_ptr()) }
                };
                if answered < 0 {
                    let problem = std::io::Error::last_os_error();
                    return Err(crate::vm::errors::simple_exception(
                        errno_class(&problem),
                        &format!("ioctl: {problem}"),
                        position,
                    ));
                }
                Ok(Object::array(vec![
                    Object::Int(i64::from(answered)),
                    super::pack_format::bytes_to_string(&buffer),
                ]))
            }
            // Whether the socket answers straight away rather than waiting,
            // as the descriptor itself is set, and setting it.
            "nonblock" => {
                let Some(descriptor) = self.socket_descriptor(handle) else {
                    return Ok(Object::Nil);
                };
                // SAFETY: the descriptor is one this program holds open.
                let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
                Ok(Object::Bool(flags >= 0 && flags & libc::O_NONBLOCK != 0))
            }
            "set_nonblock" => {
                let Some(descriptor) = self.socket_descriptor(handle) else {
                    return Ok(Object::Nil);
                };
                // SAFETY: the descriptor is one this program holds open.
                unsafe {
                    let flags = libc::fcntl(descriptor, libc::F_GETFL);
                    if flags >= 0 {
                        let wanted = if port != 0 {
                            flags | libc::O_NONBLOCK
                        } else {
                            flags & !libc::O_NONBLOCK
                        };
                        libc::fcntl(descriptor, libc::F_SETFL, wanted);
                    }
                }
                Ok(Object::Bool(port != 0))
            }
            "service_port" => {
                let (service, protocol) = match text.split_once('/') {
                    Some((named, kind)) => (named.to_string(), kind.to_string()),
                    None => (text.to_string(), "tcp".to_string()),
                };
                let name = std::ffi::CString::new(service).unwrap_or_default();
                let kind = std::ffi::CString::new(protocol).unwrap_or_default();
                let found = unsafe { libc::getservbyname(name.as_ptr(), kind.as_ptr()) };
                if found.is_null() {
                    return Ok(Object::Nil);
                }
                let port = unsafe { (*found).s_port };
                Ok(Object::Int(i64::from(u16::from_be(port as u16))))
            }
            "service_name" => {
                let kind = std::ffi::CString::new(text.to_string()).unwrap_or_default();
                let found =
                    unsafe { libc::getservbyport((port as u16).to_be() as i32, kind.as_ptr()) };
                if found.is_null() {
                    return Ok(Object::Nil);
                }
                let named = unsafe { std::ffi::CStr::from_ptr((*found).s_name) };
                Ok(Object::string(named.to_string_lossy().to_string()))
            }
            // What a socket setting holds right now, as the bytes the
            // operating system keeps it in. `text` names the level and the
            // option, and `port` how many bytes to read back.
            "socket_option" => {
                let mut parts = text.split('/');
                let level: libc::c_int = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
                let name: libc::c_int = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
                let Some(descriptor) = self.socket_descriptor(handle) else {
                    return Err(refuse("option of a closed socket".to_string()));
                };
                let wanted = if port > 0 { port as usize } else { 4 };
                let mut buffer = vec![0u8; wanted];
                let mut size = wanted as libc::socklen_t;
                let answered = unsafe {
                    libc::getsockopt(
                        descriptor,
                        level,
                        name,
                        buffer.as_mut_ptr() as *mut libc::c_void,
                        &mut size,
                    )
                };
                if answered < 0 {
                    let problem = std::io::Error::last_os_error();
                    return Err(crate::vm::errors::simple_exception(
                        errno_class(&problem),
                        &format!("getsockopt: {problem}"),
                        position,
                    ));
                }
                buffer.truncate(size as usize);
                Ok(super::pack_format::bytes_to_string(&buffer))
            }
            // Change a socket setting, with `text` naming the level, the
            // option, and the bytes to write, separated by slashes.
            "set_socket_option" => {
                let mut parts = text.splitn(3, '/');
                let level: libc::c_int = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
                let name: libc::c_int = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
                let bytes = super::pack_format::string_to_bytes(parts.next().unwrap_or(""));
                let Some(descriptor) = self.socket_descriptor(handle) else {
                    return Err(refuse("option of a closed socket".to_string()));
                };
                let answered = unsafe {
                    libc::setsockopt(
                        descriptor,
                        level,
                        name,
                        bytes.as_ptr() as *const libc::c_void,
                        bytes.len() as libc::socklen_t,
                    )
                };
                if answered < 0 {
                    let problem = std::io::Error::last_os_error();
                    return Err(crate::vm::errors::simple_exception(
                        errno_class(&problem),
                        &format!("setsockopt: {problem}"),
                        position,
                    ));
                }
                Ok(Object::Int(0))
            }
            // Every network interface this machine has, one line per entry,
            // written as name/flags/index/address/netmask/broadcast.
            "interfaces" => {
                let mut held: *mut libc::ifaddrs = std::ptr::null_mut();
                if unsafe { libc::getifaddrs(&mut held) } != 0 {
                    return Ok(Object::array(Vec::new()));
                }
                let mut found = Vec::new();
                let mut walking = held;
                while !walking.is_null() {
                    let entry = unsafe { &*walking };
                    walking = entry.ifa_next;
                    if entry.ifa_name.is_null() {
                        continue;
                    }
                    let name = unsafe { std::ffi::CStr::from_ptr(entry.ifa_name) }
                        .to_string_lossy()
                        .to_string();
                    let index = unsafe {
                        let spelled = std::ffi::CString::new(name.clone()).unwrap_or_default();
                        libc::if_nametoindex(spelled.as_ptr())
                    };
                    found.push(Object::array(vec![
                        Object::string(name),
                        Object::Int(i64::from(entry.ifa_flags)),
                        Object::Int(i64::from(index)),
                        address_of_sockaddr(entry.ifa_addr),
                        address_of_sockaddr(entry.ifa_netmask),
                        address_of_sockaddr(point_to_point_address(entry)),
                    ]));
                }
                unsafe { libc::freeifaddrs(held) };
                Ok(Object::array(found))
            }
            // Hand a descriptor to the other end of a socket named by a
            // path, and take one that was handed over. `port` carries the
            // descriptor going out.
            "send_fd" => {
                let Some(descriptor) = self.socket_descriptor(handle) else {
                    return Err(refuse("send on a closed socket".to_string()));
                };
                let sent = send_descriptor(descriptor, port as i32);
                if sent < 0 {
                    let problem = std::io::Error::last_os_error();
                    return Err(crate::vm::errors::simple_exception(
                        errno_class(&problem),
                        &format!("sendmsg: {problem}"),
                        position,
                    ));
                }
                Ok(Object::Int(sent as i64))
            }
            "receive_fd" => {
                let Some(descriptor) = self.socket_descriptor(handle) else {
                    return Err(refuse("receive on a closed socket".to_string()));
                };
                let taken = receive_descriptor(descriptor);
                if taken < 0 {
                    let problem = std::io::Error::last_os_error();
                    return Err(crate::vm::errors::simple_exception(
                        errno_class(&problem),
                        &format!("recvmsg: {problem}"),
                        position,
                    ));
                }
                Ok(Object::Int(taken as i64))
            }
            // A socket named by a path that carries each message on its
            // own. An empty path opens one with no name of its own.
            "unix_dgram_open" => {
                let held = if text.is_empty() {
                    std::os::unix::net::UnixDatagram::unbound()
                } else {
                    std::os::unix::net::UnixDatagram::bind(&text)
                }
                .map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        bind_errno_class(&problem),
                        &format!("bind({text}): {problem}"),
                        position,
                    )
                })?;
                let _ = held.set_read_timeout(Some(POLL_LIMIT));
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
                let _ = held.set_nonblocking(true);
                self.open_sockets.unix_datagrams.insert(named, held);
                Ok(Object::Int(named as i64))
            }
            "unix_dgram_send" => {
                let socket = self
                    .open_sockets
                    .unix_datagrams
                    .get(&handle)
                    .ok_or_else(|| refuse("send on a closed socket".to_string()))?;
                let mut parts = text.splitn(2, '\u{0}');
                let path = parts.next().unwrap_or("");
                let bytes = super::pack_format::string_to_bytes(parts.next().unwrap_or(""));
                let sent = if path.is_empty() {
                    socket.send(&bytes)
                } else {
                    socket.send_to(&bytes, path)
                };
                let sent = sent.map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        errno_class(&problem),
                        &format!("send: {problem}"),
                        position,
                    )
                })?;
                Ok(Object::Int(sent as i64))
            }
            "unix_dgram_receive" => {
                let wanted = if port > 0 { port as usize } else { 65536 };
                let mut buffer = vec![0u8; wanted];
                let mut deadline = std::time::Instant::now() + WAIT_LIMIT;
                let waited_enough = std::time::Instant::now() + WAIT_CEILING;
                let read = loop {
                    let socket = self
                        .open_sockets
                        .unix_datagrams
                        .get(&handle)
                        .ok_or_else(|| refuse("receive on a closed socket".to_string()))?;
                    match socket.recv_from(&mut buffer) {
                        Ok(held) => break held,
                        Err(problem)
                            if matches!(
                                problem.kind(),
                                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                            ) =>
                        {
                            // A thread that could still write is given its
                            // turn, and the wait is given more time for
                            // having handed one over, up to a ceiling that a
                            // wait nothing can ever end still reaches.
                            if std::time::Instant::now() >= deadline {
                                return Err(timed_out(position));
                            }
                            self.wait_for_other_threads(position);
                            self.raise_if_thread_killed(position)?;
                            if self.other_threads_are_waiting() {
                                deadline =
                                    (std::time::Instant::now() + WAIT_LIMIT).min(waited_enough);
                            }
                        }
                        Err(problem) => {
                            return Err(crate::vm::errors::simple_exception(
                                errno_class(&problem),
                                &format!("recv: {problem}"),
                                position,
                            ));
                        }
                    }
                };
                let (read, from) = read;
                buffer.truncate(read);
                Ok(Object::array(vec![
                    super::pack_format::bytes_to_string(&buffer),
                    match from.as_pathname() {
                        Some(path) => Object::string(path.display().to_string()),
                        None => Object::string(String::new()),
                    },
                ]))
            }
            // Which kind of socket a handle names, so one taken up from a
            // file descriptor reports the family and the kind it really is.
            "kind_of_handle" => {
                let named = if self.open_sockets.listeners.contains_key(&handle)
                    || self.open_sockets.streams.contains_key(&handle)
                {
                    "inet_stream"
                } else if self.open_sockets.unix_listeners.contains_key(&handle)
                    || self.open_sockets.unix_streams.contains_key(&handle)
                {
                    "unix_stream"
                } else if self.open_sockets.datagrams.contains_key(&handle) {
                    "inet_datagram"
                } else if self.open_sockets.unix_datagrams.contains_key(&handle) {
                    "unix_datagram"
                } else {
                    "unknown"
                };
                Ok(Object::string(named))
            }
            // ── sockets named by a path in the file system ────────────────
            "unix_listen" => {
                // A name something is already listening under cannot be bound
                // again, which is what the operating system reports and what
                // Ruby raises before the file is touched.
                if std::path::Path::new(&text).exists() {
                    return Err(crate::vm::errors::simple_exception(
                        "Errno::EADDRINUSE",
                        &format!("Address already in use - bind(2) for {text}"),
                        position,
                    ));
                }
                let held = std::os::unix::net::UnixListener::bind(&text)
                    .map_err(|problem| refuse(format!("bind({text}): {problem}")))?;
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
                let _ = held.set_nonblocking(true);
                self.open_sockets.unix_listeners.insert(named, held);
                Ok(Object::Int(named as i64))
            }
            "unix_connect" => {
                let held = std::os::unix::net::UnixStream::connect(&text).map_err(|problem| {
                    // A name with nothing listening under it is reported the
                    // way the operating system reports it, which for a path
                    // that is not there is a missing file.
                    let named = match problem.kind() {
                        std::io::ErrorKind::NotFound => "Errno::ENOENT",
                        _ => "Errno::ECONNREFUSED",
                    };
                    crate::vm::errors::simple_exception(
                        named,
                        &format!("connect({text}): {problem}"),
                        position,
                    )
                })?;
                let _ = held.set_read_timeout(Some(POLL_LIMIT));
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
                let _ = held.set_nonblocking(true);
                self.open_sockets.unix_streams.insert(named, held);
                Ok(Object::Int(named as i64))
            }
            // One try at taking a waiting connection, answering nil when
            // there is none rather than waiting for one.
            "unix_accept_now" => {
                let listener = self
                    .open_sockets
                    .unix_listeners
                    .get(&handle)
                    .ok_or_else(|| refuse("accept on a closed listener".to_string()))?;
                let _ = listener.set_nonblocking(true);
                let taken = listener.accept();
                let _ = listener.set_nonblocking(false);
                match taken {
                    Ok((stream, _)) => {
                        let _ = stream.set_read_timeout(Some(POLL_LIMIT));
                        let named = self.open_sockets.next;
                        self.open_sockets.next += 1;
                        let _ = stream.set_nonblocking(true);
                        self.open_sockets.unix_streams.insert(named, stream);
                        Ok(Object::Int(named as i64))
                    }
                    Err(problem) if problem.kind() == std::io::ErrorKind::WouldBlock => {
                        Ok(Object::Nil)
                    }
                    Err(problem) => Err(refuse(format!("accept: {problem}"))),
                }
            }
            "accept_now" => {
                let listener = self
                    .open_sockets
                    .listeners
                    .get(&handle)
                    .ok_or_else(|| refuse("accept on a closed listener".to_string()))?;
                let _ = listener.set_nonblocking(true);
                let taken = listener.accept();
                let _ = listener.set_nonblocking(false);
                match taken {
                    Ok((stream, _)) => {
                        let _ = stream.set_read_timeout(Some(POLL_LIMIT));
                        let named = self.open_sockets.next;
                        self.open_sockets.next += 1;
                        let _ = stream.set_nonblocking(true);
                        self.open_sockets.streams.insert(named, stream);
                        Ok(Object::Int(named as i64))
                    }
                    Err(problem) if problem.kind() == std::io::ErrorKind::WouldBlock => {
                        Ok(Object::Nil)
                    }
                    Err(problem) => Err(refuse(format!("accept: {problem}"))),
                }
            }
            "unix_accept" => {
                let mut deadline = std::time::Instant::now() + WAIT_LIMIT;
                let waited_enough = std::time::Instant::now() + WAIT_CEILING;
                let stream = loop {
                    let listener = self
                        .open_sockets
                        .unix_listeners
                        .get(&handle)
                        .ok_or_else(|| refuse("accept on a closed listener".to_string()))?;
                    let _ = listener.set_waiting(false);
                    let taken = listener.try_once();
                    let _ = listener.set_waiting(true);
                    match taken {
                        Ok((held, _)) => break held,
                        Err(problem) if problem.kind() == std::io::ErrorKind::WouldBlock => {
                            // A thread that could still write is given its
                            // turn, and the wait is given more time for
                            // having handed one over, up to a ceiling that a
                            // wait nothing can ever end still reaches.
                            if std::time::Instant::now() >= deadline {
                                return Err(timed_out(position));
                            }
                            self.wait_for_other_threads(position);
                            self.raise_if_thread_killed(position)?;
                            if self.other_threads_are_waiting() {
                                deadline =
                                    (std::time::Instant::now() + WAIT_LIMIT).min(waited_enough);
                            }
                        }
                        Err(problem) => {
                            return Err(crate::vm::errors::simple_exception(
                                "Errno::ECONNREFUSED",
                                &format!("accept: {problem}"),
                                position,
                            ));
                        }
                    }
                };
                let _ = stream.set_read_timeout(Some(POLL_LIMIT));
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
                let _ = stream.set_nonblocking(true);
                self.open_sockets.unix_streams.insert(named, stream);
                Ok(Object::Int(named as i64))
            }
            "unix_write" => {
                let stream = self
                    .open_sockets
                    .unix_streams
                    .get_mut(&handle)
                    .ok_or_else(|| refuse("write to a closed connection".to_string()))?;
                let bytes = super::pack_format::string_to_bytes(&text);
                stream
                    .write_all(&bytes)
                    .map_err(|problem| refuse(format!("write: {problem}")))?;
                Ok(Object::Int(bytes.len() as i64))
            }
            "unix_read" => {
                // `text` names the flags. Taking a look without taking the
                // message needs the system call rather than a plain read.
                let flags: i64 = text.parse().unwrap_or(0);
                if flags != 0
                    && let Some(descriptor) = self.socket_descriptor(handle)
                {
                    let wanted = if port > 0 { port as usize } else { 65536 };
                    let mut buffer = vec![0u8; wanted];
                    let read = unsafe {
                        libc::recv(
                            descriptor,
                            buffer.as_mut_ptr() as *mut libc::c_void,
                            wanted,
                            flags as libc::c_int,
                        )
                    };
                    if read < 0 {
                        let problem = std::io::Error::last_os_error();
                        if problem.kind() == std::io::ErrorKind::WouldBlock {
                            return Err(timed_out(position));
                        }
                        return Err(crate::vm::errors::simple_exception(
                            errno_class(&problem),
                            &format!("recv: {problem}"),
                            position,
                        ));
                    }
                    buffer.truncate(read as usize);
                    return Ok(super::pack_format::bytes_to_string(&buffer));
                }
                let stream = self
                    .open_sockets
                    .unix_streams
                    .get_mut(&handle)
                    .ok_or_else(|| refuse("read from a closed connection".to_string()))?;
                let wanted = if port > 0 { port as usize } else { 65536 };
                let mut buffer = vec![0u8; wanted];
                let read = stream.read(&mut buffer).map_err(|problem| {
                    if problem.kind() == std::io::ErrorKind::WouldBlock
                        || problem.kind() == std::io::ErrorKind::TimedOut
                    {
                        timed_out(position)
                    } else {
                        refuse(format!("read: {problem}"))
                    }
                })?;
                buffer.truncate(read);
                Ok(super::pack_format::bytes_to_string(&buffer))
            }
            // Where a socket named by a path sits, and where the other end of
            // one does.
            "unix_address" | "unix_peer" => {
                let found = if *action.as_str() == *"unix_address" {
                    self.open_sockets
                        .unix_listeners
                        .get(&handle)
                        .and_then(|held| held.local_addr().ok())
                        .or_else(|| {
                            self.open_sockets
                                .unix_streams
                                .get(&handle)
                                .and_then(|held| held.local_addr().ok())
                        })
                } else {
                    // The name the other end answers to is read straight from
                    // the operating system, since the standard library
                    // reports an unnamed address for a client's own side.
                    if let Some(named) = self.socket_descriptor(handle).and_then(peer_path_of) {
                        return Ok(Object::string(named));
                    }
                    self.open_sockets
                        .unix_streams
                        .get(&handle)
                        .and_then(|held| held.peer_addr().ok())
                };
                Ok(
                    match found
                        .and_then(|held| held.as_pathname().map(|path| path.display().to_string()))
                    {
                        Some(held) => Object::string(held),
                        None => Object::string(String::new()),
                    },
                )
            }
            // ── sockets that send each message on its own ─────────────────
            "udp_open" => {
                let held =
                    std::net::UdpSocket::bind((text.as_str(), port as u16)).map_err(|problem| {
                        crate::vm::errors::simple_exception(
                            bind_errno_class(&problem),
                            &format!("bind({text}:{port}): {problem}"),
                            position,
                        )
                    })?;
                let _ = held.set_read_timeout(Some(POLL_LIMIT));
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
                let _ = held.set_nonblocking(true);
                self.open_sockets.datagrams.insert(named, held);
                Ok(Object::Int(named as i64))
            }
            "udp_connect" => {
                let socket = self
                    .open_sockets
                    .datagrams
                    .get(&handle)
                    .ok_or_else(|| refuse("connect on a closed socket".to_string()))?;
                socket
                    .connect((text.as_str(), port as u16))
                    .map_err(|problem| refuse(format!("connect({text}:{port}): {problem}")))?;
                Ok(Object::Nil)
            }
            "udp_send" => {
                let socket = self
                    .open_sockets
                    .datagrams
                    .get(&handle)
                    .ok_or_else(|| refuse("send on a closed socket".to_string()))?;
                let bytes = super::pack_format::string_to_bytes(&text);
                let sent = socket.send(&bytes).map_err(|problem| {
                    // A datagram larger than the network will carry is
                    // refused outright rather than split.
                    if problem.raw_os_error() == Some(libc::EMSGSIZE) {
                        crate::vm::errors::simple_exception(
                            "Errno::EMSGSIZE",
                            &format!("Message too long - send(2): {problem}"),
                            position,
                        )
                    } else {
                        refuse(format!("send: {problem}"))
                    }
                })?;
                Ok(Object::Int(sent as i64))
            }
            "udp_receive" => {
                // A receive waits for a message the way a read waits for
                // what is written, so everything else the program has to run
                // gets a turn until one arrives. `text` names the flags,
                // where only "take a look without taking it" changes what is
                // done.
                let flags: i64 = text.parse().unwrap_or(0);
                let peeking = flags & libc::MSG_PEEK as i64 != 0;
                // A receive told not to wait reports that nothing is there
                // rather than handing the turn over.
                let waiting = flags & libc::MSG_DONTWAIT as i64 == 0;
                let wanted = if port > 0 { port as usize } else { 65536 };
                let mut buffer = vec![0u8; wanted];
                let mut deadline = std::time::Instant::now() + WAIT_LIMIT;
                let waited_enough = std::time::Instant::now() + WAIT_CEILING;
                let (read, from) = loop {
                    let socket = self
                        .open_sockets
                        .datagrams
                        .get(&handle)
                        .ok_or_else(|| refuse("receive on a closed socket".to_string()))?;
                    let taken = if peeking {
                        socket.peek_from(&mut buffer)
                    } else {
                        socket.recv_from(&mut buffer)
                    };
                    match taken {
                        Ok(held) => break held,
                        Err(problem)
                            if problem.kind() == std::io::ErrorKind::WouldBlock
                                || problem.kind() == std::io::ErrorKind::TimedOut =>
                        {
                            if !waiting || std::time::Instant::now() >= deadline {
                                return Err(timed_out(position));
                            }
                            self.wait_for_other_threads(position);
                            self.raise_if_thread_killed(position)?;
                            // A turn handed to a thread that could still send
                            // buys more time, up to the ceiling.
                            if self.other_threads_are_waiting() {
                                deadline =
                                    (std::time::Instant::now() + WAIT_LIMIT).min(waited_enough);
                            }
                        }
                        Err(problem) => {
                            return Err(crate::vm::errors::simple_exception(
                                errno_class(&problem),
                                &format!("recv: {problem}"),
                                position,
                            ));
                        }
                    }
                };
                buffer.truncate(read);
                Ok(Object::array(vec![
                    super::pack_format::bytes_to_string(&buffer),
                    Object::string(from.ip().to_string()),
                    Object::Int(i64::from(from.port())),
                ]))
            }
            "udp_address" => Ok(
                match self
                    .open_sockets
                    .datagrams
                    .get(&handle)
                    .and_then(|held| held.local_addr().ok())
                {
                    Some(held) => Object::array(vec![
                        Object::string(held.ip().to_string()),
                        Object::Int(i64::from(held.port())),
                    ]),
                    None => Object::Nil,
                },
            ),
            // Where a socket carrying each message on its own has been
            // pointed, which is nothing until it has been connected.
            "udp_peer" => Ok(
                match self
                    .open_sockets
                    .datagrams
                    .get(&handle)
                    .and_then(|held| held.peer_addr().ok())
                {
                    Some(held) => Object::array(vec![
                        Object::string(held.ip().to_string()),
                        Object::Int(i64::from(held.port())),
                    ]),
                    None => Object::Nil,
                },
            ),
            other => Err(MetorexError::runtime_error(
                format!("unknown socket action {other}"),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }
}

impl crate::vm::VirtualMachine {
    /// The descriptor behind a handle, whichever of the kinds of socket it
    /// names.
    fn socket_descriptor(&self, handle: u64) -> Option<i32> {
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

/// The name of the Errno class that stands for what the operating system
/// reported, so a program rescues the error it would rescue in Ruby.
fn errno_class(problem: &std::io::Error) -> &'static str {
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
        _ => "Errno::ECONNREFUSED",
    }
}

/// The name of the Errno class a failed bind stands for. A bind reports
/// addresses the machine does not answer on and ones already taken apart
/// from anything else that went wrong.
fn bind_errno_class(problem: &std::io::Error) -> &'static str {
    match problem.raw_os_error() {
        Some(libc::EADDRNOTAVAIL) => "Errno::EADDRNOTAVAIL",
        Some(libc::EADDRINUSE) => "Errno::EADDRINUSE",
        other => errno_class(&std::io::Error::from_raw_os_error(other.unwrap_or(0))),
    }
}

/// The path the other end of a socket answers to, read from the operating
/// system. Nothing when the other end has no name of its own.
fn peer_path_of(descriptor: i32) -> Option<String> {
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
fn point_to_point_address(entry: &libc::ifaddrs) -> *const libc::sockaddr {
    entry.ifa_ifu
}

#[cfg(not(target_os = "linux"))]
fn point_to_point_address(entry: &libc::ifaddrs) -> *const libc::sockaddr {
    entry.ifa_dstaddr
}

fn address_of_sockaddr(held: *const libc::sockaddr) -> Object {
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

/// Send one descriptor over a socket named by a path, alongside a single
/// byte, which is what the message needs to carry the control data.
fn send_descriptor(socket: i32, descriptor: i32) -> isize {
    let mut byte = [0u8; 1];
    let mut piece = libc::iovec {
        iov_base: byte.as_mut_ptr() as *mut libc::c_void,
        iov_len: 1,
    };
    let space = unsafe { libc::CMSG_SPACE(std::mem::size_of::<libc::c_int>() as u32) } as usize;
    let mut control = vec![0u8; space];
    let mut message: libc::msghdr = unsafe { std::mem::zeroed() };
    message.msg_iov = &mut piece;
    message.msg_iovlen = 1;
    message.msg_control = control.as_mut_ptr() as *mut libc::c_void;
    message.msg_controllen = space as _;
    unsafe {
        let header = libc::CMSG_FIRSTHDR(&message);
        (*header).cmsg_level = libc::SOL_SOCKET;
        (*header).cmsg_type = libc::SCM_RIGHTS;
        (*header).cmsg_len = libc::CMSG_LEN(std::mem::size_of::<libc::c_int>() as u32) as _;
        std::ptr::write(libc::CMSG_DATA(header) as *mut libc::c_int, descriptor);
        libc::sendmsg(socket, &message, 0)
    }
}

/// Take one descriptor handed over a socket named by a path, or -1 when none
/// arrived.
fn receive_descriptor(socket: i32) -> i32 {
    let mut byte = [0u8; 1];
    let mut piece = libc::iovec {
        iov_base: byte.as_mut_ptr() as *mut libc::c_void,
        iov_len: 1,
    };
    let space = unsafe { libc::CMSG_SPACE(std::mem::size_of::<libc::c_int>() as u32) } as usize;
    let mut control = vec![0u8; space];
    let mut message: libc::msghdr = unsafe { std::mem::zeroed() };
    message.msg_iov = &mut piece;
    message.msg_iovlen = 1;
    message.msg_control = control.as_mut_ptr() as *mut libc::c_void;
    message.msg_controllen = space as _;
    unsafe {
        if libc::recvmsg(socket, &mut message, 0) < 0 {
            return -1;
        }
        let header = libc::CMSG_FIRSTHDR(&message);
        if header.is_null()
            || (*header).cmsg_level != libc::SOL_SOCKET
            || (*header).cmsg_type != libc::SCM_RIGHTS
        {
            return -1;
        }
        std::ptr::read(libc::CMSG_DATA(header) as *const libc::c_int)
    }
}

/// The number the operating system names a socket constant by. A name this
/// platform does not carry answers None, and the socket library keeps its own
/// number for that one.
fn socket_constant(name: &str) -> Option<i64> {
    let held = match name {
        "AF_UNSPEC" => libc::AF_UNSPEC,
        "AF_UNIX" | "AF_LOCAL" => libc::AF_UNIX,
        "AF_INET" => libc::AF_INET,
        "AF_INET6" => libc::AF_INET6,
        "PF_UNSPEC" => libc::PF_UNSPEC,
        "PF_UNIX" | "PF_LOCAL" => libc::PF_UNIX,
        "PF_INET" => libc::PF_INET,
        "PF_INET6" => libc::PF_INET6,
        "SOCK_STREAM" => libc::SOCK_STREAM,
        "SOCK_DGRAM" => libc::SOCK_DGRAM,
        "SOCK_RAW" => libc::SOCK_RAW,
        "SOCK_RDM" => libc::SOCK_RDM,
        "SOCK_SEQPACKET" => libc::SOCK_SEQPACKET,
        "IPPROTO_IP" => libc::IPPROTO_IP,
        "IPPROTO_ICMP" => libc::IPPROTO_ICMP,
        "IPPROTO_TCP" => libc::IPPROTO_TCP,
        "IPPROTO_UDP" => libc::IPPROTO_UDP,
        "IPPROTO_IPV6" => libc::IPPROTO_IPV6,
        "IPPROTO_RAW" => libc::IPPROTO_RAW,
        "SOL_SOCKET" => libc::SOL_SOCKET,
        "SO_DEBUG" => libc::SO_DEBUG,
        "SO_ACCEPTCONN" => libc::SO_ACCEPTCONN,
        "SO_REUSEADDR" => libc::SO_REUSEADDR,
        "SO_KEEPALIVE" => libc::SO_KEEPALIVE,
        "SO_DONTROUTE" => libc::SO_DONTROUTE,
        "SO_BROADCAST" => libc::SO_BROADCAST,
        "SO_OOBINLINE" => libc::SO_OOBINLINE,
        "SO_REUSEPORT" => libc::SO_REUSEPORT,
        "SO_LINGER" => libc::SO_LINGER,
        "SO_SNDBUF" => libc::SO_SNDBUF,
        "SO_RCVBUF" => libc::SO_RCVBUF,
        "SO_TYPE" => libc::SO_TYPE,
        "SO_ERROR" => libc::SO_ERROR,
        "SO_SNDLOWAT" => libc::SO_SNDLOWAT,
        "SO_RCVLOWAT" => libc::SO_RCVLOWAT,
        "SO_SNDTIMEO" => libc::SO_SNDTIMEO,
        "SO_RCVTIMEO" => libc::SO_RCVTIMEO,
        "MSG_OOB" => libc::MSG_OOB,
        "MSG_PEEK" => libc::MSG_PEEK,
        "MSG_DONTROUTE" => libc::MSG_DONTROUTE,
        "MSG_EOR" => libc::MSG_EOR,
        "MSG_TRUNC" => libc::MSG_TRUNC,
        "MSG_CTRUNC" => libc::MSG_CTRUNC,
        "MSG_WAITALL" => libc::MSG_WAITALL,
        "MSG_DONTWAIT" => libc::MSG_DONTWAIT,
        "TCP_NODELAY" => libc::TCP_NODELAY,
        "TCP_MAXSEG" => libc::TCP_MAXSEG,
        "SCM_RIGHTS" => libc::SCM_RIGHTS,
        "EAI_AGAIN" => libc::EAI_AGAIN,
        "EAI_BADFLAGS" => libc::EAI_BADFLAGS,
        "EAI_FAIL" => libc::EAI_FAIL,
        "EAI_FAMILY" => libc::EAI_FAMILY,
        "EAI_MEMORY" => libc::EAI_MEMORY,
        "EAI_NONAME" => libc::EAI_NONAME,
        "EAI_SERVICE" => libc::EAI_SERVICE,
        "EAI_SOCKTYPE" => libc::EAI_SOCKTYPE,
        "EAI_SYSTEM" => libc::EAI_SYSTEM,
        "EAI_OVERFLOW" => libc::EAI_OVERFLOW,
        "IP_TTL" => libc::IP_TTL,
        "IP_MULTICAST_TTL" => libc::IP_MULTICAST_TTL,
        "IP_MULTICAST_LOOP" => libc::IP_MULTICAST_LOOP,
        "IP_ADD_MEMBERSHIP" => libc::IP_ADD_MEMBERSHIP,
        "IP_DROP_MEMBERSHIP" => libc::IP_DROP_MEMBERSHIP,
        "IPV6_V6ONLY" => libc::IPV6_V6ONLY,
        // How an address named by a path is laid out, which is not a
        // constant the C library names but a shape it settles.
        "SOCKADDR_UN_SIZE" => {
            return Some(std::mem::size_of::<libc::sockaddr_un>() as i64);
        }
        "SOCKADDR_UN_PATH_MAX" => {
            let held = libc::sockaddr_un {
                ..unsafe { std::mem::zeroed() }
            };
            return Some(held.sun_path.len() as i64 - 1);
        }
        // BSD writes the length of an address in front of the family it
        // names, where Linux writes the family alone.
        "SOCKADDR_HAS_LEN" => return Some(i64::from(!cfg!(target_os = "linux"))),
        _ => return None,
    };
    Some(i64::from(held))
}

/// An address written the way the system call takes it, with the number of
/// bytes it fills.
fn sockaddr_of(held: &std::net::SocketAddr) -> (Vec<u8>, libc::socklen_t) {
    match held {
        std::net::SocketAddr::V4(address) => {
            let mut written = unsafe { std::mem::zeroed::<libc::sockaddr_in>() };
            written.sin_family = libc::AF_INET as libc::sa_family_t;
            written.sin_port = address.port().to_be();
            written.sin_addr.s_addr = u32::from_ne_bytes(address.ip().octets());
            let size = std::mem::size_of::<libc::sockaddr_in>();
            let bytes =
                unsafe { std::slice::from_raw_parts((&raw const written).cast::<u8>(), size) };
            (bytes.to_vec(), size as libc::socklen_t)
        }
        std::net::SocketAddr::V6(address) => {
            let mut written = unsafe { std::mem::zeroed::<libc::sockaddr_in6>() };
            written.sin6_family = libc::AF_INET6 as libc::sa_family_t;
            written.sin6_port = address.port().to_be();
            written.sin6_addr.s6_addr = address.ip().octets();
            let size = std::mem::size_of::<libc::sockaddr_in6>();
            let bytes =
                unsafe { std::slice::from_raw_parts((&raw const written).cast::<u8>(), size) };
            (bytes.to_vec(), size as libc::socklen_t)
        }
    }
}
