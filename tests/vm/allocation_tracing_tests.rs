// Where ObjectSpace says an object was made while allocation tracing is on.

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

fn inspected(code: &str) -> String {
    match run(&format!("require 'objspace'\n({code}).inspect")) {
        Some(Object::String(text)) => text.as_str().to_string(),
        other => panic!("expected an inspection, got {other:?}"),
    }
}

/// `code`, followed by the class path, method name, and line recorded for
/// `made`.
fn site_of_made(code: &str) -> String {
    inspected(&format!(
        "{code}\n[ObjectSpace.allocation_class_path(made), ObjectSpace.allocation_method_id(made), ObjectSpace.allocation_sourceline(made)]"
    ))
}

#[test]
fn an_object_made_in_an_instance_method_names_its_class_and_method() {
    assert_eq!(
        site_of_made(
            "class Shelf\n  def stock\n    Object.new\n  end\nend\nmade = ObjectSpace.trace_object_allocations { Shelf.new.stock }"
        ),
        "[\"Shelf\", :stock, 4]"
    );
}

#[test]
fn an_array_made_in_a_block_inside_a_method_names_that_method() {
    assert_eq!(
        site_of_made(
            "def runner\n  yield\nend\nclass Shelf\n  def stock\n    runner { [1] }\n  end\nend\nmade = ObjectSpace.trace_object_allocations { Shelf.new.stock }"
        ),
        "[\"Shelf\", :stock, 7]"
    );
}

#[test]
fn a_hash_made_in_a_class_level_method_has_no_class_path() {
    assert_eq!(
        site_of_made(
            "class Shelf\n  def self.build\n    { size: 1 }\n  end\nend\nmade = ObjectSpace.trace_object_allocations { Shelf.build }"
        ),
        "[nil, :build, 4]"
    );
}

#[test]
fn a_string_made_in_a_singleton_class_method_has_no_class_path() {
    assert_eq!(
        site_of_made(
            "class Shelf\n  class << self\n    def label\n      \"tag\"\n    end\n  end\nend\nmade = ObjectSpace.trace_object_allocations { Shelf.label }"
        ),
        "[nil, :label, 5]"
    );
}

#[test]
fn an_object_made_at_the_top_level_names_no_method() {
    assert_eq!(
        site_of_made("made = ObjectSpace.trace_object_allocations { Object.new }"),
        "[nil, nil, 2]"
    );
}

#[test]
fn an_object_made_while_tracing_is_off_has_no_record() {
    assert_eq!(site_of_made("made = Object.new"), "[nil, nil, nil]");
}

#[test]
fn an_object_made_in_a_fiber_inside_a_method_names_that_method() {
    assert_eq!(
        site_of_made(
            "class Shelf\n  def stock\n    Fiber.new { [1] }.resume\n  end\nend\nmade = ObjectSpace.trace_object_allocations { Shelf.new.stock }"
        ),
        "[\"Shelf\", :stock, 4]"
    );
}

#[test]
fn an_object_made_in_an_instance_exec_block_names_the_method_it_was_written_in() {
    assert_eq!(
        site_of_made(
            "class Shelf\n  def stock\n    Object.new.instance_exec { [1] }\n  end\nend\nmade = ObjectSpace.trace_object_allocations { Shelf.new.stock }"
        ),
        "[\"Shelf\", :stock, 4]"
    );
}

#[test]
fn the_block_form_answers_what_the_block_does() {
    assert_eq!(
        inspected("ObjectSpace.trace_object_allocations { :done }"),
        ":done"
    );
}

#[test]
fn clearing_forgets_every_record() {
    assert_eq!(
        inspected(
            "made = ObjectSpace.trace_object_allocations { [1] }\nObjectSpace.trace_object_allocations_clear\nObjectSpace.allocation_sourceline(made)"
        ),
        "nil"
    );
}

#[test]
fn tracing_stays_on_until_each_start_has_its_stop() {
    assert_eq!(
        inspected(
            "ObjectSpace.trace_object_allocations_start\nObjectSpace.trace_object_allocations_start\nObjectSpace.trace_object_allocations_stop\nmade = [1]\nObjectSpace.trace_object_allocations_stop\nObjectSpace.allocation_sourceline(made)"
        ),
        "5"
    );
}

#[test]
fn a_stop_without_a_start_leaves_tracing_off() {
    assert_eq!(
        inspected(
            "ObjectSpace.trace_object_allocations_stop\nmade = [1]\nObjectSpace.allocation_sourceline(made)"
        ),
        "nil"
    );
}

#[test]
fn the_generation_is_an_integer() {
    assert_eq!(
        inspected(
            "made = ObjectSpace.trace_object_allocations { [1] }\nObjectSpace.allocation_generation(made).is_a?(Integer)"
        ),
        "true"
    );
}

#[test]
fn an_immediate_value_has_no_record() {
    assert_eq!(
        inspected(
            "ObjectSpace.trace_object_allocations { [nil, 42, :name].map { |held| ObjectSpace.allocation_sourcefile(held) } }"
        ),
        "[nil, nil, nil]"
    );
}

#[test]
fn an_address_a_traced_object_left_names_nothing_for_the_next_object_there() {
    assert_eq!(
        inspected(
            "def build\n  ObjectSpace.trace_object_allocations { Object.new }\n  nil\nend\n200.times { build }\nmade = Array.new(200) { Object.new }\nmade.map { |held| ObjectSpace.allocation_sourceline(held) }.compact"
        ),
        "[]"
    );
}

#[test]
fn a_command_that_is_not_a_symbol_answers_nil() {
    assert_eq!(inspected("ObjectSpace.__allocation_tracing__(1)"), "nil");
}
