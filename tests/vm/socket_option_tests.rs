// A socket made by Socket.new holds a descriptor from the start, so its
// settings are read from and written to the operating system before it is
// bound or connected.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

/// What the C library calls an errno, which is how the operating system
/// words the error a socket reports.
fn described(number: i32) -> String {
    let rendered = std::io::Error::from_raw_os_error(number).to_string();
    rendered
        .split(" (os error")
        .next()
        .unwrap_or(&rendered)
        .to_string()
}

/// What a socket program answers, run on a thread with room for the socket
/// library to load.
fn socket_inspected(code: &str) -> String {
    let source = format!(
        "require 'socket'\nsocket = Socket.new(:INET, :STREAM)\nanswer = begin\n{code}\nrescue StandardError => error\n  [error.class, error.message]\nend\nanswer.inspect"
    );
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let tokens = Lexer::new(&source).tokenize();
            let statements = Parser::new(tokens).parse().expect("parse failed");
            let mut vm = VirtualMachine::new();
            match vm.execute_program(&statements).expect("execution failed") {
                Some(Object::String(text)) => text.as_str().to_string(),
                other => panic!("expected an inspection, got {other:?}"),
            }
        })
        .expect("thread failed")
        .join()
        .expect("thread panicked")
}

#[test]
fn an_unbound_socket_reports_its_type() {
    assert_eq!(
        socket_inspected("socket.getsockopt(:SOCKET, :TYPE).int == Socket::SOCK_STREAM"),
        "true"
    );
}

#[test]
fn a_linger_option_reads_back_as_the_whole_struct() {
    assert_eq!(
        socket_inspected("socket.getsockopt(:SOCKET, :LINGER).to_s.bytesize"),
        "8"
    );
}

#[test]
fn an_unknown_option_is_refused_by_the_operating_system() {
    assert_eq!(
        socket_inspected("socket.getsockopt(Socket::SOL_SOCKET, -1)"),
        "[Errno::ENOPROTOOPT, \"Protocol not available - getsockopt(2)\"]"
    );
}

#[test]
fn setting_an_unknown_option_is_refused_by_the_operating_system() {
    assert_eq!(
        socket_inspected("socket.setsockopt(Socket::SOL_SOCKET, -1, 1)"),
        "[Errno::ENOPROTOOPT, \"Protocol not available - setsockopt(2)\"]"
    );
}

#[test]
fn a_boolean_value_is_written_as_an_int() {
    assert_eq!(
        socket_inspected(
            "socket.setsockopt(:SOCKET, :OOBINLINE, true)\non = socket.getsockopt(:SOCKET, :OOBINLINE).int\nsocket.setsockopt(:SOCKET, :OOBINLINE, false)\n[on != 0, socket.getsockopt(:SOCKET, :OOBINLINE).int]"
        ),
        "[true, 0]"
    );
}

#[test]
fn a_linger_given_as_one_int_is_refused() {
    assert_eq!(
        socket_inspected("socket.setsockopt(Socket::SOL_SOCKET, Socket::SO_LINGER, 0)"),
        "[Errno::EINVAL, \"Invalid argument - setsockopt(2)\"]"
    );
}

#[test]
fn a_nil_value_is_refused_as_a_string() {
    assert_eq!(
        socket_inspected("socket.setsockopt(Socket::SOL_SOCKET, Socket::SO_SNDBUF, nil)"),
        "[TypeError, \"no implicit conversion of nil into String\"]"
    );
}

#[test]
fn a_float_value_is_refused_as_a_string() {
    assert_eq!(
        socket_inspected("socket.setsockopt(:SOCKET, :SNDBUF, 4000.5)"),
        "[TypeError, \"no implicit conversion of Float into String\"]"
    );
}

#[test]
fn a_nil_level_is_refused_as_a_number() {
    assert_eq!(
        socket_inspected("socket.setsockopt(nil, :REUSEADDR, true)"),
        "[TypeError, \"no implicit conversion from nil to integer\"]"
    );
}

#[test]
fn an_option_object_with_another_argument_is_refused() {
    assert_eq!(
        socket_inspected(
            "option = Socket::Option.bool(:INET, :SOCKET, :REUSEADDR, true)\n[(socket.setsockopt(option, :REUSEADDR) rescue $!.message), (socket.setsockopt(option, :REUSEADDR, true) rescue $!.message)]"
        ),
        "[\"wrong number of arguments (given 2, expected 3)\", \"no implicit conversion of Socket::Option into Integer\"]"
    );
}

