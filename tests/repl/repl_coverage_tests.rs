// Additional coverage tests for repl.rs format_object Display implementations

use metorex::object::Object;
use metorex::repl::{LineResult, ReplCore};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ── repl.rs lines 272-314: format_object for various types ──────────────────

#[test]
fn format_object_exception() {
    use metorex::object::Exception;
    let exc = Object::Exception(Rc::new(RefCell::new(Exception::new(
        "RuntimeError".to_string(),
        "something broke".to_string(),
    ))));
    let formatted = ReplCore::format_object(&exc);
    assert!(
        formatted.contains("Exception") || formatted.contains("something broke"),
        "Got: {}",
        formatted
    );
}

#[test]
fn format_object_set() {
    use metorex::object::ObjectHash;
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(ObjectHash::from_object(&Object::string("a")).unwrap());
    set.insert(ObjectHash::from_object(&Object::string("b")).unwrap());
    let obj = Object::Set(Rc::new(RefCell::new(set)));
    let formatted = ReplCore::format_object(&obj);
    assert!(
        formatted.contains("Set") && formatted.contains("2"),
        "Got: {}",
        formatted
    );
}

#[test]
fn format_object_binding() {
    use metorex::object::Binding;
    let binding = Object::Binding(Rc::new(Binding::new(HashMap::new())));
    let formatted = ReplCore::format_object(&binding);
    assert!(
        formatted.contains("Binding") && formatted.contains("0"),
        "Got: {}",
        formatted
    );
}

#[test]
fn format_object_class() {
    use metorex::class::Class;
    let class = Rc::new(Class::new("MyClass", None));
    let obj = Object::Class(class);
    let formatted = ReplCore::format_object(&obj);
    assert!(
        formatted.contains("Class") && formatted.contains("MyClass"),
        "Got: {}",
        formatted
    );
}

#[test]
fn format_object_module() {
    use metorex::class::Class;
    let module = Rc::new(Class::new("MyModule", None));
    let obj = Object::Module(module);
    let formatted = ReplCore::format_object(&obj);
    assert!(
        formatted.contains("Module") && formatted.contains("MyModule"),
        "Got: {}",
        formatted
    );
}

#[test]
fn format_object_instance() {
    // Create an instance through the REPL to test format_object
    let mut core = ReplCore::new();
    core.process_line("class Dog\n  def initialize(name)\n    @name = name\n  end\nend");
    let result = core.process_line("Dog.new(\"Rex\")");
    match result {
        LineResult::Value(text) => assert!(
            text.contains("Dog") && text.contains("instance"),
            "Got: {}",
            text
        ),
        other => panic!("Expected Value, got {:?}", other),
    }
}

#[test]
fn format_object_float_infinity() {
    let formatted = ReplCore::format_object(&Object::Float(f64::INFINITY));
    assert_eq!(formatted, "inf");
}

#[test]
fn format_object_float_nan() {
    let formatted = ReplCore::format_object(&Object::Float(f64::NAN));
    assert_eq!(formatted, "NaN");
}

#[test]
fn format_object_dict_multiple_entries() {
    let mut map = HashMap::new();
    map.insert("x".to_string(), Object::Int(1));
    map.insert("y".to_string(), Object::Int(2));
    let dict = Object::Dict(Rc::new(RefCell::new(map)));
    let formatted = ReplCore::format_object(&dict);
    // Entries should be sorted
    assert!(formatted.contains("\"x\" => 1"), "Got: {}", formatted);
    assert!(formatted.contains("\"y\" => 2"), "Got: {}", formatted);
}

// ── repl.rs lines 95-97: process_line command handling ──────────────────────

#[test]
fn process_line_clear_command() {
    let mut core = ReplCore::new();
    match core.process_line(".clear") {
        LineResult::Value(text) => assert!(text.contains("Metorex REPL")),
        other => panic!("Expected Value, got {:?}", other),
    }
}

#[test]
fn process_line_reset_command() {
    let mut core = ReplCore::new();
    core.process_line("x = 42");
    match core.process_line(".reset") {
        LineResult::Continue => {}
        other => panic!("Expected Continue after reset, got {:?}", other),
    }
    // After reset, x should be undefined
    match core.process_line("x") {
        LineResult::Error(_) => {}
        other => panic!(
            "Expected Error for undefined var after reset, got {:?}",
            other
        ),
    }
}

// ── repl.rs: format_object CompiledFunction ─────────────────────────────────

#[test]
fn format_object_compiled_function() {
    use metorex::object::CompiledFunction;
    let func = Object::CompiledFunction(Rc::new(CompiledFunction::new("test_fn".to_string(), 0)));
    let formatted = ReplCore::format_object(&func);
    // Just check it doesn't panic
    assert!(!formatted.is_empty());
}
