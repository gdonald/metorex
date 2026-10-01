// DRb within one process: a reference to an object on this process's own
// server is answered by the object itself, and the server refuses methods
// a remote caller may not reach.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

const SETUP: &str = "require 'drb'\nclass Front\n  def add(*numbers) = numbers.sum\n  protected def guarded = 1\nend\n";

/// Loading the drb library and marshaling through it nests deeper than the
/// stack a test thread is given, so each program runs on a thread sized like
/// the one the binary itself uses.
fn inspected(code: &str) -> String {
    let source = format!("{SETUP}({code}).inspect");
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

/// `code` run with a service started on a free port, stopped afterward.
fn served(code: &str) -> String {
    inspected(&format!(
        "DRb.start_service('druby://localhost:0', Front.new)\nbegin\n  {code}\nensure\n  DRb.stop_service\nend"
    ))
}

#[test]
fn asking_for_the_current_server_before_one_starts_raises() {
    assert_eq!(
        inspected("begin\n  DRb.current_server\nrescue => error\n  error.class\nend"),
        "DRb::DRbServerNotFound"
    );
}

#[test]
fn a_service_on_port_zero_reports_the_port_it_was_given() {
    assert_eq!(
        served("DRb.uri.match?(%r{\\Adruby://localhost:\\d+\\z})"),
        "true"
    );
}

#[test]
fn a_uri_is_here_only_when_this_process_serves_it() {
    assert_eq!(
        served("[DRb.here?(DRb.uri), DRb.here?('druby://elsewhere:1')]"),
        "[true, false]"
    );
}

#[test]
fn a_reference_to_this_process_calls_the_front_object_directly() {
    assert_eq!(served("DRbObject.new_with_uri(DRb.uri).add(2, 3)"), "5");
}

#[test]
fn two_references_to_the_same_object_are_equal_and_hash_alike() {
    assert_eq!(
        served(
            "one = DRbObject.new_with_uri(DRb.uri)\nother = DRbObject.new_with_uri(DRb.uri)\n[one == other, one.hash == other.hash]"
        ),
        "[true, true]"
    );
}

#[test]
fn respond_to_is_asked_of_the_object_except_for_marshal_hooks() {
    assert_eq!(
        served(
            "held = DRbObject.new_with_uri(DRb.uri)\n[held.respond_to?(:add), held.respond_to?(:_dump), held.respond_to?(:marshal_dump)]"
        ),
        "[true, true, false]"
    );
}

#[test]
fn a_reference_to_the_front_object_carries_no_id() {
    assert_eq!(served("DRbObject.new_with_uri(DRb.uri).__drbref"), "nil");
}

#[test]
fn a_protected_method_is_refused() {
    assert_eq!(
        served(
            "begin\n  DRbObject.new_with_uri(DRb.uri).guarded\nrescue NoMethodError => error\n  error.message.sub(/ for .*/, '')\nend"
        ),
        "\"protected method 'guarded' called\""
    );
}

#[test]
fn send_is_refused_as_insecure() {
    assert_eq!(
        served(
            "begin\n  DRb.current_server.check_insecure_method(Front.new, :__send__)\nrescue SecurityError => error\n  error.message\nend"
        ),
        "\"insecure method '__send__'\""
    );
}

#[test]
fn a_method_named_by_a_string_is_refused() {
    assert_eq!(
        served(
            "begin\n  DRb.current_server.check_insecure_method(Front.new, 'add')\nrescue ArgumentError => error\n  error.message\nend"
        ),
        "\"add:String is not a symbol\""
    );
}

#[test]
fn an_undumped_object_cannot_be_marshaled() {
    assert_eq!(
        inspected(
            "begin\n  Marshal.dump(Object.new.extend(DRbUndumped))\nrescue TypeError => error\n  error.message\nend"
        ),
        "\"can't dump\""
    );
}

#[test]
fn a_reference_to_a_local_object_loads_back_as_the_object() {
    assert_eq!(
        served("Marshal.load(Marshal.dump(DRbObject.new(Front.new))).class"),
        "Front"
    );
}

#[test]
fn a_uri_of_another_scheme_is_refused() {
    assert_eq!(
        inspected(
            "begin\n  DRbObject.new_with_uri('http://x').add(1)\nrescue DRb::DRbError => error\n  [error.class, error.message]\nend"
        ),
        "[DRb::DRbBadURI, \"can't parse uri:http://x\"]"
    );
}

#[test]
fn a_call_to_a_stopped_server_cannot_connect() {
    assert_eq!(
        inspected(
            "DRb.start_service('druby://localhost:0', Front.new)\nuri = DRb.uri\nDRb.stop_service\nbegin\n  DRbObject.new_with_uri(uri).add(1)\nrescue DRb::DRbConnError => error\n  error.class\nend"
        ),
        "DRb::DRbConnError"
    );
}

#[test]
fn the_service_answers_its_front_and_thread() {
    assert_eq!(
        served("[DRb.front.class, DRb.thread.alive?]"),
        "[Front, true]"
    );
}