#[test]
fn level_and_option_names_are_taken_through_to_str() {
    assert_eq!(
        socket_inspected(
            "level = Object.new\ndef level.to_str = 'SOCKET'\nname = Object.new\ndef name.to_str = 'REUSEADDR'\n[socket.setsockopt(level, name, true), socket.getsockopt(:SOCKET, :REUSEADDR).bool]"
        ),
        "[0, true]"
    );
}

#[test]
fn a_socket_made_by_new_binds_listens_and_connects_on_its_own_descriptor() {
    assert_eq!(
        socket_inspected(
            "socket.bind(Socket.sockaddr_in(0, '127.0.0.1'))\nport = Socket.unpack_sockaddr_in(socket.getsockname)[0]\nsocket.listen(5)\nclient = Socket.new(:INET, :STREAM)\nconnected = client.connect(Socket.sockaddr_in(port, '127.0.0.1'))\npeer, = socket.accept\nclient.write 'via fresh'\nagain = (client.connect(Socket.sockaddr_in(port, '127.0.0.1')) rescue $!.message.sub(/:\\d+\\z/, ''))\n[connected, peer.recv(9), again]"
        ),
        format!(
            "[0, \"via fresh\", \"{} - connect(2) for 127.0.0.1\"]",
            described(libc::EISCONN)
        )
    );
}

#[test]
fn connecting_where_nothing_listens_is_refused() {
    assert_eq!(
        socket_inspected("socket.connect(Socket.sockaddr_in(1, '127.0.0.1'))"),
        "[Errno::ECONNREFUSED, \"Connection refused - connect(2) for 127.0.0.1:1\"]"
    );
}

#[test]
fn an_unbound_socket_names_no_address() {
    assert_eq!(
        socket_inspected("Socket.unpack_sockaddr_in(socket.getsockname)"),
        "[0, \"0.0.0.0\"]"
    );
}

#[test]
fn a_connect_that_must_not_wait_reports_it_is_under_way_and_then_connected() {
    assert_eq!(
        socket_inspected(
            "server = TCPServer.new('127.0.0.1', 0)\naddress = Socket.sockaddr_in(server.addr[1], '127.0.0.1')\nfirst = (socket.connect_nonblock(address) rescue [$!.class, $!.message])\nIO.select(nil, [socket])\nsecond = socket.connect_nonblock(address, exception: false)\n[first, second, (socket.connect_nonblock(address) rescue $!.class)]"
        ),
        "[[IO::EINPROGRESSWaitWritable, \"Operation now in progress - connect(2) would block\"], 0, Errno::EISCONN]"
    );
}

#[test]
fn a_connect_that_must_not_wait_answers_wait_writable_without_exceptions() {
    assert_eq!(
        socket_inspected(
            "server = TCPServer.new('127.0.0.1', 0)\nsocket.connect_nonblock(Socket.sockaddr_in(server.addr[1], '127.0.0.1'), exception: false)"
        ),
        ":wait_writable"
    );
}

#[test]
fn a_datagram_connect_that_must_not_wait_connects_at_once() {
    assert_eq!(
        socket_inspected(
            "server = Socket.new(:INET, :DGRAM)\nserver.bind(Socket.sockaddr_in(0, '127.0.0.1'))\nclient = Socket.new(:INET, :DGRAM)\n[client.connect_nonblock(server.connect_address), (client.connect_nonblock(666) rescue $!.message)]"
        ),
        "[0, \"no implicit conversion of Integer into String\"]"
    );
}

#[test]
fn a_socket_that_listens_unbound_is_reached_on_the_port_it_was_given() {
    assert_eq!(
        socket_inspected(
            "unbound = (socket.connect_address rescue $!.message)\nsocket.listen(1)\n[unbound, socket.connect_address.ip_address, socket.connect_address.ip_port > 0]"
        ),
        "[\"unbound IPv4 socket\", \"127.0.0.1\", true]"
    );
}

#[test]
fn an_unbound_datagram_socket_names_no_address() {
    assert_eq!(
        socket_inspected("Socket.new(:INET6, :DGRAM).connect_address"),
        "[SocketError, \"unbound IPv6 socket\"]"
    );
}

