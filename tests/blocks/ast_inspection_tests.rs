// Tests for AST Inspection API

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn eval(source: &str) -> Option<Object> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&statements).expect("Execution failed")
}

fn get_str_field(obj: &Object, key: &str) -> String {
    match obj {
        Object::Dict(d) => match d.borrow().get(key) {
            Some(Object::String(s)) => s.as_ref().clone(),
            _ => panic!("Field '{}' not a string in {:?}", key, obj),
        },
        _ => panic!("Expected Dict, got {:?}", obj),
    }
}

#[test]
fn test_method_body_returns_array() {
    let source = r#"
class Foo
  def bar
    42
  end
end
m = Foo.new.get_source(:bar)
m.body.length
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn test_method_body_int_literal_type() {
    let source = r#"
class Foo
  def bar
    42
  end
end
m = Foo.new.get_source(:bar)
node = m.body[0]
node["type"]
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("IntLiteral")));
}

#[test]
fn test_method_body_int_literal_value() {
    let source = r#"
class Foo
  def bar
    42
  end
end
m = Foo.new.get_source(:bar)
node = m.body[0]
node["value"]
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn test_method_body_binary_op_type() {
    let source = r#"
class Calc
  def add(a, b)
    a + b
  end
end
m = Calc.new.get_source(:add)
node = m.body[0]
node["type"]
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("BinaryOp")));
}

#[test]
fn test_method_body_binary_op_operator() {
    let source = r#"
class Calc
  def add(a, b)
    a + b
  end
end
m = Calc.new.get_source(:add)
node = m.body[0]
node["op"]
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("+")));
}

#[test]
fn test_method_body_identifier_name() {
    let source = r#"
class Foo
  def get_x
    x
  end
end
m = Foo.new.get_source(:get_x)
node = m.body[0]
node["type"]
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("Identifier")));
}

#[test]
fn test_method_arity() {
    let source = r##"
class Foo
  def greet(name, greeting)
    "#{greeting}, #{name}!"
  end
end
Foo.new.get_source(:greet).arity
"##;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn test_method_arity_no_params() {
    let source = r#"
class Foo
  def nothing
    nil
  end
end
Foo.new.get_source(:nothing).arity
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn test_block_statements_returns_array() {
    let source = r#"
def capture(&block)
  block
end
b = capture do |x|
  x + 1
end
b.statements.length
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn test_block_arity() {
    let source = r#"
def capture(&block)
  block
end
b = capture do |x, y|
  x + y
end
b.arity
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn test_block_statement_type() {
    let source = r#"
def capture(&block)
  block
end
b = capture do |x|
  x * 2
end
node = b.statements[0]
node["type"]
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("BinaryOp")));
}

#[test]
fn test_method_body_return_statement() {
    let source = r#"
class Foo
  def bar
    return 42
  end
end
m = Foo.new.get_source(:bar)
node = m.body[0]
node["type"]
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("Return")));
}

#[test]
fn test_method_body_multiple_statements() {
    let source = r#"
class Foo
  def bar
    x = 1
    y = 2
    x + y
  end
end
Foo.new.get_source(:bar).body.length
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn test_node_type_checking() {
    let source = r#"
class Calc
  def square(n)
    n * n
  end
end
m = Calc.new.get_source(:square)
node = m.body[0]
node["type"] == "BinaryOp"
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Bool(true)));
}
