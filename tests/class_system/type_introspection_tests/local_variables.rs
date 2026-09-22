// The locals in force where a call was written, and a few
// statements that answer nothing.

use super::*;

#[test]
fn local_variables_reports_top_level_locals() {
    let result = run("a = 1\nb = 2\nlocal_variables.inspect");
    assert_eq!(result, Some(Object::string("[:a, :b]".to_string())));
}

#[test]
fn local_variables_excludes_the_callers_locals_inside_a_method() {
    let result = run(r#"
outer = 1
def only_mine
  mine = 2
  local_variables
end
only_mine().inspect
"#);
    assert_eq!(result, Some(Object::string("[:mine]".to_string())));
}

#[test]
fn local_variables_reports_a_name_once_when_a_block_shadows_it() {
    let result = run(r#"
def shadowing
  name = 1
  1.times do |;name|
    return local_variables
  end
end
shadowing().inspect
"#);
    assert_eq!(result, Some(Object::string("[:name]".to_string())));
}

#[test]
fn local_variables_reports_a_bindings_locals() {
    let result = run(r#"
def bound
  first = 1
  second = 2
  binding
end
eval("local_variables", bound()).inspect
"#);
    assert_eq!(
        result,
        Some(Object::string("[:first, :second]".to_string()))
    );
}

#[test]
fn local_variables_rejects_arguments() {
    let error = run_err("local_variables(1)");
    assert!(error.contains("local_variables() expects 0 arguments, got 1"));
}

#[test]
fn local_variables_is_a_private_instance_method_on_kernel() {
    let result = run("Kernel.private_instance_methods(false).include?(:local_variables)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── case/when with no matching branch ────────────────────────────────────────

#[test]
fn case_when_without_a_matching_branch_is_nil() {
    let result = run("case 5\nwhen 1 then :one\nwhen 2 then :two\nend");
    assert_eq!(result, Some(Object::Nil));
}

// ── A bare zero-argument def name is a call ──────────────────────────────────

#[test]
fn a_bare_zero_argument_method_name_calls_it() {
    let result = run("def answer\n  42\nend\nanswer");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn a_local_holding_a_method_object_stays_a_value() {
    let result = run(r#"
def answer
  42
end
held = method(:answer)
held.class.name
"#);
    assert_eq!(result, Some(Object::string("Method".to_string())));
}

// ── File.executable? ─────────────────────────────────────────────────────────

#[test]
fn file_executable_is_false_for_a_missing_path() {
    let result = run(r#"File.executable?("/no/such/path/at/all")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn file_executable_is_true_for_a_shell_binary() {
    let result = run(r#"File.executable?("/bin/sh")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn file_executable_is_false_for_a_plain_file() {
    let result = run(r#"File.executable?("/etc/hosts")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}
