// Statement type serialization tests

use super::eval;
use metorex::object::Object;

#[test]
fn assignment_statement_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    x = 1
    x
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Assignment")));
}

#[test]
fn assignment_statement_serializes_value() {
    let result = eval(
        r#"
class C
  def f
    x = 42
    x
  end
end
node = C.new.get_source(:f).body[0]
node["value"]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("IntLiteral")));
}

#[test]
fn if_statement_serializes_type() {
    let result = eval(
        r#"
class C
  def f(x)
    if x > 0
      1
    end
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("If")));
}

#[test]
fn if_statement_has_then_branch() {
    let result = eval(
        r#"
class C
  def f(x)
    if x > 0
      1
    end
  end
end
C.new.get_source(:f).body[0]["then_branch"].length
"#,
    );
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn while_statement_serializes_type() {
    let result = eval(
        r#"
class C
  def f(n)
    while n > 0
      n = n - 1
    end
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("While")));
}

#[test]
fn while_statement_has_body() {
    let result = eval(
        r#"
class C
  def f(n)
    while n > 0
      n = n - 1
    end
  end
end
C.new.get_source(:f).body[0]["body"].length
"#,
    );
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn for_statement_serializes_type() {
    let result = eval(
        r#"
class C
  def f(arr)
    for x in arr
      x
    end
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("For")));
}

#[test]
fn for_statement_has_variable() {
    let result = eval(
        r#"
class C
  def f(arr)
    for item in arr
      item
    end
  end
end
C.new.get_source(:f).body[0]["variable"]
"#,
    );
    assert_eq!(result, Some(Object::string("item")));
}

#[test]
fn raise_statement_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    raise "error"
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Raise")));
}

#[test]
fn break_statement_serializes_type() {
    let result = eval(
        r#"
class C
  def f(arr)
    for x in arr
      break
    end
  end
end
node = C.new.get_source(:f).body[0]
node["body"][0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Break")));
}

#[test]
fn method_object_name_field() {
    let result = eval(
        r#"
class C
  def my_method
    nil
  end
end
C.new.get_source(:my_method).name
"#,
    );
    assert_eq!(result, Some(Object::string("my_method")));
}

#[test]
fn method_object_arity_field() {
    let result = eval(
        r#"
class C
  def f(a, b, c)
    a + b + c
  end
end
C.new.get_source(:f).arity
"#,
    );
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn return_value_type_is_nil_literal() {
    let result = eval(
        r#"
class C
  def f
    return nil
  end
end
C.new.get_source(:f).body[0]["value"]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("NilLiteral")));
}

#[test]
fn raise_exception_is_nil_when_no_exception() {
    let result = eval(
        r#"
class C
  def f
    raise
  end
end
C.new.get_source(:f).body[0]["exception"]
"#,
    );
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn if_else_branch_is_nil_when_no_else() {
    let result = eval(
        r#"
class C
  def f(x)
    if x > 0
      1
    end
  end
end
C.new.get_source(:f).body[0]["else_branch"]
"#,
    );
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn method_def_inside_method_body_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    def inner
      1
    end
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("MethodDef")));
}

#[test]
fn serialize_break_statement_type() {
    let result = eval(
        r#"
class C
  def stopper
    break
  end
end
C.new.get_source(:stopper).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Break")));
}

#[test]
fn serialize_unless_statement_gives_unknown() {
    let result = eval(
        r#"
class C
  def check
    unless true
      nil
    end
  end
end
C.new.get_source(:check).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Unknown")));
}
