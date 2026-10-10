// Sockets named by a path rather than by an address.

use super::*;

impl VirtualMachine {
    /// A socket named by a path that carries each message on its
    /// own. An empty path opens one with no name of its own.
    pub(crate) fn socket_unix_datagram_open(
        &mut self,
        text: String,
        position: Position,
    ) -> Result<Object, MetorexError> {
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

    pub(crate) fn socket_unix_datagram_send(
        &mut self,
        handle: u64,
        text: String,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let socket = self
            .open_sockets
            .unix_datagrams
            .get(&handle)
            .ok_or_else(|| refused(position, "send on a closed socket".to_string()))?;
        let mut parts = text.splitn(2, '\u{0}');
        let path = parts.next().unwrap_or("");
        let bytes =
            crate::vm::native_methods::pack_format::string_to_bytes(parts.next().unwrap_or(""));
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

    pub(crate) fn socket_unix_datagram_receive(
        &mut self,
        handle: u64,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let wanted = if port > 0 { port as usize } else { 65536 };
        let mut buffer = vec![0u8; wanted];
        let mut deadline = std::time::Instant::now() + WAIT_LIMIT;
        let waited_enough = std::time::Instant::now() + WAIT_CEILING;
        let read = loop {
            let socket = self
                .open_sockets
                .unix_datagrams
                .get(&handle)
                .ok_or_else(|| refused(position, "receive on a closed socket".to_string()))?;
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
                        deadline = (std::time::Instant::now() + WAIT_LIMIT).min(waited_enough);
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
            crate::vm::native_methods::pack_format::bytes_to_string(&buffer),
            match from.as_pathname() {
                Some(path) => Object::string(path.display().to_string()),
                None => Object::string(String::new()),
            },
        ]))
    }

    /// ── sockets named by a path in the file system ────────────────
    pub(crate) fn socket_unix_listen(
        &mut self,
        text: String,
        position: Position,
    ) -> Result<Object, MetorexError> {
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
            .map_err(|problem| refused(position, format!("bind({text}): {problem}")))?;
        let named = self.open_sockets.next;
        self.open_sockets.next += 1;
        let _ = held.set_nonblocking(true);
        self.open_sockets.unix_listeners.insert(named, held);
        Ok(Object::Int(named as i64))
    }

    pub(crate) fn socket_unix_connect(
        &mut self,
        text: String,
        position: Position,
    ) -> Result<Object, MetorexError> {
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

    /// Two connected sockets made by `socketpair`, neither of which has a
    /// name, answered as the handles of the two ends.
    pub(crate) fn socket_unix_pair(&mut self, position: Position) -> Result<Object, MetorexError> {
        let (first, second) = std::os::unix::net::UnixStream::pair()
            .map_err(|problem| refused(position, format!("socketpair: {problem}")))?;
        let mut handles = Vec::new();
        for held in [first, second] {
            let _ = held.set_read_timeout(Some(POLL_LIMIT));
            let _ = held.set_nonblocking(true);
            let named = self.open_sockets.next;
            self.open_sockets.next += 1;
            self.open_sockets.unix_streams.insert(named, held);
            handles.push(Object::Int(named as i64));
        }
        Ok(Object::array(handles))
    }

    /// One try at taking a waiting connection, answering nil when
    /// there is none rather than waiting for one.
    pub(crate) fn socket_unix_accept_now(
        &mut self,
        handle: u64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let listener = self
            .open_sockets
            .unix_listeners
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
                self.open_sockets.unix_streams.insert(named, stream);
                Ok(Object::Int(named as i64))
            }
            Err(problem) if problem.kind() == std::io::ErrorKind::WouldBlock => Ok(Object::Nil),
            Err(problem) => Err(crate::vm::errors::simple_exception(
                errno_class(&problem),
                &format!(
                    "{} - accept(2)",
                    crate::vm::init::errno_description(problem.raw_os_error().unwrap_or(0))
                ),
                position,
            )),
        }
    }

    pub(crate) fn socket_unix_accept(
        &mut self,
        handle: u64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut deadline = std::time::Instant::now() + WAIT_LIMIT;
        let waited_enough = std::time::Instant::now() + WAIT_CEILING;
        let stream = loop {
            let listener = self
                .open_sockets
                .unix_listeners
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
                        errno_class(&problem),
                        &format!(
                            "{} - accept(2)",
                            crate::vm::init::errno_description(problem.raw_os_error().unwrap_or(0))
                        ),
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

    pub(crate) fn socket_unix_write(
        &mut self,
        handle: u64,
        text: String,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let stream = self
            .open_sockets
            .unix_streams
            .get_mut(&handle)
            .ok_or_else(|| refused(position, "write to a closed connection".to_string()))?;
        let bytes = crate::vm::native_methods::pack_format::string_to_bytes(&text);
        let written = super::transfer::write_what_fits(stream, &bytes).map_err(|problem| {
            crate::vm::errors::simple_exception(
                errno_class(&problem),
                &format!("write: {problem}"),
                position,
            )
        })?;
        Ok(Object::Int(written as i64))
    }

    pub(crate) fn socket_unix_read(
        &mut self,
        handle: u64,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
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
            return Ok(crate::vm::native_methods::pack_format::bytes_to_string(
                &buffer,
            ));
        }
        let wanted = if port > 0 { port as usize } else { 65536 };
        let mut buffer = vec![0u8; wanted];
        let read = self.read_waiting(position, |vm| {
            let stream = vm
                .open_sockets
                .unix_streams
                .get_mut(&handle)
                .ok_or_else(|| refused(position, "read from a closed connection".to_string()))?;
            Ok(stream.read(&mut buffer))
        })?;
        buffer.truncate(read);
        Ok(crate::vm::native_methods::pack_format::bytes_to_string(
            &buffer,
        ))
    }

    /// Where a socket named by a path sits, and where the other end of
    /// one does.
    pub(crate) fn socket_unix_address(
        &mut self,
        action: &str,
        handle: u64,
    ) -> Result<Object, MetorexError> {
        let found = if action == "unix_address" {
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
            match found.and_then(|held| held.as_pathname().map(|path| path.display().to_string())) {
                Some(held) => Object::string(held),
                None => Object::string(String::new()),
            },
        )
    }
}
