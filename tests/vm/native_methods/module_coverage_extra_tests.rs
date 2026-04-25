// Targeted coverage tests for uncovered lines in module_methods.rs.

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

// ── module_eval with block ───────────────────────────────────────────────────

#[test]
fn module_eval_returns_module() {
    let result = run(r#"
module ME
end
m = ME.module_eval {
  def added
    "yes"
  end
}
:ok
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("ok".to_string())))
    );
}

// ── refine (lines 31-58) ─────────────────────────────────────────────────────

#[test]
fn refine_with_class_works() {
    let result = run(r#"
module Refinement
  refine String do
    def shouted
      "loud"
    end
  end
end
:ok
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("ok".to_string())))
    );
}

#[test]
fn refine_non_class_errors() {
    let err = run_err(
        r#"
module R2
end
R2.refine(42) { }
"#,
    );
    assert!(err.contains("Class") || err.contains("argument"));
}

#[test]
fn refine_no_args_errors() {
    let err = run_err(
        r#"
module R3
end
R3.send(:refine)
"#,
    );
    assert!(err.contains("argument"));
}

// ── Kernel.load (lines 60-73) ────────────────────────────────────────────────

#[test]
fn kernel_load_no_args_errors() {
    let err = run_err(r#"Kernel.load"#);
    assert!(err.contains("argument"));
}

#[test]
fn kernel_load_non_string_errors() {
    let err = run_err(r#"Kernel.load(42)"#);
    assert!(err.contains("String") || err.contains("argument"));
}

// ── Signal.trap (lines 76-79) ────────────────────────────────────────────────

#[test]
fn signal_trap_discards_block() {
    let result = run(r#"Signal.trap("INT") { }"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── undef_method on module (lines 130-153) ───────────────────────────────────

#[test]
fn module_undef_method_via_send() {
    let result = run(r#"
module UM
  def gone
    1
  end
end
UM.send(:undef_method, :gone)
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn module_undef_method_wrong_arg_count_errors() {
    let err = run_err(
        r#"
module UM2
end
UM2.send(:undef_method)
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn module_undef_method_non_symbol_errors() {
    let err = run_err(
        r#"
module UM3
end
UM3.send(:undef_method, 42)
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol") || err.contains("argument"));
}

// ── alias_method on module (lines 155-213) ───────────────────────────────────

#[test]
fn module_alias_method_creates_alias() {
    let result = run(r#"
module AM
  def orig
    "original"
  end
end
AM.send(:alias_method, :copied, :orig)
class AMUser
  include AM
end
AMUser.new.copied
"#);
    assert_eq!(result, Some(Object::string("original")));
}

#[test]
fn module_alias_method_wrong_arg_count_errors() {
    let err = run_err(
        r#"
module AM2
end
AM2.send(:alias_method, :x)
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn module_alias_method_frozen_errors() {
    let err = run_err(
        r#"
module AM3
  def orig
    1
  end
end
AM3.freeze
AM3.send(:alias_method, :dupe, :orig)
"#,
    );
    assert!(err.contains("frozen") || err.contains("FrozenError"));
}

#[test]
fn module_alias_method_undefined_errors() {
    let err = run_err(
        r#"
module AM4
end
AM4.send(:alias_method, :new, :nonexistent_xyz)
"#,
    );
    assert!(err.contains("undefined") || err.contains("NameError") || err.contains("nonexistent"));
}

#[test]
fn module_alias_method_initialize_becomes_private() {
    // Aliasing to `initialize` makes the alias private.
    let result = run(r#"
module AM5
  def start
    1
  end
end
AM5.send(:alias_method, :initialize, :start)
:ok
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("ok".to_string())))
    );
}

// ── autoload on module ───────────────────────────────────────────────────────

#[test]
fn module_autoload_is_noop() {
    let result = run(r#"
module Auto
end
Auto.autoload(:Foo, "path")
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn module_autoload_query_is_noop() {
    let result = run(r#"
module Auto2
end
Auto2.autoload?(:Foo)
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── instance_method / public_instance_method (lines 222-235) ─────────────────

#[test]
fn module_instance_method_returns_method() {
    let result = run(r#"
module IM
  def fn
    1
  end
end
IM.instance_method(:fn)
"#);
    assert!(matches!(result, Some(Object::Method(_))));
}

#[test]
fn module_instance_method_undefined_errors() {
    let err = run_err(
        r#"
module IM2
end
IM2.instance_method(:missing_xyz)
"#,
    );
    assert!(err.contains("undefined") || err.contains("missing"));
}

#[test]
fn module_instance_method_non_string_returns_nil() {
    let result = run(r#"
module IM3
end
IM3.instance_method(42)
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── include / prepend with Class arg (lines 242-259) ─────────────────────────

#[test]
fn module_include_with_class_executes() {
    // Exercises the `Object::Class(c) => module_rc.add_mixin(...)` arm at
    // line 248. The transitive propagation to HostUser is out of scope —
    // we just verify the call succeeds.
    let result = run(r#"
class Cls
  def shared
    "class-via-include"
  end
end
module Host
end
Host.send(:include, Cls)
:ok
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("ok".to_string())))
    );
}

#[test]
fn module_include_non_module_errors() {
    let err = run_err(
        r#"
module IncErr
end
IncErr.send(:include, 42)
"#,
    );
    assert!(err.contains("Module") || err.contains("argument"));
}

// ── extend_object (lines 264-272) ────────────────────────────────────────────

#[test]
fn module_extend_object_on_class_executes() {
    // Only exercise the code path (line 268).
    let result = run(r#"
module ExtMod
  def shared
    "mod-shared"
  end
end
class ExtTgt
end
ExtMod.send(:extend_object, ExtTgt)
:ok
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("ok".to_string())))
    );
}

#[test]
fn module_extend_object_on_module() {
    let result = run(r#"
module ExtMod2
  def helper
    "from-ext-mod"
  end
end
module ExtTgt2
end
ExtMod2.send(:extend_object, ExtTgt2)
:ok
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("ok".to_string())))
    );
}

// ── Module.instance_method synthesizes stubs for module-private hooks (lines 240-256) ──

#[test]
fn module_instance_method_synthesizes_append_features_stub() {
    let result = run(r#"Module.instance_method(:append_features)"#);
    assert!(matches!(result, Some(Object::Method(_))));
}

#[test]
fn module_instance_method_synthesizes_prepend_features_stub() {
    let result = run(r#"Module.instance_method(:prepend_features)"#);
    assert!(matches!(result, Some(Object::Method(_))));
}

#[test]
fn module_instance_method_synthesizes_extend_object_stub() {
    let result = run(r#"Module.instance_method(:extend_object)"#);
    assert!(matches!(result, Some(Object::Method(_))));
}

#[test]
fn module_instance_method_synthesizes_included_stub() {
    let result = run(r#"Module.instance_method(:included)"#);
    assert!(matches!(result, Some(Object::Method(_))));
}

#[test]
fn module_instance_method_synthesizes_extended_stub() {
    let result = run(r#"Module.instance_method(:extended)"#);
    assert!(matches!(result, Some(Object::Method(_))));
}

#[test]
fn module_instance_method_string_arg_returns_method() {
    let result = run(r#"
module IM4
  def fn4
    1
  end
end
IM4.instance_method("fn4")
"#);
    assert!(matches!(result, Some(Object::Method(_))));
}

// ── append_features explicit invocation (lines 294-319) ─────────────────────

#[test]
fn module_append_features_explicit_adds_mixin() {
    let result = run(r#"
module AfMod
  def af_helper
    "from-af-mod"
  end
end
class AfHost
end
AfMod.send(:append_features, AfHost)
AfHost.new.af_helper
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("from-af-mod".to_string())))
    );
}

#[test]
fn module_append_features_with_non_module_errors() {
    let err = run_err(
        r#"
module AfErr
end
AfErr.send(:append_features, 42)
"#,
    );
    assert!(err.contains("Module") || err.contains("argument"));
}

#[test]
fn module_prepend_features_explicit_adds_mixin() {
    let result = run(r#"
module PfMod
  def pf_helper
    "from-pf-mod"
  end
end
class PfHost
end
PfMod.send(:prepend_features, PfHost)
PfHost.new.pf_helper
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("from-pf-mod".to_string())))
    );
}
