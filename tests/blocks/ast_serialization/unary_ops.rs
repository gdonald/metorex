// Unary op serialization tests

use super::eval;
use metorex::object::Object;

#[test]
fn unary_op_minus_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a)
    -a
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string("-")));
}

#[test]
fn unary_op_not_serializes_op() {
    let result = eval(
        r#"
class C
  def f(a)
    !a
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string("!")));
}

#[test]
fn unary_op_type_is_unary_op() {
    let result = eval(
        r#"
class C
  def f(a)
    -a
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("UnaryOp")));
}

#[test]
fn unary_plus_serializes_op() {
    let result = eval(
        r#"
class C
  def f(x)
    +x
  end
end
C.new.get_source(:f).body[0]["op"]
"#,
    );
    assert_eq!(result, Some(Object::string("+")));
}
