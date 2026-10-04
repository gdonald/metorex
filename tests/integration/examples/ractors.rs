// Ractors: blocks run apart from the main program, handing values back.

use super::run_example;

/// The expected output of both `ractors/running_a_ractor` variants.
const RUNNING_A_RACTOR_OUTPUT: &str = concat!(
    "#<Ractor:#1 running>\n",
    "true\n",
    "true\n",
    "1\n",
    "\"adder\"\n",
    "[Ractor, true, false, 3, \"adder\"]\n",
    "\"#<Ractor:#2 adder WRITTEN_AT terminated>\"\n",
    "[\"thrown by remote Ractor.\", ArgumentError, \"bad\", true]\n",
    "true\n",
    ":done\n",
    "false\n",
    ":shared\n",
    "true\n",
    "true\n",
    "1\n",
    "\"no implicit conversion of Integer into String\"\n",
    "\"must be called with a block\"\n",
    "[Ractor::RemoteError, Ractor::Error, RuntimeError, StandardError]\n",
    "[StopIteration, Ractor::Error, Ractor::Error]\n",
);

#[test]
fn test_ractors_running_a_ractor_execution() {
    let output = run_example("ractors/running_a_ractor.rb");
    assert_eq!(output, RUNNING_A_RACTOR_OUTPUT);
}

#[test]
fn test_ractors_running_a_ractor_no_parens_execution() {
    let output = run_example("ractors/running_a_ractor_no_parens.rb");
    assert_eq!(output, RUNNING_A_RACTOR_OUTPUT);
}

/// The expected output of both `ractors/isolating_a_ractor` variants.
const ISOLATING_A_RACTOR_OUTPUT: &str = concat!(
    "[ArgumentError, \"can not isolate a Proc because it accesses outer variables (first, second).\"]\n",
    "[ArgumentError, \"can not isolate a Proc because it accesses outer variables (first).\"]\n",
    "[2]\n",
    "[1]\n",
    "[Ractor::IsolationError, \"can not access global variable $setting from non-main Ractor\"]\n",
    "[Ractor::IsolationError, \"can not access global variable $setting from non-main Ractor\"]\n",
    "IO\n",
    "[Ractor::IsolationError, \"can not access global variable $0 from non-main Ractor\"]\n",
    "[Ractor::IsolationError, \"can not access class variables from non-main Ractors (@@count from Box)\"]\n",
    "[Ractor::IsolationError, \"can not access non-shareable objects in constant Box::LIST by non-main Ractor.\"]\n",
    "[1, 2]\n",
    "[Ractor::IsolationError, \"can not access non-shareable objects in constant Object::LIST by non-main Ractor.\"]\n",
    "[Ractor::IsolationError, \"can not access non-shareable objects in constant Object::LIST by non-main Ractor.\"]\n",
    "[Ractor::IsolationError, \"can not set constants with non-shareable objects by non-main Ractors\"]\n",
    "[true, true, true, true, true, true, true]\n",
    "[false, false, false]\n",
    "true\n",
    "[true, true, true, true]\n",
    "true\n",
    "[false, false, true]\n",
    "[Ractor::IsolationError, \"Proc's self is not shareable: #<Proc:ADDRESS WRITTEN_AT>\"]\n",
    "[Ractor::IsolationError, \"cannot make a shareable Proc because it can refer unshareable object \\\"open\\\" from variable 'text'\"]\n",
    "[ArgumentError, \"can not make a Proc shareable because it accesses outer variables (count).\"]\n",
    "[Ractor::IsolationError, \"cannot make a shareable Proc because the outer variable 'count' may be reassigned.\"]\n",
    "[2, true, true, false]\n",
    "[3, true]\n",
    "[true, 3]\n",
    "[Ractor::Error, \"can not make shareable object for #<Thread::Mutex:ADDRESS>\"]\n",
);

#[test]
fn test_ractors_isolating_a_ractor_execution() {
    let output = run_example("ractors/isolating_a_ractor.rb");
    assert_eq!(output, ISOLATING_A_RACTOR_OUTPUT);
}

#[test]
fn test_ractors_isolating_a_ractor_no_parens_execution() {
    let output = run_example("ractors/isolating_a_ractor_no_parens.rb");
    assert_eq!(output, ISOLATING_A_RACTOR_OUTPUT);
}

/// The expected output of both `ractors/passing_messages` variants.
const PASSING_MESSAGES_OUTPUT: &str = concat!(
    "#<Ractor::Port to:#1 id:0>\n",
    "#<Ractor::Port to:#1 id:1>\n",
    "1\n",
    "2\n",
    "40\n",
    ":ok\n",
    "Ractor::Port\n",
    "Ractor::Port\n",
    "[Ractor::Error, \"only allowed from the creator Ractor of this port\"]\n",
    "#<Ractor::Port to:#1 id:4>\n",
    "#<Ractor::Port to:#3 id:0>\n",
    "true\n",
    "[Ractor::ClosedError, \"The port was already closed\"]\n",
    "[Ractor::ClosedError, \"The port was already closed\"]\n",
    "[#<Ractor::Port to:#1 id:5>, :x]\n",
    "[Ractor, :value_done]\n",
    "\"moved\"\n",
    "false\n",
    "[NoMethodError, \"undefined method 'inspect' for an instance of Ractor::MovedObject\"]\n",
    "[Ractor::MovedError, \"can not send any methods to a moved object\"]\n",
    "[Ractor::MovedError, \"can not send any methods to a moved object\"]\n",
    "BasicObject\n",
    "false\n",
    "true\n",
    "[:<<, :close, :closed?, :inspect, :receive, :send]\n",
    "[ArgumentError, \"should be Ractor::Port or Ractor\"]\n",
    "[ArgumentError, \"specify at least one Ractor::Port or Ractor\"]\n",
);

#[test]
fn test_ractors_passing_messages_execution() {
    let output = run_example("ractors/passing_messages.rb");
    assert_eq!(output, PASSING_MESSAGES_OUTPUT);
}

#[test]
fn test_ractors_passing_messages_no_parens_execution() {
    let output = run_example("ractors/passing_messages_no_parens.rb");
    assert_eq!(output, PASSING_MESSAGES_OUTPUT);
}

/// The expected output of both `ractors/local_storage` variants.
const LOCAL_STORAGE_OUTPUT: &str = concat!(
    "1\n",
    "1\n",
    "1\n",
    "2\n",
    "[nil, 3, 3]\n",
    "nil\n",
    "[nil, :first]\n",
    "[nil, :first]\n",
    "[RuntimeError, \"Cannot get ractor local storage for non-current ractor\"]\n",
    "[RuntimeError, \"Cannot set ractor local storage for non-current ractor\"]\n",
    "[TypeError, \"1 is not a symbol nor a string\"]\n",
    "[LocalJumpError, \"no block given\"]\n",
    "5\n",
    "[]\n",
    "[]\n",
    "[]\n",
);

#[test]
fn test_ractors_local_storage_execution() {
    let output = run_example("ractors/local_storage.rb");
    assert_eq!(output, LOCAL_STORAGE_OUTPUT);
}

#[test]
fn test_ractors_local_storage_no_parens_execution() {
    let output = run_example("ractors/local_storage_no_parens.rb");
    assert_eq!(output, LOCAL_STORAGE_OUTPUT);
}
