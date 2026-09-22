// The addresses a name stands for.

use super::*;

#[test]
fn a_qualified_constant_names_a_value_in_a_when_clause() {
    let result = run(r#"
module Held
  LOW = 1
  HIGH = 2
end

def named(value)
  case value
  when Held::LOW then :low
  when Held::HIGH then :high
  else :other
  end
end

[named(1), named(2), named(3)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:low, :high, :other]".to_string())
    );
}

#[test]
fn a_host_name_stands_for_every_address_it_answers_to() {
    let result = run(r#"
require 'socket'
found = Socket.resolved("localhost")
[found.class, found.empty?, found.include?("127.0.0.1")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[Array, false, true]".to_string())
    );
}

#[test]
fn a_name_that_stands_for_no_address_is_refused_when_read() {
    let error = run_err(
        r#"
require 'socket'
Socket.__address__("normalize", "not an address", 0)
"#,
    );
    assert!(error.contains("Name or service not known"), "{error}");
}

#[test]
fn the_bytes_of_a_name_that_stands_for_no_address_are_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.__address__("bytes", "not an address", 0)
"#,
    );
    assert!(error.contains("Name or service not known"), "{error}");
}

#[test]
fn a_struct_asked_for_of_a_name_that_names_no_address_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.sockaddr_in(80, "not an address")
"#,
    );
    assert!(
        error.contains("nodename nor servname provided, or not known"),
        "{error}"
    );
}

#[test]
fn a_struct_that_carries_no_address_is_refused_when_read_back() {
    let error = run_err(
        r#"
require 'socket'
Socket.unpack_sockaddr_in("held")
"#,
    );
    assert!(
        error.contains("not an AF_INET/AF_INET6 sockaddr"),
        "{error}"
    );
}

#[test]
fn an_address_read_out_of_a_struct_of_an_unknown_family_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.unpack_sockaddr_in(["10630000000000000000"].pack("H*"))
"#,
    );
    assert!(
        error.contains("not an AF_INET/AF_INET6 sockaddr"),
        "{error}"
    );
}

#[test]
fn an_address_asked_for_with_nothing_to_act_on_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.__address__
"#,
    );
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_socket_asked_for_with_nothing_to_act_on_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.__net__
"#,
    );
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_connection_says_where_its_own_end_sits() {
    let result = run(r#"
require 'socket'
server = TCPServer.new("127.0.0.1", 0)
held = TCPSocket.new("127.0.0.1", server.addr[1])
answered = [held.addr[0], held.local_address.ip_address]
held.close
server.close
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[AF_INET, 127.0.0.1]".to_string())
    );
}

#[test]
fn a_closed_socket_says_nothing_about_where_it_sat() {
    let result = run(r#"
require 'socket'
server = TCPServer.new("127.0.0.1", 0)
held = server.handle
server.close
[Socket.__net__("address", held, "", 0), Socket.__net__("peer", held, "", 0)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[nil, nil]".to_string())
    );
}

#[test]
fn a_name_that_stands_for_no_address_at_all_is_not_named_a_family() {
    let result = run(r#"
require 'socket'
[Socket.__address__("family", "127.0.0.1", 0), Socket.__address__("family", "::1", 0),
 Socket.__address__("family", "held", 0)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[4, 6, nil]".to_string())
    );
}

#[test]
fn taking_a_connection_from_a_listener_that_was_closed_is_refused() {
    let error = run_err(
        r#"
require 'socket'
server = TCPServer.new("127.0.0.1", 0)
held = server.handle
server.close
Socket.__net__("accept", held, "", 0)
"#,
    );
    assert!(error.contains("closed listener"), "{error}");
}

#[test]
fn reading_through_a_connection_that_was_closed_is_refused() {
    let error = run_err(
        r#"
require 'socket'
server = TCPServer.new("127.0.0.1", 0)
held = TCPSocket.new("127.0.0.1", server.addr[1])
held.close
begin
  held.read(4)
ensure
  server.close
end
"#,
    );
    assert!(error.contains("closed stream"), "{error}");
}

#[test]
fn listening_on_a_name_this_machine_does_not_answer_to_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.__net__("listen", 0, "203.0.113.1", 0)
"#,
    );
    assert!(error.contains("bind"), "{error}");
}
