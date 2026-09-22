// The class a class stands under, and the modules it holds.

use super::*;

#[test]
fn ancestors_of_integer() {
    // Integer, Numeric, Comparable, Object, Kernel, BasicObject.
    let result = run("Integer.ancestors.length");
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn ancestors_of_custom_chain() {
    let result = run(r#"
class A
end
class B < A
end
class C < B
end
C.ancestors.length
"#);
    // C, B, A, Object, Kernel, BasicObject — Object chain joins at A because
    // user classes without an explicit `<` parent inherit from Object.
    assert_eq!(result, Some(Object::Int(6)));
}

// ============================================================================
// instance_variables
// ============================================================================

#[test]
fn itself_returns_the_same_instance() {
    let result = run(r#"
class Widget
end
widget = Widget.new
widget.itself.equal?(widget)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn itself_returns_an_immediate_receiver() {
    let result = run("42.itself");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn itself_returns_a_class_receiver() {
    let result = run(r#"
class Widget
end
Widget.itself.name
"#);
    assert_eq!(result, Some(Object::string("Widget".to_string())));
}

#[test]
fn itself_with_an_argument_raises_argument_error() {
    let error = run_err("Object.new.itself(1)");
    assert!(error.contains("wrong number of arguments (given 1, expected 0)"));
}

#[test]
fn itself_is_reported_by_respond_to() {
    let result = run("Object.new.respond_to?(:itself)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn instance_variables_returns_array() {
    let result = run(r#"
class Person
  def initialize(name, age)
    @name = name
    @age = age
  end
end
p = Person.new("Alice", 30)
p.instance_variables.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn instance_variables_returns_symbols_in_declaration_order() {
    let result = run(r#"
class Recipe
  def initialize
    @c = 1
    @a = 2
    @b = 3
  end
end
Recipe.new.instance_variables.inspect
"#);
    assert_eq!(result, Some(Object::string("[:@c, :@a, :@b]".to_string())));
}

#[test]
fn instance_variables_appends_a_later_assignment_last() {
    let result = run(r#"
class Recipe
  def initialize
    @name = "stew"
  end
end
recipe = Recipe.new
recipe.instance_variable_set(:@rating, 5)
recipe.instance_variables.inspect
"#);
    assert_eq!(
        result,
        Some(Object::string("[:@name, :@rating]".to_string()))
    );
}

#[test]
fn instance_variables_keeps_position_when_reassigned() {
    let result = run(r#"
class Recipe
  def initialize
    @name = "stew"
    @servings = 4
  end
end
recipe = Recipe.new
recipe.instance_variable_set(:@name, "soup")
recipe.instance_variables.inspect
"#);
    assert_eq!(
        result,
        Some(Object::string("[:@name, :@servings]".to_string()))
    );
}

#[test]
fn instance_variables_empty_for_non_instance() {
    let result = run("42.instance_variables.length");
    assert_eq!(result, Some(Object::Int(0)));
}
