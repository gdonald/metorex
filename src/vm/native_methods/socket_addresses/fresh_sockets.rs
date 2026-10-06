// A stream socket made by `Socket.new` before it is bound or connected,
// which holds a descriptor of its own from the start, as one made by
// socket(2) does.

use super::*;
use std::os::unix::io::{FromRawFd as _, IntoRawFd as _};

impl VirtualMachine {
    /// Open a stream socket in the family named by `text` ("4" or "6"),
    /// kept with the listeners until it is connected, since a listener is
    /// what a socket that may yet listen is held as.
    pub(crate) fn socket_tcp_open(
        &mut self,
        text: String,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let family = if text == "6" {
            libc::AF_INET6
        } else {
            libc::AF_INET
        };
        // SAFETY: the descriptor is taken over by the listener built from
        // it, which closes it when it goes.
        let held = unsafe {
            let descriptor = libc::socket(family, libc::SOCK_STREAM, 0);
            if descriptor < 0 {
                let problem = std::io::Error::last_os_error();
                return Err(crate::vm::errors::simple_exception(
                    errno_class(&problem),
                    &format!("{} - socket(2)", os_reason(&problem)),
                    position,
                ));
            }
            std::net::TcpListener::from_raw_fd(descriptor)
        };
        let _ = held.set_nonblocking(true);
        let named = self.open_sockets.next;
        self.open_sockets.next += 1;
        self.open_sockets.listeners.insert(named, held);
        Ok(Object::Int(named as i64))
    }

    /// Open a datagram socket in the family named by `text` ("4" or "6"),
    /// bound to nothing until it is bound or sends.
    pub(crate) fn socket_udp_fresh(
        &mut self,
        text: String,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let family = if text == "6" {
            libc::AF_INET6
        } else {
            libc::AF_INET
        };
        // SAFETY: the descriptor is taken over by the socket built from it,
        // which closes it when it goes.
        let held = unsafe {
            let descriptor = libc::socket(family, libc::SOCK_DGRAM, 0);
            if descriptor < 0 {
                let problem = std::io::Error::last_os_error();
                return Err(crate::vm::errors::simple_exception(
                    errno_class(&problem),
                    &format!("{} - socket(2)", os_reason(&problem)),
                    position,
                ));
            }
            std::net::UdpSocket::from_raw_fd(descriptor)
        };
        let _ = held.set_read_timeout(Some(POLL_LIMIT));
        let _ = held.set_nonblocking(true);
        let named = self.open_sockets.next;
        self.open_sockets.next += 1;
        self.open_sockets.datagrams.insert(named, held);
        Ok(Object::Int(named as i64))
    }

    /// Bind an open socket to the address and port named.
    pub(crate) fn socket_bind_on(
        &mut self,
        handle: u64,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let wanted = self.looked_up_address(&text, port, position)?;
        let Some(descriptor) = self.socket_descriptor(handle) else {
            return Err(refused(position, "bind on a closed socket".to_string()));
        };
        let (address, length) = sockaddr_of(&wanted);
        // SAFETY: the descriptor is one this program holds open, and the
        // address is `length` bytes long.
        if unsafe { libc::bind(descriptor, address.as_ptr().cast(), length) } < 0 {
            let problem = std::io::Error::last_os_error();
            return Err(crate::vm::errors::simple_exception(
                bind_errno_class(&problem),
                &format!("{} - bind(2) for {text}:{port}", os_reason(&problem)),
                position,
            ));
        }
        Ok(Object::Int(0))
    }

    /// Connect an open stream socket to the address and port named. The
    /// connection is made without blocking, and other threads take turns
    /// while it is under way.
    pub(crate) fn socket_connect_on(
        &mut self,
        handle: u64,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let wanted = self.looked_up_address(&text, port, position)?;
        let Some(descriptor) = self.socket_descriptor(handle) else {
            return Err(refused(position, "connect on a closed socket".to_string()));
        };
        let failed = |problem: std::io::Error| {
            crate::vm::errors::simple_exception(
                errno_class(&problem),
                &format!("{} - connect(2) for {text}:{port}", os_reason(&problem)),
                position,
            )
        };
        let (address, length) = sockaddr_of(&wanted);
        // SAFETY: the descriptor is one this program holds open, and the
        // address is `length` bytes long.
        let started = unsafe { libc::connect(descriptor, address.as_ptr().cast(), length) };
        if started < 0 {
            let problem = std::io::Error::last_os_error();
            if problem.raw_os_error() != Some(libc::EINPROGRESS) {
                return Err(failed(problem));
            }
            let mut deadline = std::time::Instant::now() + WAIT_LIMIT;
            let waited_enough = std::time::Instant::now() + WAIT_CEILING;
            while !descriptor_writable(descriptor) {
                if std::time::Instant::now() >= deadline {
                    return Err(failed(std::io::Error::from_raw_os_error(libc::ETIMEDOUT)));
                }
                self.wait_for_other_threads(position);
                self.raise_if_thread_killed(position)?;
                if self.other_threads_are_waiting() {
                    deadline = (std::time::Instant::now() + WAIT_LIMIT).min(waited_enough);
                }
            }
            let mut pending: libc::c_int = 0;
            let mut size = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
            // SAFETY: SO_ERROR answers one int into `pending`.
            unsafe {
                libc::getsockopt(
                    descriptor,
                    libc::SOL_SOCKET,
                    libc::SO_ERROR,
                    (&mut pending as *mut libc::c_int).cast(),
                    &mut size,
                )
            };
            if pending != 0 {
                return Err(failed(std::io::Error::from_raw_os_error(pending)));
            }
        }
        self.hold_as_connection(handle);
        Ok(Object::Int(0))
    }
}

