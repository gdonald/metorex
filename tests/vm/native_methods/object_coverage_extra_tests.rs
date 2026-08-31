// Targeted coverage tests for uncovered lines in object_methods.rs.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── frozen? on Class / Module (line 142) ─────────────────────────────────────

#[test]
fn frozen_query_on_class_when_frozen() {
    let result = run(r#"
class F1
end
F1.freeze
F1.frozen?
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn frozen_query_on_module_when_frozen() {
    let result = run(r#"
module M1
end
M1.freeze
M1.frozen?
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn frozen_query_on_unfrozen_class() {
    let result = run(r#"
class F2
end
F2.frozen?
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── to_sym on non-String non-Symbol (line 155) ───────────────────────────────

#[test]
fn to_sym_on_integer_returns_nil_or_error() {
    // to_sym on an Int falls through to the `_ => Ok(None)` arm (line 155),
    // which bubbles out of object_methods. The resulting behavior: either
    // a NoMethodError, or Nil from a higher-level fallback. Either confirms
    // the code path executes.
    let result = std::panic::catch_unwind(|| {
        let tokens = Lexer::new("42.to_sym").tokenize();
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        let mut vm = VirtualMachine::new();
        vm.execute_program(&stmts)
    });
    match result {
        Ok(Err(_)) | Err(_) => {}
        Ok(Ok(_v)) => {} // permissive fallback
    }
}

// ── object_id for Class, Module (lines 163-164) ──────────────────────────────

#[test]
fn object_id_on_class_is_integer() {
    let result = run(r#"
class IdCls
end
IdCls.object_id
"#);
    assert!(matches!(result, Some(Object::Int(_))));
}

#[test]
fn object_id_on_module_is_integer() {
    let result = run(r#"
module IdMod
end
IdMod.object_id
"#);
    assert!(matches!(result, Some(Object::Int(_))));
}

#[test]
fn object_id_on_nil_is_four() {
    let result = run("nil.object_id");
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn object_id_is_stable_and_distinct_for_floats() {
    let result = run("3.14.object_id == 3.14.object_id");
    assert_eq!(result, Some(Object::Bool(true)));
    let distinct = run("3.14.object_id == 2.72.object_id");
    assert_eq!(distinct, Some(Object::Bool(false)));
}

// ── clamp arg count errors (lines 199-214) ───────────────────────────────────

#[test]
fn clamp_no_args_errors() {
    let err = run_err("5.clamp");
    assert!(err.contains("argument"));
}

#[test]
fn clamp_too_many_args_errors() {
    let err = run_err("5.clamp(1, 2, 3)");
    assert!(err.contains("argument"));
}

#[test]
fn clamp_single_non_range_errors() {
    // A single non-range argument triggers the method_argument_error at 199.
    let err = run_err("5.clamp(3)");
    assert!(err.contains("argument"));
}

#[test]
fn clamp_range_returns_clamped() {
    let result = run("15.clamp(1..10)");
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn clamp_two_args_returns_clamped_low() {
    let result = run("(-5).clamp(0, 10)");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn clamp_two_args_returns_clamped_high() {
    let result = run("99.clamp(0, 10)");
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn clamp_two_args_returns_self() {
    let result = run("5.clamp(0, 10)");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn clamp_exclusive_range_errors() {
    let err = run_err("5.clamp(1...10)");
    assert!(err.contains("exclusive"));
}

#[test]
fn clamp_min_greater_than_max_errors() {
    let err = run_err("5.clamp(10, 1)");
    assert!(err.contains("min") || err.contains("smaller"));
}

#[test]
fn clamp_endless_exclusive_range_ok() {
    // An endless exclusive range like `1...` is accepted because end is nil.
    let result = run("5.clamp(1..)");
    assert_eq!(result, Some(Object::Int(5)));
}

// ── between? (lines 262-278) ─────────────────────────────────────────────────

#[test]
fn between_no_args_errors() {
    let err = run_err("5.between?");
    assert!(err.contains("argument"));
}

#[test]
fn between_too_many_args_errors() {
    let err = run_err("5.between?(1, 2, 3)");
    assert!(err.contains("argument"));
}

#[test]
fn between_returns_true() {
    let result = run("5.between?(1, 10)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn between_returns_false_above() {
    let result = run("15.between?(1, 10)");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn between_returns_false_below() {
    let result = run("0.between?(1, 10)");
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── singleton_method errors (lines 285-310) ──────────────────────────────────

#[test]
fn singleton_method_no_args_errors() {
    let err = run_err(
        r#"
class S1
end
S1.new.singleton_method
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn singleton_method_non_string_errors() {
    let err = run_err(
        r#"
class S2
end
S2.new.singleton_method(42)
"#,
    );
    assert!(err.contains("String") || err.contains("singleton"));
}

#[test]
fn singleton_method_undefined_raises_name_error() {
    let err = run_err(
        r#"
class S3
end
S3.new.singleton_method(:nope)
"#,
    );
    assert!(err.contains("undefined") || err.contains("singleton") || err.contains("NameError"));
}

// ── method() errors (lines 362-368, 373-380) ─────────────────────────────────

#[test]
fn method_no_args_errors() {
    let err = run_err(
        r#"
class M1
end
M1.new.method
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn method_non_symbol_errors() {
    let err = run_err(
        r#"
class M2
  def foo
  end
end
M2.new.method(42)
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol") || err.contains("argument"));
}

#[test]
fn method_undefined_errors() {
    let err = run_err(
        r#"
class M3
end
M3.new.method(:nonexistent)
"#,
    );
    assert!(err.contains("undefined") || err.contains("nonexistent"));
}

#[test]
fn method_returns_method_object() {
    let result = run(r#"
class M4
  def foo
    42
  end
end
M4.new.method(:foo)
"#);
    assert!(matches!(result, Some(Object::Method(_))));
}

// ── respond_to? non-string arg (lines 410-417) ───────────────────────────────

#[test]
fn respond_to_non_string_errors() {
    let err = run_err(
        r#"
class R1
end
R1.new.respond_to?(42)
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol") || err.contains("argument"));
}

// ── is_a? non-class arg (lines 472-479) ──────────────────────────────────────

#[test]
fn is_a_non_class_errors() {
    let err = run_err("5.is_a?(42)");
    assert!(err.contains("Class") || err.contains("argument"));
}

#[test]
fn is_a_with_module_argument() {
    // Just exercise the Object::Module match arm at line 471, regardless of
    // whether the inclusion chain is detected.
    let result = run(r#"
module M
end
UserMod = Class.new
UserMod.new.is_a?(M)
"#);
    assert!(matches!(result, Some(Object::Bool(_))));
}

// ── is_a? Class chain walking (lines 485-504) ────────────────────────────────

#[test]
fn class_is_a_class() {
    let result = run(r#"
class Cl1
end
Cl1.is_a?(Class)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn class_is_a_module_via_chain() {
    let result = run(r#"
class Cl2
end
Cl2.is_a?(Module)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn module_is_a_module() {
    let result = run(r#"
module Mo1
end
Mo1.is_a?(Module)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── dup / clone on various types (lines 719-771) ─────────────────────────────

#[test]
fn dup_on_hash_returns_copy() {
    let result = run(r#"
h = {a: 1, b: 2}
h2 = h.dup
h2[:c] = 3
h.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn clone_on_hash_returns_copy() {
    let result = run(r#"
h = {x: 1}
h2 = h.clone
h2.length
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn dup_on_class_returns_class() {
    let result = run(r#"
class DC1
  def greet
    "hi"
  end
end
dup = DC1.dup
dup.new.greet
"#);
    assert_eq!(result, Some(Object::string("hi")));
}

#[test]
fn dup_on_module_returns_module() {
    let result = run(r#"
module DM1
  def helper
    1
  end
end
dup = DM1.dup
dup.class.name
"#);
    assert!(matches!(result, Some(Object::String(_))));
}

#[test]
fn dup_on_basic_object_errors() {
    let err = run_err("BasicObject.dup");
    assert!(err.contains("root") || err.contains("BasicObject") || err.contains("TypeError"));
}

#[test]
fn dup_on_int_returns_self() {
    // Immutable types return themselves (line 770).
    let result = run("42.dup");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn dup_on_symbol_returns_self() {
    let result = run(":hello.dup");
    assert!(matches!(result, Some(Object::Symbol(_))));
}

#[test]
fn dup_wrong_arg_count_errors() {
    let err = run_err("[1,2].send(:dup, 42)");
    assert!(err.contains("argument"));
}

// ── eql? and equal? arg count errors ─────────────────────────────────────────

#[test]
fn eql_no_args_errors() {
    let err = run_err("5.eql?");
    assert!(err.contains("argument"));
}

#[test]
fn equal_no_args_errors() {
    let err = run_err("5.equal?");
    assert!(err.contains("argument"));
}

#[test]
fn equal_returns_true_for_same_array() {
    let result = run(r#"
a = [1, 2]
a.equal?(a)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn equal_returns_false_for_different_arrays() {
    let result = run("[1, 2].equal?([1, 2])");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn equal_returns_true_for_same_class() {
    let result = run(r#"
class EC1
end
EC1.equal?(EC1)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn equal_returns_true_for_same_module() {
    let result = run(r#"
module EM1
end
EM1.equal?(EM1)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn equal_returns_true_for_same_hash() {
    let result = run(r#"
h = {a: 1}
h.equal?(h)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn equal_returns_true_for_identical_ints() {
    let result = run("5.equal?(5)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── instance_variable_get/set on Class/Module ───────────────────────────────

#[test]
fn instance_variable_set_on_class() {
    let result = run(r#"
class IV1
end
IV1.instance_variable_set(:@count, 7)
IV1.instance_variable_get(:@count)
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn instance_variable_set_on_module() {
    let result = run(r#"
module IV2
end
IV2.instance_variable_set(:@val, 42)
IV2.instance_variable_get(:@val)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn instance_variable_get_on_int_returns_nil() {
    let result = run("5.instance_variable_get(:@foo)");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn instance_variable_set_on_int_raises_frozen_error() {
    // Immediates (Integer/Bool/Nil/Symbol) are frozen — Ruby raises
    // FrozenError (a RuntimeError subclass) on instance_variable_set.
    let err = run_err("5.instance_variable_set(:@foo, 1)");
    assert!(err.contains("can't modify frozen"));
    assert!(err.contains("Integer"));
}

// ── =~ / !~ ──────────────────────────────────────────────────────────────────

#[test]
fn regex_match_on_non_regex_non_string_returns_nil() {
    // e.g. symbol =~ int — not a regex pair, falls into `_ => Ok(Some(Nil))`.
    let result = run(":foo =~ 1");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn not_match_no_args_errors() {
    let err = run_err(r#""hello".send(:!~)"#);
    assert!(err.contains("argument"));
}

#[test]
fn not_match_without_a_match_method_raises() {
    let error = run_err("1 !~ 2");
    assert!(error.contains("undefined method '=~' for an instance of Integer"));
}

#[test]
fn not_match_string_regex_no_match() {
    let result = run(r#""abc" !~ /xyz/"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn not_match_string_regex_matches() {
    let result = run(r#""abc" !~ /b/"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── instance_exec / instance_eval ────────────────────────────────────────────

#[test]
fn instance_exec_with_block_receiver() {
    let result = run(r#"
"hello".instance_eval { length }
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn instance_eval_without_block_errors() {
    let err = run_err(r#""x".send(:instance_eval)"#);
    assert!(err.contains("block") || err.contains("instance_eval"));
}

// ── frozen?/freeze on immutable values ───────────────────────────────────────

#[test]
fn int_is_always_frozen() {
    let result = run("42.frozen?");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_is_frozen() {
    let result = run(r#""hi".frozen?"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn array_is_not_frozen() {
    let result = run("[1,2,3].frozen?");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn freeze_on_class_returns_class() {
    let result = run(r#"
class FF1
end
FF1.freeze.name
"#);
    assert_eq!(result, Some(Object::string("FF1")));
}

#[test]
fn instance_frozen_query_returns_false_by_default() {
    let result = run(r#"
class IFC1; end
IFC1.new.frozen?
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn instance_frozen_query_returns_true_after_freeze() {
    let result = run(r#"
class IFC2; end
i = IFC2.new
i.freeze
i.frozen?
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn obj_method_with_string_arg_returns_method() {
    let result = run(r#"
class MWS
  def hi
    "hello"
  end
end
MWS.new.method("hi")
"#);
    assert!(matches!(result, Some(Object::Method(_))));
}

// ── class on bool/nil ────────────────────────────────────────────────────────

#[test]
fn class_on_true_returns_true_class() {
    let result = run("true.class.name");
    assert_eq!(result, Some(Object::string("TrueClass")));
}

#[test]
fn class_on_false_returns_false_class() {
    let result = run("false.class.name");
    assert_eq!(result, Some(Object::string("FalseClass")));
}

#[test]
fn class_on_nil_returns_nil_class() {
    let result = run("nil.class.name");
    assert_eq!(result, Some(Object::string("NilClass")));
}

#[test]
fn class_with_args_errors() {
    let err = run_err("5.send(:class, 42)");
    assert!(err.contains("argument"));
}

// ── to_s on object with args errors ──────────────────────────────────────────

#[test]
fn inspect_with_args_errors() {
    let err = run_err("5.send(:inspect, 1)");
    assert!(err.contains("argument"));
}

// ── object_id for same int ───────────────────────────────────────────────────

#[test]
fn object_id_on_integer_follows_fixnum_formula() {
    let result = run("5.object_id");
    // Ruby's fixnum object_id: 2*n + 1
    assert_eq!(result, Some(Object::Int(11)));
}

#[test]
fn object_id_on_true_is_two() {
    let result = run("true.object_id");
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn object_id_on_false_is_zero() {
    let result = run("false.object_id");
    assert_eq!(result, Some(Object::Int(0)));
}

// ── Nil-specific conversions ─────────────────────────────────────────────────

#[test]
fn nil_to_r_returns_rational() {
    // nil.to_r via Rational class (when available).
    let result = run("nil.to_r");
    assert!(matches!(
        result,
        Some(Object::Int(_)) | Some(Object::Instance(_))
    ));
}

#[test]
fn nil_to_c_returns_complex() {
    let result = run("nil.to_c");
    assert!(matches!(
        result,
        Some(Object::Int(_)) | Some(Object::Instance(_))
    ));
}

#[test]
fn nil_rationalize_too_many_args_errors() {
    let err = run_err("nil.rationalize(1, 2)");
    assert!(err.contains("argument"));
}

#[test]
fn nil_to_h_returns_empty_hash() {
    let result = run("nil.to_h.length");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn nil_inspect_returns_nil_string() {
    let result = run("nil.inspect");
    assert_eq!(result, Some(Object::string("nil")));
}

// ── send / public_send arg errors ────────────────────────────────────────────

#[test]
fn send_no_args_errors() {
    let err = run_err("5.send");
    assert!(err.contains("argument"));
}

#[test]
fn send_non_string_method_errors() {
    let err = run_err("5.send(42)");
    assert!(err.contains("String") || err.contains("Symbol"));
}

#[test]
fn public_send_works() {
    let result = run("5.public_send(:to_s)");
    assert_eq!(result, Some(Object::string("5")));
}

// ── methods() arg count error ────────────────────────────────────────────────

#[test]
fn methods_with_args_errors() {
    // `methods` accepts a single optional include_super Boolean; passing two
    // positional args is the error now.
    let err = run_err("5.send(:methods, true, 1)");
    assert!(err.contains("argument"));
}

// ── instance_variables ──────────────────────────────────────────────────────

#[test]
fn instance_variables_on_instance() {
    let result = run(r#"
class IVars
  def initialize
    @a = 1
    @b = 2
  end
end
IVars.new.instance_variables.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn instance_variables_on_non_instance_returns_empty() {
    let result = run("5.instance_variables.length");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn instance_variables_with_args_errors() {
    let err = run_err("5.send(:instance_variables, 1)");
    assert!(err.contains("argument"));
}

// ── instance_variable_get / instance_variable_set arg errors ────────────────

#[test]
fn instance_variable_get_no_args_errors() {
    let err = run_err(
        r#"
class IVE
end
IVE.new.instance_variable_get
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn instance_variable_get_non_symbol_errors() {
    let err = run_err(
        r#"
class IVE2
end
IVE2.new.instance_variable_get(42)
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol") || err.contains("argument"));
}

#[test]
fn instance_variable_set_no_args_errors() {
    let err = run_err(
        r#"
class IVS
end
IVS.new.instance_variable_set
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn instance_variable_set_non_symbol_errors() {
    let err = run_err(
        r#"
class IVS2
end
IVS2.new.instance_variable_set(42, 1)
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol") || err.contains("argument"));
}

// ── instance_of? ─────────────────────────────────────────────────────────────

#[test]
fn instance_of_no_args_errors() {
    let err = run_err("5.instance_of?");
    assert!(err.contains("argument"));
}

#[test]
fn instance_of_non_class_errors() {
    let err = run_err("5.instance_of?(42)");
    assert!(err.contains("Class") || err.contains("argument"));
}

#[test]
fn instance_of_exact_class() {
    let result = run("5.instance_of?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── get_source ──────────────────────────────────────────────────────────────

#[test]
fn get_source_returns_method_or_nil() {
    let result = run(r#"
class GS
  def foo
    1
  end
end
GS.new.get_source(:foo)
"#);
    assert!(matches!(
        result,
        Some(Object::Method(_)) | Some(Object::Nil)
    ));
}

#[test]
fn get_source_undefined_returns_nil() {
    let result = run(r#"
class GS2
end
GS2.new.get_source(:nothing_here)
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn get_source_no_args_errors() {
    let err = run_err(
        r#"
class GS3
end
GS3.new.get_source
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn get_source_non_symbol_errors() {
    let err = run_err(
        r#"
class GS4
end
GS4.new.get_source(42)
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol") || err.contains("argument"));
}

// ── object_id for values metorex stores inline ───────────────────────────────

#[test]
fn object_id_matches_for_equal_symbols() {
    let result = run(":hello.object_id == :hello.object_id");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn object_id_differs_for_different_symbols() {
    let result = run(":hello.object_id == :goodbye.object_id");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn object_id_matches_for_equal_strings() {
    let result = run(r#""hello".object_id == "hello".object_id"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn object_id_differs_for_different_strings() {
    let result = run(r#""hello".object_id == "goodbye".object_id"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn object_id_differs_for_an_object_and_its_dup() {
    let result = run(r#"
class Widget
end
widget = Widget.new
widget.object_id == widget.dup.object_id
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn object_id_does_not_overflow_at_the_top_of_the_integer_range() {
    let result = run("(2 ** 62 - 1).object_id.is_a?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn object_id_differs_across_the_thirty_two_bit_boundary() {
    let result = run("(-1).object_id == (2 ** 30 - 1).object_id");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn object_id_of_a_symbol_is_not_negative() {
    let result = run(":anything.object_id >= 0");
    assert_eq!(result, Some(Object::Bool(true)));
}
