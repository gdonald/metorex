// Method object API tests

use super::eval;
use metorex::object::Object;

#[test]
fn method_name_returns_name() {
    let result = eval(
        r#"
class C
  def my_func
    nil
  end
end
C.new.get_source(:my_func).name
"#,
    );
    assert_eq!(result, Some(Object::string("my_func")));
}

#[test]
fn method_owner_returns_defining_class() {
    let result = eval(
        r#"
class MyClass
  def greet
    nil
  end
end
MyClass.new.get_source(:greet).owner == MyClass
"#,
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn method_source_location_returns_string() {
    let result = eval(
        r#"
class C
  def f
    nil
  end
end
C.new.get_source(:f).source_location.length > 0
"#,
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn method_parameters_returns_array() {
    let result = eval(
        r#"
class C
  def f(x, y)
    x + y
  end
end
C.new.get_source(:f).parameters.length
"#,
    );
    assert_eq!(result, Some(Object::Int(2)));
}