#[test]
fn a_datagram_message_names_the_socket_that_sent_it() {
    assert_eq!(
        socket_inspected(
            "server = Socket.new(:INET, :DGRAM)\nserver.bind(Socket.sockaddr_in(0, '127.0.0.1'))\nclient = Socket.new(:INET, :DGRAM)\nclient.connect(server.getsockname)\nclient.write 'hello'\nmessage, from, flags = server.recvmsg\n[message, from.inspect.start_with?('#<Addrinfo: 127.0.0.1:'), from.socktype == Socket::SOCK_DGRAM, flags]"
        ),
        "[\"hello\", true, true, 0]"
    );
}

#[test]
fn a_udp_socket_receives_a_message_with_its_sender() {
    assert_eq!(
        socket_inspected(
            "server = UDPSocket.new\nserver.bind('127.0.0.1', 0)\nclient = UDPSocket.new\nclient.send('u', 0, '127.0.0.1', server.addr[1])\nmessage, from, = server.recvmsg\n[message, from.ip_address]"
        ),
        "[\"u\", \"127.0.0.1\"]"
    );
}

#[test]
fn a_message_read_off_a_connection_names_an_empty_address() {
    assert_eq!(
        socket_inspected(
            "socket.bind(Socket.sockaddr_in(0, '127.0.0.1'))\nsocket.listen(1)\ncaller = Socket.new(:INET, :STREAM)\ncaller.connect(socket.getsockname)\ncaller.write 'hi'\naccepted, = socket.accept\nread, empty, = accepted.recvmsg\ncaller.close\n[read, empty.inspect, (empty.ip_address rescue $!.class), accepted.recvmsg]"
        ),
        "[\"hi\", \"#<Addrinfo: empty-sockaddr SOCK_STREAM>\", SocketError, nil]"
    );
}

#[test]
fn a_receive_that_must_not_wait_reports_nothing_there_yet() {
    assert_eq!(
        socket_inspected(
            "server = Socket.new(:INET, :DGRAM)\nserver.bind(Socket.sockaddr_in(0, '127.0.0.1'))\n[server.recvmsg_nonblock(exception: false), (server.recvmsg_nonblock rescue $!.message), (server.recv_nonblock(1) rescue $!.message)]"
        ),
        "[:wait_readable, \"Resource temporarily unavailable - recvmsg(2) would block\", \"Resource temporarily unavailable - recvfrom(2) would block\"]"
    );
}

#[test]
fn reading_a_socket_that_never_connected_is_refused() {
    assert_eq!(
        socket_inspected(
            "[(socket.recvmsg_nonblock rescue $!.message), (socket.recv(1) rescue $!.message)]"
        ),
        format!(
            "[\"{reason} - recvmsg(2)\", \"{reason} - recvfrom(2)\"]",
            reason = described(libc::ENOTCONN)
        )
    );
}

#[test]
fn accepting_on_a_socket_that_does_not_listen_is_refused() {
    assert_eq!(
        socket_inspected(
            "[(socket.accept_nonblock rescue $!.message), (socket.accept rescue $!.message)]"
        ),
        "[\"Invalid argument - accept(2)\", \"Invalid argument - accept(2)\"]"
    );
}

#[test]
fn a_connect_with_a_timeout_gives_up_with_timeout_error() {
    assert_eq!(
        socket_inspected(
            "socket.timeout = 0\nbegin\n  socket.connect(Socket.pack_sockaddr_in(1, '192.0.2.1'))\n  :connected\nrescue IO::TimeoutError => error\n  error.message\nrescue Errno::ECONNREFUSED, Errno::ENETUNREACH\n  'user specified timeout for 192.0.2.1:1'\nend"
        ),
        "\"user specified timeout for 192.0.2.1:1\""
    );
}

#[test]
fn a_tcp_connection_to_an_ipv6_address_opens_an_ipv6_socket() {
    assert_eq!(
        socket_inspected(
            "server = TCPServer.new('::1', 0)\nconnected = Socket.tcp('::1', server.addr[1])\nconnected.local_address.ipv6?"
        ),
        "true"
    );
}

#[test]
fn an_addrinfo_from_an_array_keeps_the_socket_type_and_protocol_it_was_given() {
    assert_eq!(
        socket_inspected(
            "given = Addrinfo.new(['AF_INET', 80, nil, '127.0.0.1'], nil, nil, Socket::IPPROTO_UDP)\nraw = Addrinfo.new(['AF_INET', 80, nil, '127.0.0.1'], nil, :RAW, Socket::IPPROTO_ICMP)\n[given.socktype, given.protocol, raw.socktype == Socket::SOCK_RAW, raw.protocol]"
        ),
        "[0, 17, true, 1]"
    );
}

