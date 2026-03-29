// Block object API tests

use super::eval;
use metorex::object::Object;

#[test]
fn block_call_method_invokes_block() {
    let result = eval(
        r#"
def capture(&block)
  block
end
b = capture { |x| x * 3 }
b.call(7)
"#,
    );
    assert_eq!(result, Some(Object::Int(21)));
}

#[test]
fn block_binding_returns_object() {
    let result = eval(
        r#"
def capture(&block)
  block
end
b = capture { |x| x }
binding = b.binding
binding.nil?
"#,
    );
    assert_eq!(result, Some(Object::Bool(false)));
}
