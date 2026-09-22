// The addresses a socket carries and the connections made over them.

use super::*;

#[test]
fn an_address_says_which_family_it_belongs_to() {
    let result = run(r#"
require 'socket'
[Addrinfo.tcp("127.0.0.1", 80).afamily == Socket::AF_INET,
 Addrinfo.tcp("::1", 80).afamily == Socket::AF_INET6,
 Addrinfo.unix("/tmp/held").afamily == Socket::AF_UNIX]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true, true]".to_string())
    );
}

#[test]
fn an_address_is_shown_the_way_ruby_shows_one() {
    let result = run(r#"
require 'socket'
[Addrinfo.tcp("127.0.0.1", 80).inspect, Addrinfo.tcp("::1", 80).inspect,
 Addrinfo.udp("127.0.0.1", 80).inspect, Addrinfo.ip("127.0.0.1").inspect]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(
            "[#<Addrinfo: 127.0.0.1:80 TCP>, #<Addrinfo: [::1]:80 TCP>, \
             #<Addrinfo: 127.0.0.1:80 UDP>, #<Addrinfo: 127.0.0.1>]"
                .to_string()
        )
    );
}

#[test]
fn an_address_says_what_kind_of_address_it_is() {
    let result = run(r#"
require 'socket'
[Addrinfo.ip("127.0.0.1").ipv4_loopback?, Addrinfo.ip("10.0.0.1").ipv4_private?,
 Addrinfo.ip("224.0.0.1").ipv4_multicast?, Addrinfo.ip("::1").ipv6_loopback?,
 Addrinfo.ip("ff02::1").ipv6_mc_linklocal?, Addrinfo.ip("fe80::1").ipv6_linklocal?,
 Addrinfo.ip("::ffff:127.0.0.1").ipv6_v4mapped?, Addrinfo.ip("8.8.8.8").ipv4_private?]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true, true, true, true, true, true, false]".to_string())
    );
}

#[test]
fn an_address_reads_back_out_of_the_struct_it_is_carried_in() {
    let result = run(r#"
require 'socket'
held = Socket.sockaddr_in(80, "127.0.0.1")
[Socket.unpack_sockaddr_in(held), Addrinfo.new(held).ip_address,
 Socket.unpack_sockaddr_in(Socket.sockaddr_in(443, "::1"))]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[[80, 127.0.0.1], 127.0.0.1, [443, ::1]]".to_string())
    );
}

#[test]
fn an_address_that_names_nothing_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Addrinfo.tcp("no.such.host.metorex.invalid", 80)
"#,
    );
    assert!(error.contains("getaddrinfo"), "{error}");
}

#[test]
fn a_path_too_long_for_the_struct_that_holds_it_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.sockaddr_un("/" + "a" * 200)
"#,
    );
    assert!(error.contains("too long unix socket path"), "{error}");
}

#[test]
fn a_connection_made_to_a_socket_carries_what_is_written_through_it() {
    let result = run(r#"
require 'socket'
server = TCPServer.new("127.0.0.1", 0)
port = server.addr[1]
client = TCPSocket.new("127.0.0.1", port)
client.write("held")
accepted = server.accept
answered = [accepted.read(4), accepted.peeraddr[2], server.addr[0], port > 0]
accepted.close
client.close
server.close
answered << server.closed?
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[held, 127.0.0.1, AF_INET, true, true]".to_string())
    );
}

#[test]
fn reaching_a_port_nothing_is_listening_on_is_refused() {
    let error = run_err(
        r#"
require 'socket'
TCPSocket.new("127.0.0.1", 1)
"#,
    );
    assert!(error.contains("connect"), "{error}");
}

#[test]
fn writing_through_a_connection_that_was_closed_is_refused() {
    let error = run_err(
        r#"
require 'socket'
server = TCPServer.new("127.0.0.1", 0)
held = TCPSocket.new("127.0.0.1", server.addr[1])
held.close
begin
  held.write("held")
ensure
  server.close
end
"#,
    );
    assert!(error.contains("closed connection"), "{error}");
}

#[test]
fn this_machine_says_what_name_it_answers_to() {
    let result = run(r#"
require 'socket'
held = Socket.gethostname
[held.class, held.empty?]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[String, false]".to_string())
    );
}

#[test]
fn an_address_action_nothing_answers_to_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.__address__("no_such_action", "127.0.0.1", 0)
"#,
    );
    assert!(error.contains("unknown address action"), "{error}");
}

#[test]
fn a_socket_action_nothing_answers_to_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.__net__("no_such_action", 0, "", 0)
"#,
    );
    assert!(error.contains("unknown socket action"), "{error}");
}