impl VirtualMachine {
    /// One try at connecting an open stream socket without waiting.
    /// Answers 0 once it is connected, 1 while the connection is under
    /// way, and 2 when it was connected already.
    pub(crate) fn socket_connect_now(
        &mut self,
        handle: u64,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let wanted = self.looked_up_address(&text, port, position)?;
        let Some(descriptor) = self.socket_descriptor(handle) else {
            return Err(refused(position, "connect on a closed socket".to_string()));
        };
        let (address, length) = sockaddr_of(&wanted);
        // SAFETY: the descriptor is one this program holds open, and the
        // address is `length` bytes long.
        let started = unsafe { libc::connect(descriptor, address.as_ptr().cast(), length) };
        let answer = if started == 0 {
            0
        } else {
            let problem = std::io::Error::last_os_error();
            match problem.raw_os_error() {
                Some(code) if code == libc::EINPROGRESS || code == libc::EALREADY => {
                    return Ok(Object::Int(1));
                }
                Some(code) if code == libc::EISCONN => 2,
                _ => {
                    return Err(crate::vm::errors::simple_exception(
                        errno_class(&problem),
                        &format!("{} - connect(2) for {text}:{port}", os_reason(&problem)),
                        position,
                    ));
                }
            }
        };
        self.hold_as_connection(handle);
        Ok(Object::Int(answer))
    }

    /// Move a socket that has connected from the listeners to the
    /// connections, which is how a connected socket is held.
    fn hold_as_connection(&mut self, handle: u64) {
        if let Some(listener) = self.open_sockets.listeners.remove(&handle) {
            // SAFETY: the descriptor came out of the listener, which no
            // longer owns it.
            let stream = unsafe { std::net::TcpStream::from_raw_fd(listener.into_raw_fd()) };
            let _ = stream.set_read_timeout(Some(POLL_LIMIT));
            let _ = stream.set_nonblocking(true);
            self.open_sockets.streams.insert(handle, stream);
        }
    }
}

/// Whether a descriptor has room to write right now.
fn descriptor_writable(descriptor: libc::c_int) -> bool {
    let mut asked = libc::pollfd {
        fd: descriptor,
        events: libc::POLLOUT,
        revents: 0,
    };
    // SAFETY: one entry, asked without waiting.
    unsafe { libc::poll(&mut asked, 1, 0) > 0 && asked.revents != 0 }
}

impl VirtualMachine {
    /// The address and port named, resolved the way a bind or a connect
    /// takes them. A name is looked up while the other threads run.
    fn looked_up_address(
        &mut self,
        text: &str,
        port: i64,
        position: Position,
    ) -> Result<std::net::SocketAddr, MetorexError> {
        let named = text.to_string();
        let found = self.run_beside_threads(
            move || {
                use std::net::ToSocketAddrs as _;
                (named.as_str(), port as u16)
                    .to_socket_addrs()
                    .ok()
                    .and_then(|mut found| found.next())
            },
            position,
        );
        found.ok_or_else(|| {
            crate::vm::errors::simple_exception(
                "SocketError",
                &format!("getaddrinfo: {text}: nodename nor servname provided, or not known"),
                position,
            )
        })
    }
}

/// What the operating system says went wrong, without the error number
/// Rust adds after it.
fn os_reason(problem: &std::io::Error) -> String {
    let described = problem.to_string();
    described
        .split(" (os error")
        .next()
        .unwrap_or(&described)
        .to_string()
}

