// Where a constant is found, how a private one is refused, and the
// assignments of a constant the parser refuses.

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

fn parse_error(code: &str) -> String {
    Parser::new(Lexer::new(code).tokenize())
        .parse()
        .expect_err("the code parsed")
        .iter()
        .map(|error| error.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

fn symbol(name: &str) -> Option<Object> {
    Some(Object::symbol(name.to_string()))
}

fn text(value: &str) -> Option<Object> {
    Some(Object::string(value))
}

#[test]
fn a_constant_in_a_prepended_module_is_found_ahead_of_the_global() {
    let code = "module Early\n  LEVEL = :prepended\nend\nObject::LEVEL = :global\nclass Holder\n  prepend Early\n  def self.level\n    LEVEL\n  end\nend\nHolder.level";
    assert_eq!(run(code), symbol("prepended"));
}

#[test]
fn a_constant_in_an_included_module_is_found_ahead_of_the_global() {
    let code = "module Mixed\n  LEVEL = :mixed\nend\nObject::LEVEL = :global\nclass Holder\n  include Mixed\n  def self.level\n    LEVEL\n  end\nend\nHolder.level";
    assert_eq!(run(code), symbol("mixed"));
}

#[test]
fn a_constant_only_the_global_holds_is_read_from_inside_a_class() {
    let code = "Object::LEVEL = :global\nclass Holder\n  def self.level\n    LEVEL\n  end\nend\nHolder.level";
    assert_eq!(run(code), symbol("global"));
}

#[test]
fn a_constant_in_a_module_included_at_the_top_level_is_found_from_a_module() {
    let code = "module Shared\n  RETRIES = 3\nend\ninclude Shared\nmodule Settings\n  def self.retries\n    RETRIES\n  end\nend\n[Settings.retries, ::RETRIES]";
    assert_eq!(
        run(code),
        Some(Object::array(vec![Object::Int(3), Object::Int(3)]))
    );
}

#[test]
fn a_missing_constant_in_a_class_body_asks_const_missing() {
    let code = "class Lenient\n  def self.const_missing(name)\n    name\n  end\n  FOUND = ABSENT\nend\nLenient::FOUND";
    assert_eq!(run(code), symbol("ABSENT"));
}

#[test]
fn a_missing_constant_in_a_method_is_named_under_its_class() {
    let code = "class Strict\n  def self.lookup\n    ABSENT\n  end\nend\nbegin\n  Strict.lookup\nrescue NameError => error\n  error.message\nend";
    assert_eq!(run(code), text("uninitialized constant Strict::ABSENT"));
}

#[test]
fn a_module_named_by_its_own_name_method_is_named_that_way() {
    let code = "held = Module.new do\n  def self.name\n    \"Named\"\n  end\nend\nbegin\n  held::ABSENT\nrescue NameError => error\n  error.message\nend";
    assert_eq!(run(code), text("uninitialized constant Named::ABSENT"));
}

#[test]
fn a_module_whose_inspect_answers_no_string_is_named_by_its_own_name() {
    let code = "module Plain\n  def self.inspect\n    42\n  end\nend\nbegin\n  Plain::ABSENT\nrescue NameError => error\n  error.message\nend";
    assert_eq!(run(code), text("uninitialized constant Plain::ABSENT"));
}

#[test]
fn a_private_constant_read_through_its_scope_asks_const_missing() {
    let code = "module Guarded\n  HIDDEN = 1\n  private_constant :HIDDEN\n  def self.const_missing(name)\n    :asked\n  end\nend\nGuarded::HIDDEN";
    assert_eq!(run(code), symbol("asked"));
}

#[test]
fn a_private_constant_read_through_its_scope_from_its_own_body_is_read() {
    let code = "class Vault\n  SECRET = :hidden\n  private_constant :SECRET\n  READ = Vault::SECRET\n  KNOWN = defined?(Vault::SECRET)\nend\n[Vault::READ, Vault::KNOWN]";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::symbol("hidden".to_string()),
            Object::string("constant")
        ]))
    );
}

#[test]
fn a_private_constant_of_object_is_refused_with_the_top_level_scope() {
    let code = "class Object\n  HIDDEN_TOP = 1\n  private_constant :HIDDEN_TOP\nend\nbegin\n  ::HIDDEN_TOP\nrescue NameError => error\n  [error.message, defined?(::HIDDEN_TOP), HIDDEN_TOP]\nend";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::string("private constant Object::HIDDEN_TOP referenced"),
            Object::Nil,
            Object::Int(1)
        ]))
    );
}

#[test]
fn a_top_level_constant_nothing_holds_is_not_defined() {
    assert_eq!(run("defined?(::NOTHING_HOLDS_THIS)"), Some(Object::Nil));
}

#[test]
fn a_private_module_is_refused_when_reopened_through_its_scope() {
    let code = "module Container\n  module Hidden\n  end\n  private_constant :Hidden\nend\nbegin\n  module Container::Hidden\n  end\nrescue NameError => error\n  error.message\nend";
    assert_eq!(
        run(code),
        text("private constant Container::Hidden referenced")
    );
}

#[test]
fn a_private_class_is_reopened_from_the_body_of_its_scope() {
    let code = "class Container\n  class Hidden\n  end\n  private_constant :Hidden\n  class Container::Hidden\n    VALUE = 7\n  end\n  READ = Hidden::VALUE\nend\nContainer::READ";
    assert_eq!(run(code), Some(Object::Int(7)));
}

#[test]
fn a_local_holding_a_module_is_seen_by_eval_in_a_block() {
    let code =
        "def inside\n  yield\nend\ninside do\n  held = Module.new\n  eval(\"held\").class\nend";
    assert_eq!(
        run(code).map(|class| class.to_string()),
        Some("Module".to_string())
    );
}

#[test]
fn a_constant_assigned_in_a_method_is_refused() {
    assert!(parse_error("def assigns\n  LIMIT = 1\nend").contains("dynamic constant assignment"));
}

#[test]
fn a_scoped_constant_assigned_in_a_method_is_refused() {
    assert!(
        parse_error("def assigns\n  Object::LIMIT = 1\nend")
            .contains("dynamic constant assignment")
    );
}

#[test]
fn a_top_level_constant_assigned_in_a_method_is_refused() {
    assert!(parse_error("def assigns\n  ::LIMIT = 1\nend").contains("dynamic constant assignment"));
}

#[test]
fn a_constant_among_several_targets_in_a_method_is_refused() {
    assert!(
        parse_error("def assigns\n  LIMIT, other = 1, 2\nend")
            .contains("dynamic constant assignment")
    );
}

#[test]
fn a_constant_assigned_outside_a_method_is_accepted() {
    assert_eq!(
        run("LIMIT, other = 1, 2\nLIMIT + other"),
        Some(Object::Int(3))
    );
}

#[test]
fn a_class_nested_in_a_class_inherits_from_the_class_holding_it() {
    assert_eq!(
        run(
            "class Outer; class Inner < Outer; end; class Rooted < ::Outer; end; end\n[Outer::Inner.superclass.name, Outer::Rooted.superclass.name]"
        ),
        Some(Object::array(vec![
            Object::string("Outer".to_string()),
            Object::string("Outer".to_string()),
        ]))
    );
}

#[test]
fn a_module_body_names_itself_as_a_module() {
    assert_eq!(
        run("module Holder; KIND = Holder.class; end\nHolder::KIND.name"),
        Some(Object::string("Module".to_string()))
    );
}
