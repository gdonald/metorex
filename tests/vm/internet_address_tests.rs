// Coverage tests for IPAddr

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

/// Loading and running the IPAddr library nests deeper than the stack a test
/// thread is given, so each program runs on a thread sized like the one the
/// binary itself uses.
fn on_a_deep_stack(work: impl FnOnce() -> String + Send + 'static) -> String {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(work)
        .expect("thread failed")
        .join()
        .expect("thread panicked")
}

/// The value a program answers, written the way `inspect` writes it.
fn shown(code: &str) -> String {
    let held = format!("__answered__ = begin\n{code}\nend\n__answered__.inspect");
    on_a_deep_stack(move || {
        let tokens = Lexer::new(&held).tokenize();
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        let mut vm = VirtualMachine::new();
        match vm.execute_program(&stmts).expect("execution failed") {
            Some(Object::String(written)) => written.to_string(),
            other => panic!("expected a string, got {other:?}"),
        }
    })
}

/// The text a program answers, with the quotes `inspect` puts around a
/// string taken back off.
fn quoted(code: &str) -> String {
    let written = shown(code);
    written
        .strip_prefix('"')
        .and_then(|held| held.strip_suffix('"'))
        .expect("expected a string")
        .to_string()
}

fn run_err(code: &str) -> String {
    let held = code.to_string();
    on_a_deep_stack(move || {
        let tokens = Lexer::new(&held).tokenize();
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        let mut vm = VirtualMachine::new();
        vm.execute_program(&stmts).unwrap_err().to_string()
    })
}

const HERE: &str = "require 'ipaddr'\nhere = IPAddr.new('192.168.1.2/24')\n";

// ── Reading an address ─────────────────────────────────────────────────────

#[test]
fn an_address_reports_the_parts_it_holds() {
    assert_eq!(quoted(&format!("{HERE}here.to_s")), "192.168.1.0");
    assert_eq!(quoted(&format!("{HERE}here.to_string")), "192.168.1.0");
    assert_eq!(
        quoted(&format!("{HERE}here.inspect")),
        "#<IPAddr: IPv4:192.168.1.0/255.255.255.0>"
    );
    assert_eq!(shown(&format!("{HERE}here.ipv4?")), "true");
    assert_eq!(shown(&format!("{HERE}here.ipv6?")), "false");
    assert_eq!(shown(&format!("{HERE}here.prefix")), "24");
    assert_eq!(shown(&format!("{HERE}here.to_i")), "3232235776");
    assert_eq!(
        shown(&format!("{HERE}here.family == Socket::AF_INET")),
        "true"
    );
}

#[test]
fn an_ipv6_address_is_written_with_its_longest_run_of_zeros_left_out() {
    assert_eq!(quoted("require 'ipaddr'\nIPAddr.new.to_s"), "::");
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('0:0:0:1::').to_s"),
        "0:0:0:1::"
    );
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('3ffe:0505:0002:0000:0000:0000:0000:0001').to_s"),
        "3ffe:505:2::1"
    );
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('0123:4567:89ab:cdef:0ABC:DEF0:1234:5678').to_s"),
        "123:4567:89ab:cdef:abc:def0:1234:5678"
    );
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('3ffe:505:2::1').to_string"),
        "3ffe:0505:0002:0000:0000:0000:0000:0001"
    );
}

#[test]
fn a_mask_may_be_written_as_a_length_or_as_an_address() {
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('3ffe:505:2::/48').to_s"),
        "3ffe:505:2::"
    );
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('3ffe:505:2::/ffff:ffff:ffff::').to_s"),
        "3ffe:505:2::"
    );
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('192.168.1.2/255.255.255.0').to_s"),
        "192.168.1.0"
    );
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('3ffe:505:2::/48').mask(32).to_s"),
        "3ffe:505::"
    );
}

#[test]
fn an_ipv4_address_written_inside_an_ipv6_one_keeps_its_dotted_form() {
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('::192.168.1.2').to_s"),
        "::192.168.1.2"
    );
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('::ffff:192.168.1.2').to_s"),
        "::ffff:192.168.1.2"
    );
    assert_eq!(
        shown("require 'ipaddr'\nIPAddr.new('::192.168.1.2').ipv4_compat?"),
        "true"
    );
    assert_eq!(
        shown("require 'ipaddr'\nIPAddr.new('::ffff:192.168.1.2').ipv4_mapped?"),
        "true"
    );
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('::ffff:192.168.1.2').native.to_s"),
        "192.168.1.2"
    );
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('192.168.1.2').ipv4_compat.to_s"),
        "::192.168.1.2"
    );
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('::1:192.168.1.2/120').to_string"),
        "0000:0000:0000:0000:0000:0001:c0a8:0100"
    );
}

