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

#[test]
fn module_include_adds_instance_methods() {
    let result = run("
module Greetable
  def greet
    42
  end
end
class Foo
  include Greetable
end
Foo.new.greet
");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn module_extend_adds_class_methods() {
    let result = run("
module ClassOps
  def answer
    99
  end
end
class Bar
  extend ClassOps
end
Bar.answer
");
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn module_methods_can_access_instance_vars() {
    let result = run("
module Named
  def get_name
    @name
  end
end
class Person
  include Named
  def initialize(name)
    @name = name
  end
end
Person.new(\"Bob\").get_name
");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("Bob".to_string())))
    );
}

#[test]
fn multiple_modules_can_be_included() {
    let result = run("
module A
  def a_val
    1
  end
end
module B
  def b_val
    2
  end
end
class C
  include A
  include B
end
c = C.new
c.a_val + c.b_val
");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Module body features ────────────────────────────────────────────────────

#[test]
fn module_body_ivar_assignment() {
    run("module MSpec\n  @exit = nil\n  @abort = nil\nend");
}

#[test]
fn module_body_class_self_attr() {
    run("module Foo\n  class << self\n    attr_reader :bar\n  end\nend");
}

#[test]
fn module_body_ivar_and_method() {
    run("module Config\n  @debug = true\n  def self.debug\n    @debug\n  end\nend");
}

#[test]
fn module_class_self_method_def() {
    run("module Helpers\n  class << self\n    def answer\n      42\n    end\n  end\nend");
}

#[test]
fn module_class_self_attr_writer() {
    run("module Conf\n  class << self\n    attr_writer :debug\n  end\nend");
}

#[test]
fn module_class_self_attr_accessor() {
    run("module Conf\n  class << self\n    attr_accessor :verbose\n  end\nend");
}

#[test]
fn module_class_self_method_and_attr() {
    run(
        "module Helpers\n  class << self\n    attr_reader :count\n    def reset\n      42\n    end\n  end\nend",
    );
}

#[test]
fn module_class_self_attr_accessor_in_module() {
    run(
        "module Tracker\n  class << self\n    attr_accessor :count\n    attr_writer :name\n    attr_reader :id\n    def reset\n      42\n    end\n  end\nend",
    );
}

// ── autoload registration / query ───────────────────────────────────────────

#[test]
fn module_autoload_query_returns_path() {
    let result = run("module M; end\nM.autoload :Foo, \"path/to/foo.rb\"\nM.autoload?(:Foo)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new(
            "path/to/foo.rb".to_string()
        )))
    );
}

#[test]
fn module_autoload_query_unknown_returns_nil() {
    let result = run("module M; end\nM.autoload?(:Missing)");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn module_autoload_can_be_overwritten() {
    let result =
        run("module M; end\nM.autoload :X, \"a.rb\"\nM.autoload :X, \"b.rb\"\nM.autoload?(:X)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("b.rb".to_string())))
    );
}

#[test]
fn class_autoload_query_returns_path() {
    let result = run("class K; end\nK.autoload :Bar, \"bar.rb\"\nK.autoload?(:Bar)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("bar.rb".to_string())))
    );
}

#[test]
fn class_autoload_inherits_from_superclass() {
    let result =
        run("class P; end\nP.autoload :Child, \"child.rb\"\nclass C < P; end\nC.autoload?(:Child)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("child.rb".to_string())))
    );
}

#[test]
fn class_autoload_skips_inheritance_when_disabled() {
    let result = run(
        "class P; end\nP.autoload :OnlyParent, \"only.rb\"\nclass C < P; end\nC.autoload?(:OnlyParent, false)",
    );
    assert_eq!(result, Some(Object::Nil));
}

// ── const_defined? with autoload + inherit param ────────────────────────────

#[test]
fn const_defined_true_for_registered_autoload() {
    let result = run("class K; end\nK.autoload :Loadable, \"x.rb\"\nK.const_defined?(:Loadable)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn const_defined_inherit_false_skips_ancestors() {
    let result = run(
        "class P; end\nP.autoload :Inherited, \"x.rb\"\nclass C < P; end\nC.const_defined?(:Inherited, false)",
    );
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn const_defined_inherit_true_finds_ancestor_autoload() {
    let result = run(
        "class P; end\nP.autoload :InheritedTrue, \"x.rb\"\nclass C < P; end\nC.const_defined?(:InheritedTrue)",
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Qualified module-path reopening ─────────────────────────────────────────

#[test]
fn module_qualified_path_reopens_existing() {
    let result = run(
        "module Outer; module Inner; end; end\nmodule Outer::Inner\n  X = 7\nend\nOuter::Inner::X",
    );
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn module_qualified_path_three_levels() {
    let result = run(
        "module A; module B; module C; end; end; end\nmodule A::B::C\n  V = 99\nend\nA::B::C::V",
    );
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn module_qualified_path_creates_when_missing() {
    let result =
        run("module Outer; end\nmodule Outer::NewlyDefined\n  Y = 5\nend\nOuter::NewlyDefined::Y");
    assert_eq!(result, Some(Object::Int(5)));
}

// ── defined? on autoload-registered constants ───────────────────────────────

#[test]
fn defined_returns_constant_for_registered_autoload_via_scope() {
    // `defined?(M::X)` should return "constant" without triggering the
    // autoload — even if the file doesn't exist yet.
    let result = run("module M; end\nM.autoload :X, \"nonexistent_file.rb\"\ndefined?(M::X)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("constant".to_string())))
    );
}

#[test]
fn defined_returns_nil_for_unregistered_constant() {
    let result = run("module M; end\ndefined?(M::Missing)");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn defined_returns_constant_for_bare_autoload_in_lexical_scope() {
    // Inside a `module M` body, `defined?(R)` for an autoload-registered
    // constant should return "constant" without triggering the load.
    let result =
        run("module M\n  autoload :R, \"missing.rb\"\n  $checked = defined?(R)\nend\n$checked");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("constant".to_string())))
    );
}

// ── autoload constant-name validation ───────────────────────────────────────

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

#[test]
fn autoload_rejects_lowercase_constant_name() {
    let err = run_err("module M; end\nM.autoload \"a\", \"x.rb\"");
    assert!(err.contains("NameError") || err.contains("constant"));
}

#[test]
fn autoload_rejects_numeric_first_char() {
    let err = run_err("module M; end\nM.autoload \"1foo\", \"x.rb\"");
    assert!(err.contains("NameError") || err.contains("constant"));
}

#[test]
fn autoload_rejects_name_with_space() {
    let err = run_err("module M; end\nM.autoload \"a name\", \"x.rb\"");
    assert!(err.contains("NameError") || err.contains("constant"));
}

#[test]
fn autoload_accepts_valid_uppercase_name() {
    let result = run("module M; end\nM.autoload :Valid, \"x.rb\"\nM.autoload?(:Valid)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("x.rb".to_string())))
    );
}

// ── Module#=== / Object#extend (case equality + singleton mixins) ──────────

#[test]
fn module_case_equal_with_included_module() {
    let result = run("
module M
end
class C
  include M
end
M === C.new
");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn module_case_equal_with_transitive_include() {
    let result = run("
module Outer
end
module Inner
  include Outer
end
class C
  include Inner
end
Outer === C.new
");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn module_case_equal_unrelated_returns_false() {
    let result = run("
module M
end
class C
end
M === C.new
");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn object_extend_attaches_module_to_singleton() {
    let result = run("
module M
end
class C
end
o = C.new
o.extend(M)
M === o
");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_walks_mixin_chain_after_extend() {
    let result = run("
module M
end
class C
end
o = C.new
o.extend(M)
o.is_a?(M)
");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_walks_transitive_module_includes() {
    let result = run("
module Outer
end
module Inner
  include Outer
end
class C
  include Inner
end
C.new.is_a?(Outer)
");
    assert_eq!(result, Some(Object::Bool(true)));
}
