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
    pub(crate) next: u64,
}

/// How long a read or an accept waits before giving up. Metorex runs one
/// thread, so a socket that would block forever has nobody left to write to
/// it, and waiting is what a deadlock looks like from the inside.
const WAIT_LIMIT: std::time::Duration = std::time::Duration::from_secs(2);

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

fn wait_for_connection<L: Accepts>(
    listener: &L,
    position: Position,
) -> Result<L::Stream, MetorexError> {
    let _ = listener.set_waiting(false);
    let deadline = std::time::Instant::now() + WAIT_LIMIT;
    loop {
        match listener.try_once() {
            Ok(held) => {
                let _ = listener.set_waiting(true);
                return Ok(held);
            }
            Err(problem) if problem.kind() == std::io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() >= deadline {
                    let _ = listener.set_waiting(true);
                    return Err(timed_out(position));
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(problem) => {
                let _ = listener.set_waiting(true);
                return Err(crate::vm::errors::simple_exception(
                    "Errno::ECONNREFUSED",
                    &format!("accept: {problem}"),
                    position,
                ));
            }
        }
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
                let held = std::net::TcpListener::bind((text.as_str(), port as u16))
                    .map_err(|problem| refuse(format!("bind({text}:{port}): {problem}")))?;
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
                self.open_sockets.listeners.insert(named, held);
                Ok(Object::Int(named as i64))
            }
            // Reach a name and a port that something is listening on.
            "connect" => {
                let held = std::net::TcpStream::connect((text.as_str(), port as u16))
                    .map_err(|problem| refuse(format!("connect({text}:{port}): {problem}")))?;
                let _ = held.set_read_timeout(Some(WAIT_LIMIT));
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
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
                let listener = self
                    .open_sockets
                    .listeners
                    .get(&handle)
                    .ok_or_else(|| refuse("accept on a closed listener".to_string()))?;
                let (stream, _) = wait_for_connection(listener, position)?;
                let _ = stream.set_read_timeout(Some(WAIT_LIMIT));
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
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
                stream
                    .write_all(&bytes)
                    .map_err(|problem| refuse(format!("write: {problem}")))?;
                Ok(Object::Int(bytes.len() as i64))
            }
            // Read what the other end sent, up to the count asked for.
            "read" => {
                let stream = self
                    .open_sockets
                    .streams
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
            "fileno" => {
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
                self.open_sockets.unix_listeners.insert(named, held);
                Ok(Object::Int(named as i64))
            }
            "unix_connect" => {
                let held = std::os::unix::net::UnixStream::connect(&text)
                    .map_err(|problem| refuse(format!("connect({text}): {problem}")))?;
                let _ = held.set_read_timeout(Some(WAIT_LIMIT));
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
                self.open_sockets.unix_streams.insert(named, held);
                Ok(Object::Int(named as i64))
            }
            "unix_accept" => {
                let listener = self
                    .open_sockets
                    .unix_listeners
                    .get(&handle)
                    .ok_or_else(|| refuse("accept on a closed listener".to_string()))?;
                let (stream, _) = wait_for_connection(listener, position)?;
                let _ = stream.set_read_timeout(Some(WAIT_LIMIT));
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
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
                let held = std::net::UdpSocket::bind((text.as_str(), port as u16))
                    .map_err(|problem| refuse(format!("bind({text}:{port}): {problem}")))?;
                let _ = held.set_read_timeout(Some(WAIT_LIMIT));
                let named = self.open_sockets.next;
                self.open_sockets.next += 1;
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
                let socket = self
                    .open_sockets
                    .datagrams
                    .get(&handle)
                    .ok_or_else(|| refuse("receive on a closed socket".to_string()))?;
                let wanted = if port > 0 { port as usize } else { 65536 };
                let mut buffer = vec![0u8; wanted];
                let (read, from) = socket.recv_from(&mut buffer).map_err(|problem| {
                    if problem.kind() == std::io::ErrorKind::WouldBlock
                        || problem.kind() == std::io::ErrorKind::TimedOut
                    {
                        timed_out(position)
                    } else {
                        refuse(format!("recv: {problem}"))
                    }
                })?;
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
            other => Err(MetorexError::runtime_error(
                format!("unknown socket action {other}"),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }
}
