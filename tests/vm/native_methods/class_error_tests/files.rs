// What a refused file or load reports.

use super::*;
// ── class_eval / module_eval string form ────────────────────────────────────

#[test]
fn class_eval_string_returns_last_value() {
    let result = run(r#"
class CeStr
end
CeStr.class_eval("1 + 1")
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn class_eval_string_evaluates_in_context_of_self() {
    let result = run(r#"
module CeSelf
end
CeSelf.class_eval("self") == CeSelf
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn class_eval_string_defines_methods() {
    let result = run(r#"
class CeDef
end
CeDef.class_eval("def greet; 'hi'; end")
CeDef.new.greet
"#);
    assert_eq!(result, Some(Object::string("hi")));
}

#[test]
fn class_eval_block_returns_last_value() {
    let result = run(r#"
module CeBlock
end
CeBlock.class_eval { 40 + 2 }
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn class_eval_block_yields_the_module() {
    let result = run(r#"
module CeYield
end
given = nil
CeYield.class_eval { |m| given = m }
given == CeYield
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn class_eval_string_uses_filename_and_lineno() {
    let result = run(r#"
module CeLoc
end
CeLoc.class_eval("[__FILE__, __LINE__]", "custom.rb", 102)
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::string("custom.rb"),
            Object::Int(102)
        ]))
    );
}

#[test]
fn class_eval_too_many_arguments_errors() {
    let err = run_err(
        r#"
class CeArgs
end
CeArgs.class_eval("1 + 1", "f", 0, "extra")
"#,
    );
    assert!(err.contains("given 4, expected 1..3"));
}

#[test]
fn class_eval_block_with_arguments_errors() {
    let err = run_err(
        r#"
class CeMix
end
CeMix.class_eval("1 + 1") { 2 }
"#,
    );
    assert!(err.contains("given 1, expected 0"));
}

#[test]
fn class_eval_non_string_code_without_to_str_errors() {
    let err = run_err(
        r#"
class CeBad
end
CeBad.class_eval(42)
"#,
    );
    assert!(err.contains("no implicit conversion of Integer into String"));
}

// ── class_exec / module_exec ────────────────────────────────────────────────

#[test]
fn class_exec_defines_method_in_receiver_scope() {
    let result = run(r#"
class CxDef
end
CxDef.class_exec { def foo; "foo"; end }
CxDef.new.foo
"#);
    assert_eq!(result, Some(Object::string("foo")));
}

#[test]
fn class_exec_passes_arguments_to_block() {
    let result = run(r#"
class CxArgs
end
CxArgs.class_exec(7) { |n| n }
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn class_exec_returns_last_value() {
    let result = run(r#"
module CxRet
end
CxRet.module_exec { 1 + 1 }
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn class_exec_without_block_raises_local_jump_error() {
    let err = run_err(
        r#"
class CxNoBlock
end
CxNoBlock.class_exec
"#,
    );
    assert!(err.contains("no block given"));
}

#[test]
fn class_exec_on_module_subclass_instance() {
    // `Sub < Module; Sub.new` is itself a module and answers class_exec.
    let result = run(r#"
class CxModSub < Module
end
CxModSub.new.class_exec { 1 + 1 }
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── private_methods on class ────────────────────────────────────────────────

#[test]
fn class_private_methods_returns_array() {
    let result = run(r#"
class WithPriv
  def public_one
    1
  end
  private
  def secret
    2
  end
end
WithPriv.private_methods(false).length
"#);
    assert!(result.is_some());
}

// ── Class#initialize ────────────────────────────────────────────────────────

#[test]
fn class_has_private_initialize() {
    let result = run("Class.private_methods.include?(:initialize)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn initialize_on_already_initialized_class_raises_type_error() {
    let err = run_err("Integer.send(:initialize)");
    assert!(err.contains("already initialized"));
}

#[test]
fn initialize_on_object_class_raises_type_error() {
    let err = run_err("Object.send(:initialize)");
    assert!(err.contains("already initialized"));
}

#[test]
fn initialize_on_basic_object_raises_type_error() {
    let err = run_err("BasicObject.send(:initialize)");
    assert!(err.contains("already initialized"));
}

#[test]
fn initialize_with_class_as_argument_raises_type_error() {
    let err = run_err("u = Class.allocate\nu.send(:initialize, Class)");
    assert!(err.contains("already initialized"));
}

#[test]
fn initialize_on_uninitialized_class_succeeds() {
    let result = run("u = Class.allocate\nu.send(:initialize)");
    assert_eq!(result, Some(Object::Nil));
}
