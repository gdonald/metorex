// Opening a socket, waiting on it, and where each end sits.

use super::*;

impl VirtualMachine {
    /// Take a name and a port and start listening there.
    pub(crate) fn socket_listen(
        &mut self,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let held =
            std::net::TcpListener::bind((text.as_str(), port as u16)).map_err(|problem| {
                crate::vm::errors::simple_exception(
                    bind_errno_class(&problem),
                    &format!("bind({text}:{port}): {problem}"),
                    position,
                )
            })?;
        let named = self.open_sockets.next;
        self.open_sockets.next += 1;
        let _ = held.set_nonblocking(true);
        self.open_sockets.listeners.insert(named, held);
        Ok(Object::Int(named as i64))
    }

    /// Take a name and a port without yet waiting for anything to
    /// arrive. A socket that is bound and no more refuses what tries
    /// to reach it, which is what a connection to one is told.
    pub(crate) fn socket_bind_only(
        &mut self,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
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

    /// Start waiting for connections on a socket already bound.
    pub(crate) fn socket_listen_on(
        &mut self,
        handle: u64,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(descriptor) = self.socket_descriptor(handle) else {
            return Err(refused(position, "listen on a closed socket".to_string()));
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

    /// Reach a name and a port that something is listening on.
    pub(crate) fn socket_connect(
        &mut self,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let held = std::net::TcpStream::connect((text.as_str(), port as u16))
            .map_err(|problem| refused(position, format!("connect({text}:{port}): {problem}")))?;
        let _ = held.set_read_timeout(Some(POLL_LIMIT));
        let named = self.open_sockets.next;
        self.open_sockets.next += 1;
        let _ = held.set_nonblocking(true);
        self.open_sockets.streams.insert(named, held);
        Ok(Object::Int(named as i64))
    }

    /// Where a listener or a connection sits.
    pub(crate) fn socket_own_address(&mut self, handle: u64) -> Result<Object, MetorexError> {
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

    /// Where the other end of a connection sits.
    pub(crate) fn socket_peer_address(&mut self, handle: u64) -> Result<Object, MetorexError> {
        Ok(
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
        )
    }

    /// Take the next connection something has made to a listener.
    pub(crate) fn socket_accept(
        &mut self,
        handle: u64,
        position: Position,
    ) -> Result<Object, MetorexError> {
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
                .ok_or_else(|| refused(position, "accept on a closed listener".to_string()))?;
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
                        deadline = (std::time::Instant::now() + WAIT_LIMIT).min(waited_enough);
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

    pub(crate) fn socket_accept_now(
        &mut self,
        handle: u64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let listener = self
            .open_sockets
            .listeners
            .get(&handle)
            .ok_or_else(|| refused(position, "accept on a closed listener".to_string()))?;
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
            Err(problem) if problem.kind() == std::io::ErrorKind::WouldBlock => Ok(Object::Nil),
            Err(problem) => Err(refused(position, format!("accept: {problem}"))),
        }
    }
}
