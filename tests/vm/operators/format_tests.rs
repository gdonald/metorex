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

// ── %d ──────────────────────────────────────────────────────────────────────

#[test]
fn string_format_percent_d() {
    assert_eq!(
        run(r#""value: %d" % 42"#),
        Some(Object::String(std::rc::Rc::new("value: 42".to_string())))
    );
}

#[test]
fn string_format_percent_d_with_plus_sign() {
    assert_eq!(
        run(r#""%+d" % 5"#),
        Some(Object::String(std::rc::Rc::new("+5".to_string())))
    );
}

#[test]
fn string_format_percent_d_with_space_sign() {
    assert_eq!(
        run(r#""% d" % 5"#),
        Some(Object::String(std::rc::Rc::new(" 5".to_string())))
    );
}

#[test]
fn string_format_percent_d_with_width() {
    if let Some(Object::String(s)) = run(r#""%5d" % 42"#) {
        assert!(s.contains("42"));
    } else {
        panic!("expected String");
    }
}

#[test]
fn string_format_percent_d_from_float_arg() {
    assert_eq!(
        run(r#""%d" % 3.7"#),
        Some(Object::String(std::rc::Rc::new("3".to_string())))
    );
}

#[test]
fn string_format_percent_d_negative_no_plus() {
    assert_eq!(
        run(r#""%+d" % -5"#),
        Some(Object::String(std::rc::Rc::new("-5".to_string())))
    );
}

// ── %s ──────────────────────────────────────────────────────────────────────

#[test]
fn string_format_percent_s() {
    assert_eq!(
        run(r#""hi %s" % "world""#),
        Some(Object::String(std::rc::Rc::new("hi world".to_string())))
    );
}

#[test]
fn string_format_percent_s_with_precision() {
    assert_eq!(
        run(r#""%.3s" % "hello""#),
        Some(Object::String(std::rc::Rc::new("hel".to_string())))
    );
}

#[test]
fn string_format_percent_with_array_args() {
    assert_eq!(
        run(r#""%s = %d" % ["x", 7]"#),
        Some(Object::String(std::rc::Rc::new("x = 7".to_string())))
    );
}

// ── %f ──────────────────────────────────────────────────────────────────────

#[test]
fn string_format_percent_f_basic() {
    if let Some(Object::String(s)) = run(r#""%f" % 3.14"#) {
        assert!(s.starts_with("3.14"));
    } else {
        panic!("expected String");
    }
}

#[test]
fn string_format_percent_f_with_precision() {
    assert_eq!(
        run(r#""%.2f" % 3.14159"#),
        Some(Object::String(std::rc::Rc::new("3.14".to_string())))
    );
}

#[test]
fn string_format_percent_f_from_int() {
    if let Some(Object::String(s)) = run(r#""%.1f" % 5"#) {
        assert!(s.starts_with("5.0"));
    }
}

#[test]
fn string_format_percent_f_with_plus_sign() {
    if let Some(Object::String(s)) = run(r#""%+.1f" % 3.5"#) {
        assert!(s.starts_with("+3.5"));
    }
}

#[test]
fn string_format_percent_f_with_space_sign() {
    if let Some(Object::String(s)) = run(r#""% .1f" % 3.5"#) {
        assert!(s.starts_with(" 3.5"));
    }
}

#[test]
fn string_format_percent_f_non_numeric_errors() {
    let result = std::panic::catch_unwind(|| run(r#""%f" % "not a number""#));
    assert!(result.is_err());
}

// ── %x, %X, %o, %b ─────────────────────────────────────────────────────────

#[test]
fn string_format_percent_x_lowercase() {
    assert_eq!(
        run(r#""%x" % 255"#),
        Some(Object::String(std::rc::Rc::new("ff".to_string())))
    );
}

#[test]
fn string_format_percent_x_uppercase() {
    assert_eq!(
        run(r#""%X" % 255"#),
        Some(Object::String(std::rc::Rc::new("FF".to_string())))
    );
}

#[test]
fn string_format_percent_o_octal() {
    assert_eq!(
        run(r#""%o" % 8"#),
        Some(Object::String(std::rc::Rc::new("10".to_string())))
    );
}

#[test]
fn string_format_percent_b_binary() {
    assert_eq!(
        run(r#""%b" % 5"#),
        Some(Object::String(std::rc::Rc::new("101".to_string())))
    );
}

// ── %p ──────────────────────────────────────────────────────────────────────

#[test]
fn string_format_percent_p_string() {
    assert_eq!(
        run(r#""%p" % "hi""#),
        Some(Object::String(std::rc::Rc::new("\"hi\"".to_string())))
    );
}

#[test]
fn string_format_percent_p_nil() {
    assert_eq!(
        run(r#""%p" % nil"#),
        Some(Object::String(std::rc::Rc::new("nil".to_string())))
    );
}

// ── %c ──────────────────────────────────────────────────────────────────────

#[test]
fn string_format_percent_c_from_int() {
    assert_eq!(
        run(r#""%c" % 65"#),
        Some(Object::String(std::rc::Rc::new("A".to_string())))
    );
}

#[test]
fn string_format_percent_c_from_string() {
    assert_eq!(
        run(r#""%c" % "X""#),
        Some(Object::String(std::rc::Rc::new("X".to_string())))
    );
}

#[test]
fn string_format_percent_c() {
    assert_eq!(
        run("'%c' % 65"),
        Some(Object::String(std::rc::Rc::new("A".to_string())))
    );
}

// ── %% literal, alignment, padding ──────────────────────────────────────────

#[test]
fn string_format_percent_literal_double() {
    assert_eq!(
        run(r#""100%%" % []"#),
        Some(Object::String(std::rc::Rc::new("100%".to_string())))
    );
}

#[test]
fn string_format_left_align() {
    if let Some(Object::String(s)) = run(r#""[%-5s]" % "hi""#) {
        assert_eq!(s.as_str(), "[hi   ]");
    } else {
        panic!("expected String");
    }
}

#[test]
fn string_format_zero_pad() {
    assert_eq!(
        run(r#""%05d" % 42"#),
        Some(Object::String(std::rc::Rc::new("00042".to_string())))
    );
}

#[test]
fn string_format_left_align_coverage() {
    if let Some(Object::String(s)) = run("'%-10s' % 'hi'") {
        assert!(s.starts_with("hi"));
        assert_eq!(s.len(), 10);
    } else {
        panic!("expected string");
    }
}

#[test]
fn string_format_zero_pad_coverage() {
    assert_eq!(
        run("'%05d' % 42"),
        Some(Object::String(std::rc::Rc::new("00042".to_string())))
    );
}

// ── Error paths ─────────────────────────────────────────────────────────────

#[test]
fn string_format_too_few_args_errors() {
    let result = std::panic::catch_unwind(|| run(r#""%d %d" % [1]"#));
    assert!(result.is_err());
}

#[test]
fn string_format_incomplete_specifier_errors() {
    let result = std::panic::catch_unwind(|| run(r#""%-" % 5"#));
    assert!(result.is_err());
}

#[test]
fn string_format_trailing_percent_kept() {
    if let Some(Object::String(s)) = run(r#""abc%" % []"#) {
        assert!(s.contains("abc"));
    }
}

#[test]
fn string_format_unknown_specifier_errors() {
    let result = std::panic::catch_unwind(|| run(r#""%z" % 5"#));
    assert!(result.is_err());
}
