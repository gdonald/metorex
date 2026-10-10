// What a TracePoint reads, and what it refuses outside a handler.

use super::*;

#[test]
fn a_trace_reads_nothing_outside_a_handler() {
    let error = run_err("TracePoint.new(:line) {}.lineno");
    assert!(
        error.contains("access from outside"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_trace_refuses_an_event_it_does_not_know() {
    let error = run_err("TracePoint.new(:nowhere) {}");
    assert!(
        error.contains("unknown event"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_trace_needs_a_handler() {
    let error = run_err("TracePoint.new(:line)");
    assert!(
        error.contains("must be called with a block"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn allow_reentry_is_refused_outside_a_handler() {
    let error = run_err("TracePoint.allow_reentry { 1 }");
    assert!(
        error.contains("allow_reentry"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_trace_passes_over_the_core_library_it_runs_through() {
    // `upcase` is answered from the core library, whose statements a trace
    // never sees, so only the program's own two lines are counted.
    let result = run(r#"
seen = 0
tracer = TracePoint.new(:line) { |point| seen += 1 }
tracer.enable
held = "quiet".upcase
tracer.disable
seen
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("2".to_string()));
}
