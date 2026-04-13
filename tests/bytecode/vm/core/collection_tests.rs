// Tests for bytecode VM collection execution: arrays, hashes, index ops (13.10).

use metorex::bytecode::vm::BytecodeVm;
use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use std::rc::Rc;

fn run(source: &str) -> Result<Object, String> {
    let tokens = Lexer::new(source).tokenize();
    let stmts = Parser::new(tokens).parse().map_err(|errs| {
        errs.iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    })?;
    let compiler = Compiler::new();
    let chunk = compiler.compile(&stmts).map_err(|e| e.to_string())?;
    let mut vm = BytecodeVm::new();
    vm.execute(&chunk).map_err(|e| e.to_string())
}

fn run_ok(source: &str) -> Object {
    run(source).expect("execution failed")
}

// ── 13.10 Collection Execution ──────────────────────────────────────

#[test]
fn execute_array_literal() {
    let result = run_ok("return [1, 2, 3]");
    match result {
        Object::Array(arr) => {
            let arr = arr.borrow();
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Object::Int(1));
            assert_eq!(arr[2], Object::Int(3));
        }
        _ => panic!("Expected Array, got {:?}", result),
    }
}

#[test]
fn execute_hash_literal() {
    let result = run_ok("return {\"a\" => 1, \"b\" => 2}");
    match result {
        Object::Dict(dict) => {
            let dict = dict.borrow();
            assert_eq!(dict.len(), 2);
            assert_eq!(*dict.get("a").unwrap(), Object::Int(1));
        }
        _ => panic!("Expected Dict, got {:?}", result),
    }
}

#[test]
fn execute_array_index_get() {
    let result = run_ok("a = [10, 20, 30]\nreturn a[1]");
    assert_eq!(result, Object::Int(20));
}

#[test]
fn execute_array_index_set() {
    let result = run_ok("a = [1, 2, 3]\na[0] = 99\nreturn a[0]");
    assert_eq!(result, Object::Int(99));
}

#[test]
fn execute_hash_with_non_string_key() {
    let chunk = {
        let tokens = Lexer::new("return {1 => \"one\"}").tokenize();
        let stmts = Parser::new(tokens)
            .parse()
            .map_err(|errs| {
                errs.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap();
        Compiler::new().compile(&stmts).unwrap()
    };
    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk);
    assert!(result.is_ok());
}

#[test]
fn execute_index_get_type_error() {
    let result = run("return 42[0]");
    assert!(result.is_err());
}

#[test]
fn execute_index_set_out_of_bounds() {
    let result = run("a = [1]\na[5] = 99");
    assert!(result.is_err());
}

#[test]
fn execute_index_set_type_error() {
    let result = run("a = 42\na[0] = 1");
    assert!(result.is_err());
}

// ── Dict index operations ───────────────────────────────────────────

#[test]
fn execute_dict_index_get() {
    let result = run_ok("h = {\"a\" => 1}\nreturn h[\"a\"]");
    assert_eq!(result, Object::Int(1));
}

#[test]
fn execute_dict_index_get_missing() {
    let result = run_ok("h = {\"a\" => 1}\nreturn h[\"z\"]");
    assert_eq!(result, Object::Nil);
}

#[test]
fn execute_dict_index_set() {
    let result = run_ok("h = {\"a\" => 1}\nh[\"b\"] = 2\nreturn h[\"b\"]");
    assert_eq!(result, Object::Int(2));
}

// ── Array negative index ────────────────────────────────────────────

#[test]
fn execute_array_negative_index() {
    let result = run_ok("a = [10, 20, 30]\nreturn a[-1]");
    assert_eq!(result, Object::Int(30));
}

#[test]
fn execute_array_negative_index_set() {
    let result = run_ok("a = [10, 20, 30]\na[-1] = 99\nreturn a[2]");
    assert_eq!(result, Object::Int(99));
}

// ── OP_CLOSE_UPVALUE ───────────────────────────────────────────────

#[test]
fn execute_close_upvalue_via_closure() {
    let src = r#"
def make_counter
  count = 0
  def increment
    count = count + 1
    return count
  end
  return increment
end
f = make_counter()
f()
return f()
"#;
    let result = run_ok(src);
    assert_eq!(result, Object::Int(2));
}

// ── Mixed int/float arithmetic ──────────────────────────────────────

#[test]
fn execute_int_float_addition() {
    let result = run_ok("return 1 + 2.5");
    assert_eq!(result, Object::Float(3.5));
}

#[test]
fn execute_int_float_comparison() {
    let result = run_ok("return 1 < 2.5");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_int_minus_float() {
    let result = run_ok("return 5 - 2.5");
    assert_eq!(result, Object::Float(2.5));
}

#[test]
fn execute_int_times_float() {
    let result = run_ok("return 3 * 2.5");
    assert_eq!(result, Object::Float(7.5));
}

// ── String concatenation ────────────────────────────────────────────

#[test]
fn execute_string_concat() {
    let result = run_ok("return \"hello\" + \" world\"");
    assert_eq!(result, Object::String(Rc::new("hello world".to_string())));
}
