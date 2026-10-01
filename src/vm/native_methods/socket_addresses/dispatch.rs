// Which socket action a name stands for. Every arm hands the call on to a
// method of its own, grouped into the modules beside this one.

use super::*;

impl VirtualMachine {
    /// `Socket.__net__(action, handle, text, port)` — opening, accepting,
    /// reading, and writing, which belong to the operating system.
    pub(crate) fn socket_net(
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
        let port = match arguments.get(3) {
            Some(Object::Int(held)) => *held,
            _ => 0,
        };
        match &*action.as_str() {
            "listen" => self.socket_listen(text, port, position),
            "bind_only" => self.socket_bind_only(text, port, position),
            "tcp_open" => self.socket_tcp_open(text, position),
            "udp_fresh" => self.socket_udp_fresh(text, position),
            "numeric_addrinfo" => self.socket_numeric_addrinfo(text, port, position),
            "bind_on" => self.socket_bind_on(handle, text, port, position),
            "connect_on" => self.socket_connect_on(handle, text, port, position),
            "connect_now" => self.socket_connect_now(handle, text, port, position),
            "listen_on" => self.socket_listen_on(handle, port, position),
            "connect" => self.socket_connect(text, port, position),
            "address" => self.socket_own_address(handle),
            "peer" => self.socket_peer_address(handle),
            "accept" => self.socket_accept(handle, position),
            "write" => self.socket_write(handle, text, port, position),
            "ready?" => self.socket_is_ready(handle, port),
            "read_now" => self.socket_read_now(handle, port, position),
            "read" => self.socket_read(handle, port, position),
            "recv_flags" => self.socket_receive_with_flags(handle, text, port, position),
            "close" => self.socket_close(handle),
            "shutdown" => self.socket_shutdown(handle, port, position),
            "fileno" => self.socket_fileno(handle),
            "handle_of_fd" => self.socket_handle_of_descriptor(port),
            "constant" => self.socket_constant(text),
            "ioctl" => self.socket_ioctl(handle, text, port, position),
            "nonblock" => self.socket_nonblock(handle),
            "set_nonblock" => self.socket_set_nonblock(handle, port),
            "service_port" => self.socket_service_port(text),
            "service_name" => self.socket_service_name(text, port),
            "socket_option" => self.socket_option(handle, text, port, position),
            "set_socket_option" => self.socket_set_option(handle, text, position),
            "interfaces" => self.socket_interfaces(),
            "send_fd" => self.socket_send_descriptor(handle, port, position),
            "receive_fd" => self.socket_receive_descriptor(handle, position),
            "unix_dgram_open" => self.socket_unix_datagram_open(text, position),
            "unix_dgram_send" => self.socket_unix_datagram_send(handle, text, position),
            "unix_dgram_receive" => self.socket_unix_datagram_receive(handle, port, position),
            "kind_of_handle" => self.socket_kind_of_handle(handle),
            "unix_listen" => self.socket_unix_listen(text, position),
            "unix_connect" => self.socket_unix_connect(text, position),
            "unix_pair" => self.socket_unix_pair(position),
            "unix_accept_now" => self.socket_unix_accept_now(handle, position),
            "accept_now" => self.socket_accept_now(handle, position),
            "unix_accept" => self.socket_unix_accept(handle, position),
            "unix_write" => self.socket_unix_write(handle, text, position),
            "unix_read" => self.socket_unix_read(handle, text, port, position),
            "unix_address" | "unix_peer" => self.socket_unix_address(&action.as_str(), handle),
            "udp_open" => self.socket_datagram_open(text, port, position),
            "udp_connect" => self.socket_datagram_connect(handle, text, port, position),
            "udp_send" => self.socket_datagram_send(handle, text, position),
            "udp_send_to" => self.socket_datagram_send_to(handle, text, port, position),
            "udp_receive" => self.socket_datagram_receive(handle, text, port, position),
            "udp_address" => self.socket_datagram_address(handle),
            "udp_peer" => self.socket_datagram_peer(handle),
            other => Err(MetorexError::runtime_error(
                format!("unknown socket action {other}"),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }
}

/// The refusal a socket answers for a name nothing is listening on.
pub(crate) fn refused(position: Position, what: String) -> MetorexError {
    crate::vm::errors::simple_exception("Errno::ECONNREFUSED", &what, position)
}