#[test]
fn text_that_names_no_address_is_refused() {
    assert!(run_err("require 'ipaddr'\nIPAddr.new('::1/255.255.255.0')").contains("not same"));
    assert!(
        run_err("require 'ipaddr'\nIPAddr.new('[192.168.1.2]/120')").contains("invalid address")
    );
    assert!(
        run_err("require 'ipaddr'\nIPAddr.new('::ffff:192.168.1.2/120', Socket::AF_INET)")
            .contains("address family mismatch")
    );
    assert!(
        run_err("require 'ipaddr'\nIPAddr.new(1)").contains("address family must be specified")
    );
}

// ── Working with an address ────────────────────────────────────────────────

#[test]
fn the_bit_operations_answer_a_new_address() {
    const A: &str = "require 'ipaddr'\na = IPAddr.new('3ffe:505:2::')\n";
    assert_eq!(
        quoted(&format!("{A}(a | IPAddr.new('0:0:0:1::')).to_s")),
        "3ffe:505:2:1::"
    );
    assert_eq!(
        quoted(&format!("{A}(a & IPAddr.new('ffff:ffff::')).to_s")),
        "3ffe:505::"
    );
    assert_eq!(quoted(&format!("{A}(a >> 16).to_s")), "0:3ffe:505:2::");
    assert_eq!(quoted(&format!("{A}(a << 16).to_s")), "505:2::");
    assert_eq!(
        quoted("require 'ipaddr'\n(~IPAddr.new).to_s"),
        "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"
    );
    // The receiver is left as it was, since each operation makes a new one.
    assert_eq!(
        quoted(&format!("{A}a | IPAddr.new('0:0:0:1::')\na.to_s")),
        "3ffe:505:2::"
    );
}

#[test]
fn a_network_holds_the_addresses_that_fall_inside_it() {
    const NET: &str = "require 'ipaddr'\nnet = IPAddr.new('192.168.2.0/24')\n";
    assert_eq!(
        shown(&format!("{NET}net.include?(IPAddr.new('192.168.2.255'))")),
        "true"
    );
    assert_eq!(
        shown(&format!("{NET}net.include?(IPAddr.new('192.168.3.0'))")),
        "false"
    );
    assert_eq!(
        shown(&format!(
            "{NET}net.include?((192 << 24) + (168 << 16) + (2 << 8) + 13)"
        )),
        "true"
    );
    assert_eq!(
        shown(&format!("{NET}net === IPAddr.new('192.168.2.1')")),
        "true"
    );
}

#[test]
fn two_addresses_are_equal_when_they_name_the_same_number() {
    assert_eq!(
        shown("require 'ipaddr'\nIPAddr.new('3ffe:505:2::') == IPAddr.new('3ffe:505:2::')"),
        "true"
    );
    assert_eq!(
        shown("require 'ipaddr'\nIPAddr.new('3ffe:505:2::') == IPAddr.new('3ffe:505:3::')"),
        "false"
    );
    assert_eq!(
        shown("require 'ipaddr'\nIPAddr.new('1.1.1.1') == 'sometext'"),
        "false"
    );
    assert_eq!(
        shown("require 'ipaddr'\nIPAddr.new('1.1.1.1') <=> IPAddr.new('1.1.1.2')"),
        "-1"
    );
}

#[test]
fn an_address_names_the_entry_a_reverse_lookup_is_made_under() {
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('192.168.2.1').reverse"),
        "1.2.168.192.in-addr.arpa"
    );
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('3ffe:505:2::f').ip6_arpa"),
        "f.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.2.0.0.0.5.0.5.0.e.f.f.3.ip6.arpa"
    );
    assert_eq!(
        quoted("require 'ipaddr'\nIPAddr.new('3ffe:505:2::f').ip6_int"),
        "f.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.2.0.0.0.5.0.5.0.e.f.f.3.ip6.int"
    );
    assert!(
        run_err("require 'ipaddr'\nIPAddr.new('192.168.2.1').ip6_arpa")
            .contains("not an IPv6 address")
    );
}
