// Queue, SizedQueue, Mutex, and ConditionVariable are named under Thread and
// reached by their short names too, and a NameError dumped by Ruby loads
// back with its message, name, and arguments.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn inspected(code: &str) -> String {
    let source = format!("({code}).inspect");
    let tokens = Lexer::new(&source).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    match vm.execute_program(&statements).expect("execution failed") {
        Some(Object::String(text)) => text.as_str().to_string(),
        other => panic!("expected an inspection, got {other:?}"),
    }
}

#[test]
fn the_thread_classes_are_named_under_thread() {
    assert_eq!(
        inspected("[Mutex, Queue, SizedQueue, ConditionVariable].map(&:name)"),
        "[\"Thread::Mutex\", \"Thread::Queue\", \"Thread::SizedQueue\", \"Thread::ConditionVariable\"]"
    );
}

#[test]
fn the_short_and_long_names_reach_the_same_class() {
    assert_eq!(
        inspected(
            "[Mutex.equal?(Thread::Mutex), ConditionVariable.equal?(Thread::ConditionVariable)]"
        ),
        "[true, true]"
    );
}

#[test]
fn thread_lists_the_classes_it_holds() {
    assert_eq!(
        inspected("Thread.constants.sort"),
        "[:Backtrace, :ConditionVariable, :Mutex, :Queue, :SizedQueue]"
    );
}

#[test]
fn a_mutex_made_by_its_long_name_still_locks() {
    assert_eq!(inspected("Thread::Mutex.new.synchronize { 3 }"), "3");
}

/// What Ruby 4.0 writes for the NoMethodError `Object.new.__send__(:hidden_thing)`
/// raises at the top level of `-e`.
const RUBY_NO_METHOD_ERROR: &[u8] = b"\x04\x08o:\x12NoMethodError\x0b:\tmesgIu:\x17NameError::message>undefined method 'hidden_thing' for an instance of Object\x06:\x06EF:\x07bt[\x06I\"\x15-e:1:in '<main>'\x06;\x08T:\tname:\x11hidden_thing:\targs[\x00:\x12private_call?T:\x11bt_locations@\x07";

#[test]
fn a_name_error_dumped_by_ruby_loads_with_its_message_name_and_arguments() {
    let dumped: String = RUBY_NO_METHOD_ERROR
        .iter()
        .map(|byte| format!("\\x{byte:02x}"))
        .collect();
    assert_eq!(
        inspected(&format!(
            "loaded = Marshal.load(\"{dumped}\".b)\n[loaded.class, loaded.message, loaded.name, loaded.args]"
        )),
        "[NoMethodError, \"undefined method 'hidden_thing' for an instance of Object\", :hidden_thing, []]"
    );
}