#[test]
fn an_addrinfo_from_an_array_shows_the_name_it_was_written_with() {
    assert_eq!(
        socket_inspected(
            "[Addrinfo.new(['AF_INET', 80, 'localhost', '127.0.0.1'], nil, :STREAM, 6).inspect, Addrinfo.new(['INET', 80, '127.0.0.1', '127.0.0.1']).inspect]"
        ),
        "[\"#<Addrinfo: 127.0.0.1:80 TCP (localhost)>\", \"#<Addrinfo: 127.0.0.1:80>\"]"
    );
}

#[test]
fn an_addrinfo_whose_address_is_not_of_its_family_is_put_to_the_resolver() {
    assert_eq!(
        socket_inspected(
            "begin\n  Addrinfo.new(['AF_INET6', 80, 'hostname', '127.0.0.1'])\nrescue Socket::ResolutionError => error\n  [error.class, error.error_code.is_a?(Integer)]\nend"
        ),
        "[Socket::ResolutionError, true]"
    );
}

#[test]
fn an_addrinfo_pairing_datagrams_with_tcp_is_refused_by_the_resolver() {
    assert_eq!(
        socket_inspected(
            "begin\n  Addrinfo.new(['AF_INET', 80, nil, '127.0.0.1'], nil, :DGRAM, Socket::IPPROTO_TCP)\nrescue SocketError => error\n  error.class\nend"
        ),
        "Socket::ResolutionError"
    );
}

#[test]
fn an_addrinfo_of_an_unknown_family_is_refused() {
    assert_eq!(
        socket_inspected("Addrinfo.new(['AF_NOPE', 1, nil, '127.0.0.1'])"),
        "[SocketError, \"unknown address family: AF_NOPE\"]"
    );
}

#[test]
fn an_addrinfo_port_must_be_a_number() {
    assert_eq!(
        socket_inspected("Addrinfo.new(['AF_INET', '80', nil, '127.0.0.1'])"),
        "[TypeError, \"no implicit conversion of String into Integer\"]"
    );
}

#[test]
fn an_addrinfo_from_a_unix_array_is_a_stream_socket_path() {
    assert_eq!(
        socket_inspected("Addrinfo.new(['AF_UNIX', '/tmp/x'], nil, :DGRAM).inspect"),
        "\"#<Addrinfo: /tmp/x SOCK_STREAM>\""
    );
}

#[test]
fn an_addrinfo_from_a_packed_unix_address_names_no_socket_type() {
    assert_eq!(
        socket_inspected(
            "held = Addrinfo.new(Socket.pack_sockaddr_un('socket'))\n[held.inspect, held.unix_path, held.pfamily, held.socktype, Addrinfo.new(Socket.pack_sockaddr_un('/tmp/s'), :UNIX, :DGRAM).inspect]"
        ),
        "[\"#<Addrinfo: UNIX socket>\", \"socket\", 0, 0, \"#<Addrinfo: /tmp/s SOCK_DGRAM>\"]"
    );
}

#[test]
fn the_protocols_the_platform_names_are_defined() {
    assert_eq!(
        socket_inspected(
            "[Socket::IPPROTO_HOPOPTS, Socket::IPPROTO_ICMPV6 == 58, Socket::IPPROTO_IGMP]"
        ),
        "[0, true, 2]"
    );
}

#[test]
fn an_addrinfo_reads_special_host_names_as_the_addresses_they_stand_for() {
    assert_eq!(
        socket_inspected(
            "[Addrinfo.new(['AF_INET', 0, '', '<any>']).inspect, Addrinfo.new(['AF_INET', 5, '', '<broadcast>']).inspect, Addrinfo.new(['AF_INET', 0, '', '']).to_sockaddr == Socket.sockaddr_in(0, '')]"
        ),
        "[\"#<Addrinfo: 0.0.0.0 ()>\", \"#<Addrinfo: 255.255.255.255:5 ()>\", true]"
    );
}

#[test]
fn an_addrinfo_on_port_zero_shows_no_port() {
    assert_eq!(
        socket_inspected("Addrinfo.tcp('127.0.0.1', 0).inspect"),
        "\"#<Addrinfo: 127.0.0.1 TCP>\""
    );
}
