// Expression type serialization tests

use super::eval;
use metorex::object::Object;

#[test]
fn float_literal_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    3.14
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("FloatLiteral")));
}

#[test]
fn float_literal_serializes_value() {
    let result = eval(
        r#"
class C
  def f
    3.14
  end
end
C.new.get_source(:f).body[0]["value"]
"#,
    );
    assert_eq!(result, Some(Object::Float(3.14)));
}

#[test]
fn string_literal_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    "hello"
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("StringLiteral")));
}

#[test]
fn string_literal_serializes_value() {
    let result = eval(
        r#"
class C
  def f
    "hello"
  end
end
C.new.get_source(:f).body[0]["value"]
"#,
    );
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn bool_literal_true_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    true
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("BoolLiteral")));
}

#[test]
fn bool_literal_true_serializes_value() {
    let result = eval(
        r#"
class C
  def f
    true
  end
end
C.new.get_source(:f).body[0]["value"]
"#,
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn bool_literal_false_serializes_value() {
    let result = eval(
        r#"
class C
  def f
    false
  end
end
C.new.get_source(:f).body[0]["value"]
"#,
    );
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn nil_literal_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    nil
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("NilLiteral")));
}

#[test]
fn symbol_literal_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    :hello
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Symbol")));
}

#[test]
fn symbol_literal_serializes_value() {
    let result = eval(
        r#"
class C
  def f
    :hello
  end
end
C.new.get_source(:f).body[0]["value"]
"#,
    );
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn instance_variable_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    @name
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("InstanceVariable")));
}

#[test]
fn instance_variable_serializes_name() {
    let result = eval(
        r#"
class C
  def f
    @name
  end
end
C.new.get_source(:f).body[0]["name"]
"#,
    );
    assert_eq!(result, Some(Object::string("name")));
}

#[test]
fn class_variable_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    @@count
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("ClassVariable")));
}

#[test]
fn class_variable_serializes_name() {
    let result = eval(
        r#"
class C
  def f
    @@count
  end
end
C.new.get_source(:f).body[0]["name"]
"#,
    );
    assert_eq!(result, Some(Object::string("count")));
}

#[test]
fn global_variable_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    $global
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("GlobalVariable")));
}

#[test]
fn global_variable_serializes_name() {
    let result = eval(
        r#"
class C
  def f
    $global
  end
end
C.new.get_source(:f).body[0]["name"]
"#,
    );
    assert_eq!(result, Some(Object::string("global")));
}

#[test]
fn call_expression_serializes_type() {
    let result = eval(
        r#"
def helper
  0
end
class C
  def f
    helper
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Identifier")));
}

#[test]
fn method_call_serializes_type() {
    let result = eval(
        r#"
class C
  def f(arr)
    arr.length
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("MethodCall")));
}

#[test]
fn method_call_serializes_method_name() {
    let result = eval(
        r#"
class C
  def f(arr)
    arr.length
  end
end
C.new.get_source(:f).body[0]["method"]
"#,
    );
    assert_eq!(result, Some(Object::string("length")));
}

#[test]
fn array_expression_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    [1, 2, 3]
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Array")));
}

#[test]
fn array_expression_has_elements() {
    let result = eval(
        r#"
class C
  def f
    [1, 2, 3]
  end
end
C.new.get_source(:f).body[0]["elements"].length
"#,
    );
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn index_expression_serializes_type() {
    let result = eval(
        r#"
class C
  def f(arr)
    arr[0]
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Index")));
}

#[test]
fn dictionary_expression_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    {"a" => 1}
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Dictionary")));
}

#[test]
fn dictionary_has_entries() {
    let result = eval(
        r#"
class C
  def f
    {"a" => 1, "b" => 2}
  end
end
C.new.get_source(:f).body[0]["entries"].length
"#,
    );
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn self_expr_in_method_call_receiver() {
    let result = eval(
        r#"
class C
  def f
    self.class
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("MethodCall")));
}

#[test]
fn interpolated_string_serializes_type() {
    let result = eval(
        r##"
class C
  def f(name)
    "hello #{name}"
  end
end
C.new.get_source(:f).body[0]["type"]
"##,
    );
    assert_eq!(result, Some(Object::string("InterpolatedString")));
}
