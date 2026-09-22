// The locals a binding names, and the code it runs.

use super::*;

#[test]
fn an_assignment_made_through_a_binding_stays_in_it() {
    let result = run(r#"
held = binding
eval "added = 7", held
[held.local_variable_get(:added), eval("added", held)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[7, 7]".to_string())
    );
}

#[test]
fn a_binding_names_the_locals_in_force_where_it_was_taken() {
    let result = run(r#"
def holding
  first = 1
  second = 2
  binding
end

held = holding
[held.local_variables.sort, held.local_variable_get(:first)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[[:first, :second], 1]".to_string())
    );
}

#[test]
fn a_binding_takes_a_local_it_did_not_have() {
    let result = run(r#"
held = binding
answered = [held.local_variable_defined?(:added)]
held.local_variable_set :added, 3
answered << held.local_variable_defined?(:added)
answered << held.local_variable_get(:added)
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[false, true, 3]".to_string())
    );
}

#[test]
fn a_binding_refuses_a_local_nothing_bound() {
    let error = run_err(r#"binding.local_variable_get(:no_such_local)"#);
    assert!(error.contains("is not defined"), "{error}");
}

#[test]
fn a_binding_names_a_local_by_symbol_or_string_and_nothing_else() {
    let error = run_err(r#"binding.local_variable_get(7)"#);
    assert!(error.contains("is not a symbol nor a string"), "{error}");
}

#[test]
fn a_binding_says_where_it_was_taken() {
    let result = run(r#"
held = binding
answered = held.source_location
[answered.class, answered.last > 0]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[Array, true]".to_string())
    );
}

#[test]
fn a_copy_of_a_binding_shares_the_locals_it_names() {
    // A copy is shallow: the two name the same locals, so writing one is seen
    // through either. A local added afterwards belongs to the one it was
    // added to alone.
    let result = run(r#"
held = binding
held.local_variable_set :counted, 1
copied = held.dup
copied.local_variable_set :counted, 2
copied.local_variable_set :later, 3
[held.local_variable_get(:counted), copied.local_variable_get(:counted),
 held.local_variable_defined?(:later), copied.local_variable_defined?(:later)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[2, 2, false, true]".to_string())
    );
}

#[test]
fn a_binding_runs_code_where_it_was_taken() {
    let result = run(r#"
first = 4
held = binding
held.eval "first * 3"
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("12".to_string())
    );
}

#[test]
fn a_writer_written_in_a_singleton_class_body_answers_an_assignment() {
    let result = run(r#"
module Held
  class << self
    def level
      @level
    end

    def level=(value)
      @level = value * 2
    end
  end
end

Held.level = 5
Held.level
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("10".to_string())
    );
}
