// A WeakRef reaches its object only while something else holds it.

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

#[test]
fn a_reference_reaches_each_kind_of_object_while_it_is_held() {
    let code = "require 'weakref'\nheld = [Object.new, [1], {a: 1}, +'text']\nheld.map { |object| WeakRef.new(object).__getobj__.equal?(object) }.uniq";
    assert_eq!(run(code), Some(Object::array(vec![Object::Bool(true)])));
}

#[test]
fn a_reference_to_each_kind_of_object_dies_with_its_last_holder() {
    let code = "require 'weakref'\ndef build(kind)\n  case kind\n  when 0 then WeakRef.new(Object.new)\n  when 1 then WeakRef.new([1])\n  when 2 then WeakRef.new({a: 1})\n  else WeakRef.new(+'text')\n  end\nend\n(0..3).map { |kind| build(kind).weakref_alive? }.uniq";
    assert_eq!(run(code), Some(Object::array(vec![Object::Bool(false)])));
}

#[test]
fn a_dead_reference_raises_ref_error() {
    let code = "require 'weakref'\ndef build\n  WeakRef.new(Object.new)\nend\nbegin\n  build.__getobj__\nrescue WeakRef::RefError => error\n  error.message\nend";
    assert_eq!(
        run(code),
        Some(Object::string("Invalid Reference - probably recycled"))
    );
}

#[test]
fn a_reference_to_a_value_never_freed_stays_alive() {
    assert_eq!(
        run("require 'weakref'\n[WeakRef.new(5).weakref_alive?, WeakRef.new(:name).__getobj__]"),
        Some(Object::array(vec![
            Object::Bool(true),
            Object::symbol("name".to_string())
        ]))
    );
}

#[test]
fn a_handle_that_names_nothing_reaches_nil() {
    assert_eq!(
        run("[__weak_target__(-1), __weak_target__(999), __weak_target__(:name)]"),
        Some(Object::array(vec![Object::Nil, Object::Nil, Object::Nil]))
    );
}
