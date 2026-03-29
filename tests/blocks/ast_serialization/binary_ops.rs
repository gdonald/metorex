// Binary op serialization tests

use super::eval;
use metorex::object::Object;

#[test]
fn binary_op_subtract_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a, b)
    a - b
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string("-")));
}

#[test]
fn binary_op_divide_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a, b)
    a / b
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string("/")));
}

#[test]
fn binary_op_modulo_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a, b)
    a % b
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string("%")));
}

#[test]
fn binary_op_equal_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a, b)
    a == b
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string("==")));
}

#[test]
fn binary_op_not_equal_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a, b)
    a != b
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string("!=")));
}

#[test]
fn binary_op_less_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a, b)
    a < b
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string("<")));
}

#[test]
fn binary_op_greater_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a, b)
    a > b
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string(">")));
}

#[test]
fn binary_op_less_equal_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a, b)
    a <= b
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string("<=")));
}

#[test]
fn binary_op_greater_equal_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a, b)
    a >= b
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string(">=")));
}

#[test]
fn binary_op_and_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a, b)
    a && b
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string("&&")));
}

#[test]
fn binary_op_or_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a, b)
    a || b
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string("||")));
}
