// Reading and writing through a socket, and closing one.

use super::*;

impl VirtualMachine {
    pub(crate) fn socket_write(
        &mut self,
        handle: u64,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let stream = self
            .open_sockets
            .streams
            .get_mut(&handle)
            .ok_or_else(|| refused(position, "write to a closed connection".to_string()))?;
        let bytes = crate::vm::native_methods::pack_format::string_to_bytes(&text);
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
        let written = write_what_fits(stream, &bytes).map_err(|problem| {
            crate::vm::errors::simple_exception(
                errno_class(&problem),
                &format!("write: {problem}"),
                position,
            )
        })?;
        Ok(Object::Int(written as i64))
    }

    /// Whether the socket has something to read, or room to write,
    /// as the system reports it rather than by trying and undoing.
    pub(crate) fn socket_is_ready(
        &mut self,
        handle: u64,
        port: i64,
    ) -> Result<Object, MetorexError> {
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

    /// Read what has arrived without waiting for more. Nothing there
    /// yet is reported at once rather than handing the turn over,
    /// which is what a read told not to wait asks for.
    pub(crate) fn socket_read_now(
        &mut self,
        handle: u64,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let wanted = if port > 0 { port as usize } else { 65536 };
        let mut buffer = vec![0u8; wanted];
        let stream = self
            .open_sockets
            .streams
            .get_mut(&handle)
            .ok_or_else(|| refused(position, "read from a closed connection".to_string()))?;
        match stream.read(&mut buffer) {
            Ok(read) => {
                buffer.truncate(read);
                Ok(crate::vm::native_methods::pack_format::bytes_to_string(
                    &buffer,
                ))
            }
            Err(problem)
                if matches!(
                    problem.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                Err(timed_out(position))
            }
            Err(problem) => Err(crate::vm::errors::simple_exception(
                errno_class(&problem),
                &format!("read: {problem}"),
                position,
            )),
        }
    }

    /// Read what the other end sent, up to the count asked for.
    pub(crate) fn socket_read(
        &mut self,
        handle: u64,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.refuse_unless_connected(handle, position)?;
        let wanted = if port > 0 { port as usize } else { 65536 };
        let mut buffer = vec![0u8; wanted];
        let read = self.read_waiting(position, |vm| {
            let stream =
                vm.open_sockets.streams.get_mut(&handle).ok_or_else(|| {
                    refused(position, "read from a closed connection".to_string())
                })?;
            Ok(stream.read(&mut buffer))
        })?;
        buffer.truncate(read);
        Ok(crate::vm::native_methods::pack_format::bytes_to_string(
            &buffer,
        ))
    }

    /// Receive with flags such as MSG_PEEK, which a plain read has no way
    /// to ask for, waiting the way a read does until something arrives.
    pub(crate) fn socket_receive_with_flags(
        &mut self,
        handle: u64,
        flags: String,
        length: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.refuse_unless_connected(handle, position)?;
        let flags: libc::c_int = flags.parse().unwrap_or(0);
        let wanted = if length > 0 { length as usize } else { 65536 };
        let mut buffer = vec![0u8; wanted];
        // Told not to wait, the receive is tried once.
        let waits = flags & libc::MSG_DONTWAIT == 0;
        let read = self.read_waiting_unless(waits, position, |vm| {
            let descriptor = vm
                .socket_descriptor(handle)
                .ok_or_else(|| refused(position, "read from a closed connection".to_string()))?;
            // SAFETY: the descriptor is one this program holds open, and the
            // buffer holds `wanted` bytes.
            let received = unsafe {
                libc::recv(
                    descriptor,
                    buffer.as_mut_ptr() as *mut libc::c_void,
                    wanted,
                    flags,
                )
            };
            Ok(if received < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(received as usize)
            })
        })?;
        buffer.truncate(read);
        Ok(crate::vm::native_methods::pack_format::bytes_to_string(
            &buffer,
        ))
    }