impl VirtualMachine {
    /// Look an address up the way an Addrinfo built from an array is:
    /// getaddrinfo(3) told the host and the port are numeric, with the
    /// family, the socket type, and the protocol as hints. `text` holds the
    /// host and the three hints with a NUL between each. Answers the first
    /// result as its family, socket type, protocol, address, and port, or
    /// the resolver's error code and its reason when there is none.
    pub(crate) fn socket_numeric_addrinfo(
        &mut self,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut parts = text.split('\0');
        let host = parts.next().unwrap_or_default().to_string();
        let mut hint =
            || -> libc::c_int { parts.next().and_then(|held| held.parse().ok()).unwrap_or(0) };
        let (family, socktype, protocol) = (hint(), hint(), hint());
        if let Some(found) = numeric_lookup(&host, port, family, socktype, protocol) {
            return Ok(found);
        }
        let failed = |message: String| {
            crate::vm::errors::simple_exception("SocketError", &message, position)
        };
        let node = std::ffi::CString::new(host)
            .map_err(|_| failed("getaddrinfo: string contains null byte".to_string()))?;
        let service = std::ffi::CString::new(port.to_string()).unwrap_or_default();
        // SAFETY: the hints are zeroed and then filled, the strings live for
        // the call, and the result is freed once it has been read.
        unsafe {
            let mut hints: libc::addrinfo = std::mem::zeroed();
            hints.ai_family = family;
            hints.ai_socktype = socktype;
            hints.ai_protocol = protocol;
            hints.ai_flags = libc::AI_NUMERICHOST | libc::AI_NUMERICSERV;
            let mut found: *mut libc::addrinfo = std::ptr::null_mut();
            let answered = libc::getaddrinfo(node.as_ptr(), service.as_ptr(), &hints, &mut found);
            if answered != 0 {
                let reason = std::ffi::CStr::from_ptr(libc::gai_strerror(answered))
                    .to_string_lossy()
                    .to_string();
                return Ok(Object::array(vec![
                    Object::Int(i64::from(answered)),
                    Object::string(reason),
                ]));
            }
            let first = &*found;
            let (address, number) = match first.ai_family {
                libc::AF_INET6 => {
                    let held = &*(first.ai_addr as *const libc::sockaddr_in6);
                    (
                        std::net::Ipv6Addr::from(held.sin6_addr.s6_addr).to_string(),
                        u16::from_be(held.sin6_port),
                    )
                }
                _ => {
                    let held = &*(first.ai_addr as *const libc::sockaddr_in);
                    (
                        std::net::Ipv4Addr::from(u32::from_be(held.sin_addr.s_addr)).to_string(),
                        u16::from_be(held.sin_port),
                    )
                }
            };
            let answer = Object::array(vec![
                Object::Int(i64::from(first.ai_family)),
                Object::Int(i64::from(first.ai_socktype)),
                Object::Int(i64::from(first.ai_protocol)),
                Object::string(address),
                Object::Int(i64::from(number)),
            ]);
            libc::freeaddrinfo(found);
            Ok(answer)
        }
    }
}

/// The lookup Ruby makes on its own before asking the resolver, for an
/// address written as digits: the kinds of socket it pairs with each
/// protocol are STREAM with TCP, DGRAM with UDP, and RAW with any protocol,
/// and the first that fits the hints is the answer. None when nothing fits,
/// and the resolver is asked instead.
fn numeric_lookup(
    host: &str,
    port: i64,
    family: libc::c_int,
    socktype: libc::c_int,
    protocol: libc::c_int,
) -> Option<Object> {
    const KINDS: [(libc::c_int, libc::c_int); 3] = [
        (libc::SOCK_STREAM, libc::IPPROTO_TCP),
        (libc::SOCK_DGRAM, libc::IPPROTO_UDP),
        (libc::SOCK_RAW, 0),
    ];
    let spelled_as = |allowed: &str| host.chars().all(|held| allowed.contains(held));
    let found_family = if (family == libc::PF_UNSPEC || family == libc::PF_INET6)
        && spelled_as("0123456789abcdefABCDEF.:")
        && host.parse::<std::net::Ipv6Addr>().is_ok()
    {
        libc::PF_INET6
    } else if (family == libc::PF_UNSPEC || family == libc::PF_INET)
        && spelled_as("0123456789.")
        && host.parse::<std::net::Ipv4Addr>().is_ok()
    {
        libc::PF_INET
    } else {
        return None;
    };
    let (kind, _) = KINDS.iter().find(|(kind, paired)| {
        (socktype == 0 || socktype == *kind)
            && (protocol == 0 || *paired == 0 || protocol == *paired)
    })?;
    Some(Object::array(vec![
        Object::Int(i64::from(found_family)),
        Object::Int(i64::from(*kind)),
        Object::Int(i64::from(protocol)),
        Object::string(host.to_string()),
        Object::Int(i64::from(port as u16)),
    ]))
}
