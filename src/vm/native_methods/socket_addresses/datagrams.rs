// Sockets that send each message on its own.

use super::*;

impl VirtualMachine {
    /// ── sockets that send each message on its own ─────────────────
    pub(crate) fn socket_datagram_open(
        &mut self,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let held = std::net::UdpSocket::bind((text.as_str(), port as u16)).map_err(|problem| {
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

    pub(crate) fn socket_datagram_connect(
        &mut self,
        handle: u64,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let socket = self
            .open_sockets
            .datagrams
            .get(&handle)
            .ok_or_else(|| refused(position, "connect on a closed socket".to_string()))?;
        socket
            .connect((text.as_str(), port as u16))
            .map_err(|problem| refused(position, format!("connect({text}:{port}): {problem}")))?;
        Ok(Object::Nil)
    }

    pub(crate) fn socket_datagram_send(
        &mut self,
        handle: u64,
        text: String,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let socket = self
            .open_sockets
            .datagrams
            .get(&handle)
            .ok_or_else(|| refused(position, "send on a closed socket".to_string()))?;
        let bytes = crate::vm::native_methods::pack_format::string_to_bytes(&text);
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
                refused(position, format!("send: {problem}"))
            }
        })?;
        Ok(Object::Int(sent as i64))
    }

    pub(crate) fn socket_datagram_receive(
        &mut self,
        handle: u64,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
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
                .ok_or_else(|| refused(position, "receive on a closed socket".to_string()))?;
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
        buffer.truncate(read);
        Ok(Object::array(vec![
            crate::vm::native_methods::pack_format::bytes_to_string(&buffer),
            Object::string(from.ip().to_string()),
            Object::Int(i64::from(from.port())),
        ]))
    }

    pub(crate) fn socket_datagram_address(&mut self, handle: u64) -> Result<Object, MetorexError> {
        Ok(
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
        )
    }

    /// Where a socket carrying each message on its own has been
    /// pointed, which is nothing until it has been connected.
    pub(crate) fn socket_datagram_peer(&mut self, handle: u64) -> Result<Object, MetorexError> {
        Ok(
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
        )
    }
}