    /// A socket that never connected has nothing to read from, which is
    /// what the operating system reports for one.
    fn refuse_unless_connected(&self, handle: u64, position: Position) -> Result<(), MetorexError> {
        if self.open_sockets.listeners.contains_key(&handle) {
            return Err(crate::vm::errors::simple_exception(
                "Errno::ENOTCONN",
                &format!(
                    "{} - recvfrom(2)",
                    crate::vm::init::errno_description(libc::ENOTCONN)
                ),
                position,
            ));
        }
        Ok(())
    }

    /// Read through `attempt` once when `waits` is false, reporting that
    /// nothing has arrived, and otherwise until something does.
    pub(crate) fn read_waiting_unless(
        &mut self,
        waits: bool,
        position: Position,
        mut attempt: impl FnMut(&mut Self) -> Result<std::io::Result<usize>, MetorexError>,
    ) -> Result<usize, MetorexError> {
        if waits {
            return self.read_waiting(position, attempt);
        }
        match attempt(self)? {
            Ok(held) => Ok(held),
            Err(problem) if problem.kind() == std::io::ErrorKind::WouldBlock => {
                Err(timed_out(position))
            }
            Err(problem) => Err(crate::vm::errors::simple_exception(
                errno_class(&problem),
                &format!("read: {problem}"),
                position,
            )),
        }
    }

    /// Read through `attempt` until something arrives. Nothing having
    /// arrived yet is not the end of the matter, so whatever else the
    /// program has to run gets a turn and the read is tried again.
    pub(crate) fn read_waiting(
        &mut self,
        position: Position,
        mut attempt: impl FnMut(&mut Self) -> Result<std::io::Result<usize>, MetorexError>,
    ) -> Result<usize, MetorexError> {
        let mut deadline = std::time::Instant::now() + WAIT_LIMIT;
        let waited_enough = std::time::Instant::now() + WAIT_CEILING;
        loop {
            match attempt(self)? {
                Ok(held) => return Ok(held),
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
                        &format!("read: {problem}"),
                        position,
                    ));
                }
            }
        }
    }

    pub(crate) fn socket_close(&mut self, handle: u64) -> Result<Object, MetorexError> {
        self.open_sockets.listeners.remove(&handle);
        self.open_sockets.streams.remove(&handle);
        self.open_sockets.unix_listeners.remove(&handle);
        self.open_sockets.unix_streams.remove(&handle);
        self.open_sockets.datagrams.remove(&handle);
        Ok(Object::Nil)
    }

    /// The number the operating system holds a socket under, which is
    /// what `fileno` reports.
    /// Close one direction of a connection, so the other end reads
    /// the end of the stream without the socket itself closing.
    pub(crate) fn socket_shutdown(
        &mut self,
        handle: u64,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
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
            return Err(refused(
                position,
                "shutdown of a closed connection".to_string(),
            ));
        };
        // SAFETY: `number` is a descriptor this program holds open.
        let answered = unsafe { libc::shutdown(number, port as libc::c_int) };
        if answered < 0 {
            let problem = std::io::Error::last_os_error();
            return Err(refused(position, format!("shutdown: {problem}")));
        }
        Ok(Object::Int(0))
    }
}

/// Write as much of `bytes` as the socket takes without waiting, answering
/// how much that was. The socket does not block, so a full buffer ends the
/// write early, and the caller waits for room before writing the rest.
pub(crate) fn write_what_fits(
    stream: &mut impl std::io::Write,
    bytes: &[u8],
) -> std::io::Result<usize> {
    let mut written = 0;
    while written < bytes.len() {
        match stream.write(&bytes[written..]) {
            Ok(count) => written += count,
            Err(problem) if problem.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(problem) if problem.kind() == std::io::ErrorKind::Interrupted => {}
            Err(problem) => return Err(problem),
        }
    }
    Ok(written)
}
