// IO#wait: the event-mask form, the timeout-and-modes form, and how each
// refuses arguments it cannot use.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&statements).expect("execution failed")
}

/// `code` run with a pipe open as `reading` and `writing`, answered as
/// inspected.
fn on_a_pipe(code: &str) -> String {
    match run(&format!(
        "reading, writing = IO.pipe\nbegin\n  ({code}).inspect\nrescue ArgumentError, TypeError, IOError => error\n  [error.class, error.message].inspect\nend"
    )) {
        Some(Object::String(text)) => text.as_str().to_string(),
        other => panic!("expected an inspection, got {other:?}"),
    }
}

#[test]
fn a_float_event_mask_is_truncated() {
    assert_eq!(on_a_pipe("writing.wait(4.5, 0)"), "4");
}

#[test]
fn an_event_mask_is_converted_through_to_int() {
    assert_eq!(
        on_a_pipe("mask = Object.new\ndef mask.to_int = 4\nwriting.wait(mask, 0)"),
        "4"
    );
}

#[test]
fn an_event_mask_that_is_not_a_number_is_refused() {
    assert_eq!(
        on_a_pipe("reading.wait('x', 0)"),
        "[TypeError, \"no implicit conversion of String into Integer\"]"
    );
}

#[test]
fn a_timeout_beside_a_mode_that_is_not_a_number_is_refused() {
    assert_eq!(
        on_a_pipe("reading.wait(:r, 'x')"),
        "[TypeError, \"can't convert String into time interval\"]"
    );
}

#[test]
fn a_mask_timeout_that_is_not_a_number_is_refused() {
    assert_eq!(
        on_a_pipe("reading.wait(IO::READABLE, 'x')"),
        "[TypeError, \"can't convert String into time interval\"]"
    );
}

#[test]
fn a_mask_wait_with_no_timeout_answers_once_the_stream_is_ready() {
    assert_eq!(
        on_a_pipe("writing.write('a')\nreading.wait(IO::READABLE, nil)"),
        "1"
    );
}

#[test]
fn a_wait_that_runs_out_answers_nil() {
    assert_eq!(on_a_pipe("reading.wait(IO::READABLE, 0.03)"), "nil");
}

#[test]
fn a_wait_with_no_arguments_answers_the_stream_once_it_is_readable() {
    assert_eq!(
        on_a_pipe("writing.write('a')\nreading.wait.equal?(reading)"),
        "true"
    );
}

#[test]
fn a_priority_wait_on_a_pipe_runs_out() {
    assert_eq!(on_a_pipe("reading.wait(IO::PRIORITY, 0)"), "nil");
}

#[test]
fn wait_readable_and_wait_writable_answer_the_stream() {
    assert_eq!(
        on_a_pipe(
            "writing.write('a')\n[reading.wait_readable(0).equal?(reading), writing.wait_writable(0).equal?(writing)]"
        ),
        "[true, true]"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn the_read_end_of_a_pipe_reads_as_writable_the_way_select_reports_it() {
    assert_eq!(
        on_a_pipe(
            "writing.write('a')\n[reading.wait(IO::READABLE | IO::WRITABLE, 0), IO.select(nil, [reading], nil, 0).nil?]"
        ),
        "[5, false]"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn the_read_end_of_a_pipe_is_not_writable_the_way_poll_reports_it() {
    assert_eq!(
        on_a_pipe(
            "writing.write('a')\n[reading.wait(IO::READABLE | IO::WRITABLE, 0), IO.select(nil, [reading], nil, 0).nil?]"
        ),
        "[1, true]"
    );
}
