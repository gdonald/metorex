// Writing to a socket: a send told not to wait writes what fits, and a
// write larger than the buffer waits for the reader to take it.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

/// What a socket program answers, run on a thread with room for the socket
/// library to load.
fn socket_inspected(code: &str) -> String {
    let source = format!("require 'socket'\n({code}).inspect");
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
fn a_send_that_must_not_wait_writes_part_of_a_large_message() {
    assert_eq!(
        socket_inspected(
            "reader, writer = UNIXSocket.pair\nwrote = writer.sendmsg_nonblock('x' * 10_000_000)\n[wrote > 0, wrote < 10_000_000]"
        ),
        "[true, true]"
    );
}

#[test]
fn a_send_that_must_not_wait_on_a_full_buffer_answers_wait_writable() {
    assert_eq!(
        socket_inspected(
            "reader, writer = UNIXSocket.pair\nwriter.sendmsg_nonblock('x' * 10_000_000)\nwriter.sendmsg_nonblock('x' * 10_000_000, exception: false)"
        ),
        ":wait_writable"
    );
}

#[test]
fn a_tcp_write_larger_than_the_buffer_waits_until_the_reader_takes_it() {
    assert_eq!(
        socket_inspected(
            "server = TCPServer.new('127.0.0.1', 0)\nclient = TCPSocket.new('127.0.0.1', server.addr[1])\npeer = server.accept\ncounted = Thread.new do\n  total = 0\n  while (chunk = peer.read(65_536)) && !chunk.empty?\n    total += chunk.bytesize\n  end\n  total\nend\nwritten = client.write('y' * 3_000_000)\nclient.close\n[written, counted.value]"
        ),
        "[3000000, 3000000]"
    );
}

#[test]
fn a_unix_socket_write_larger_than_the_buffer_waits_until_the_reader_takes_it() {
    assert_eq!(
        socket_inspected(
            "reader, writer = UNIXSocket.pair\ncounted = Thread.new do\n  total = 0\n  while total < 3_000_000 && (chunk = reader.recv(65_536)) && !chunk.empty?\n    total += chunk.bytesize\n  end\n  total\nend\n[writer.write('y' * 3_000_000), counted.value]"
        ),
        "[3000000, 3000000]"
    );
}

#[test]
fn a_datagram_sent_to_an_address_leaves_the_socket_unjoined() {
    assert_eq!(
        socket_inspected(
            "server = Socket.new(:INET, :DGRAM)\nserver.bind(Socket.sockaddr_in(0, '127.0.0.1'))\nclient = Socket.new(:INET, :DGRAM)\nsent = client.send('hello', 0, server.getsockname)\nagain = (client.send('again', 0) rescue [$!.class, $!.message])\n[sent, again, server.recv(10)]"
        ),
        "[5, [Errno::EDESTADDRREQ, \"Destination address required - send(2)\"], \"hello\"]"
    );
}

#[test]
fn a_message_is_taken_through_to_str_and_refused_without_it() {
    assert_eq!(
        socket_inspected(
            "server = Socket.new(:INET, :DGRAM)\nserver.bind(Socket.sockaddr_in(0, '127.0.0.1'))\nclient = Socket.new(:INET, :DGRAM)\nmessage = Object.new\ndef message.to_str = 'hi'\n[client.send(message, 0, server.connect_address), (client.send(5, 0, server.getsockname) rescue $!.message)]"
        ),
        "[2, \"no implicit conversion of Integer into String\"]"
    );
}

#[test]
fn a_udp_socket_sends_to_a_packed_address_without_joining_it() {
    assert_eq!(
        socket_inspected(
            "server = UDPSocket.new\nserver.bind('127.0.0.1', 0)\nclient = UDPSocket.new\npacked = Socket.sockaddr_in(server.addr[1], '127.0.0.1')\n[client.send('a', 0, packed), client.send('b', 0, server.connect_address), client.send('c', 0, '127.0.0.1', server.addr[1]), (client.send('d', 0) rescue $!.class), server.recv(1), server.recv(1), server.recv(1)]"
        ),
        "[1, 1, 1, Errno::EDESTADDRREQ, \"a\", \"b\", \"c\"]"
    );
}

#[test]
fn a_receive_told_to_peek_leaves_the_data_for_the_next_read() {
    assert_eq!(
        socket_inspected(
            "server = TCPServer.new('127.0.0.1', 0)\nclient = TCPSocket.new('127.0.0.1', server.addr[1])\npeer = server.accept\nclient.write 'peeked'\n[peer.recv(6, Socket::MSG_PEEK), peer.recv(6)]"
        ),
        "[\"peeked\", \"peeked\"]"
    );
}

#[test]
fn a_unix_pair_is_joined_without_a_name() {
    assert_eq!(
        socket_inspected(
            "first, second = UNIXSocket.pair\nfirst.write 'ping'\nthird, fourth = Socket.pair(:UNIX, :STREAM)\nfourth.write 'pong'\n[second.recv(4), third.recv(4), first.path, third.local_address.unix_path]"
        ),
        "[\"ping\", \"pong\", \"\", \"\"]"
    );
}
