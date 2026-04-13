use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── Array#pack ───────────────────────────────────────────────────────────────

#[test]
fn array_pack_char() {
    let result = run("[65].pack('c')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 1);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_short() {
    let result = run("[1].pack('s')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 2);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_int() {
    let result = run("[1].pack('l')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 4);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_long() {
    let result = run("[1].pack('q')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 8);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_native_long() {
    let result = run("[0].pack('l!')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 8);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_unsupported_directive_error() {
    let err = run_err("[1].pack('Z')");
    assert!(err.contains("unsupported"));
}

#[test]
fn array_pack_wrong_arg_type_error() {
    let err = run_err("[1].pack(42)");
    assert!(err.contains("String"));
}

#[test]
fn array_pack_v_directive() {
    let result = run("[1].pack('V')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 4);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_big_q_directive() {
    let result = run("[1].pack('Q')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 8);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_big_c_directive() {
    let result = run("[65].pack('C')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 1);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_big_s_directive() {
    let result = run("[1].pack('S')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 2);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_v_lowercase_directive() {
    let result = run("[1].pack('v')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 2);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_big_i_directive() {
    let result = run("[1].pack('I')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 4);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_big_l_directive() {
    let result = run("[1].pack('L')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 4);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_big_j_directive() {
    let result = run("[1].pack('J')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 8);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_native_i_directive() {
    let result = run("[0].pack('i!')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 8);
    } else {
        panic!("expected string");
    }
}
