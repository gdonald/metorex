// Tests for comprehensive AST serialization coverage
// Exercises all binary/unary op types, expression types, and statement types
// via the get_source / .body AST inspection API.

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

// ── Binary op serialization ──────────────────────────────────────────────────

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

// ── Unary op serialization ───────────────────────────────────────────────────

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

// ── Expression type serialization ────────────────────────────────────────────

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

// ── Statement type serialization ─────────────────────────────────────────────

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

// ── Method object API ────────────────────────────────────────────────────────

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
fn method_owner_returns_string() {
    let result = eval(
        r#"
class MyClass
  def greet
    nil
  end
end
MyClass.new.get_source(:greet).owner.length > 0
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

// ── Block object API ─────────────────────────────────────────────────────────

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

// ── Call expression serialization (lines 113-119 in ast_methods.rs) ──────────

#[test]
fn call_with_args_serializes_type() {
    let result = eval(
        r#"
def helper(x)
  x
end
class C
  def f(n)
    helper(n)
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Call")));
}

#[test]
fn call_with_args_has_arguments() {
    let result = eval(
        r#"
def helper(x)
  x
end
class C
  def f(n)
    helper(n)
  end
end
C.new.get_source(:f).body[0]["arguments"].length
"#,
    );
    assert_eq!(result, Some(Object::Int(1)));
}

// ── Lambda expression serialization (lines 164-175 in ast_methods.rs) ────────

#[test]
fn lambda_expression_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    lambda do |x|
      x * 2
    end
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Lambda")));
}

#[test]
fn lambda_expression_has_body() {
    let result = eval(
        r#"
class C
  def f
    lambda do |x|
      x * 2
    end
  end
end
C.new.get_source(:f).body[0]["body"].length
"#,
    );
    assert_eq!(result, Some(Object::Int(1)));
}

// ── Grouped expression serialization (lines 176-179 in ast_methods.rs) ───────

#[test]
fn grouped_expression_serializes_type() {
    let result = eval(
        r#"
class C
  def f(x)
    (x + 1)
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Grouped")));
}

// ── Self in method call chain serializes as MethodCall ───────────────────────

#[test]
fn self_in_method_chain_serializes_as_method_call() {
    // In the parsed AST, `self` is an Identifier, so `self.foo` is MethodCall
    // where the receiver serializes as an Identifier
    let result = eval(
        r#"
class C
  def bar
    42
  end
  def f
    self.bar
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("MethodCall")));
}

// ── ScopeResolution serialization (lines 182-186 in ast_methods.rs) ──────────

#[test]
fn scope_resolution_serializes_type() {
    let result = eval(
        r#"
class Outer
  Inner = 42
  def f
    Outer::Inner
  end
end
Outer.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("ScopeResolution")));
}

#[test]
fn scope_resolution_has_name() {
    let result = eval(
        r#"
class Outer
  Inner = 42
  def f
    Outer::Inner
  end
end
Outer.new.get_source(:f).body[0]["name"]
"#,
    );
    assert_eq!(result, Some(Object::string("Inner")));
}

// ── Range expression serializes as Unknown (line 191 in ast_methods.rs) ──────

#[test]
fn range_expression_in_method_body_serializes_unknown() {
    let result = eval(
        r#"
class C
  def f
    1..5
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    // Range falls through to the `_` catch-all → "Unknown"
    assert_eq!(result, Some(Object::string("Unknown")));
}

// ── UnaryOp Plus serialization (line 53 in ast_methods.rs) ───────────────────

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

// ── MethodDef inside method body (line 259 in ast_methods.rs) ────────────────

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

// ── ast_methods.rs line 279: Statement::Break ────────────────────────────────

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

// ── ast_methods.rs line 280: fallthrough _ => "Unknown" ──────────────────────

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
