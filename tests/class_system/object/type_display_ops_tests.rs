// Coverage tests for Object::type_name, Display, operations (equals, hash, is_truthy),
// constructors, and lib::version

use metorex::object::{Class, Method, Object, ObjectHash};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// ── lib::version ────────────────────────────────────────────────────────────

#[test]
fn version_returns_something() {
    let v = metorex::version();
    assert!(!v.is_empty());
}

// ── type_name for all variants ──────────────────────────────────────────────

#[test]
fn type_name_nil() {
    assert_eq!(Object::Nil.type_name(), "Nil");
}

#[test]
fn type_name_bool() {
    assert_eq!(Object::Bool(true).type_name(), "Bool");
}

#[test]
fn type_name_int() {
    assert_eq!(Object::Int(0).type_name(), "Int");
}

#[test]
fn type_name_float() {
    assert_eq!(Object::Float(0.0).type_name(), "Float");
}

#[test]
fn type_name_string() {
    assert_eq!(Object::string("x").type_name(), "String");
}

#[test]
fn type_name_symbol() {
    assert_eq!(
        Object::Symbol(Rc::new("s".to_string())).type_name(),
        "Symbol"
    );
}

#[test]
fn type_name_array() {
    assert_eq!(Object::empty_array().type_name(), "Array");
}

#[test]
fn type_name_dict() {
    assert_eq!(Object::empty_dict().type_name(), "Dict");
}

#[test]
fn type_name_instance() {
    let class = Rc::new(Class::new("Foo", None));
    assert_eq!(Object::instance(class).type_name(), "Instance");
}

#[test]
fn type_name_class() {
    let class = Rc::new(Class::new("Foo", None));
    assert_eq!(Object::Class(class).type_name(), "Class");
}

#[test]
fn type_name_module() {
    let module = Rc::new(Class::new("Mod", None));
    assert_eq!(Object::Module(module).type_name(), "Module");
}

#[test]
fn type_name_method() {
    let m = Rc::new(Method::new("f".to_string(), vec![], vec![]));
    assert_eq!(Object::Method(m).type_name(), "Method");
}

#[test]
fn type_name_block() {
    use metorex::object::BlockStatement;
    let b = Rc::new(BlockStatement::new(vec![], vec![], HashMap::new()));
    assert_eq!(Object::Block(b).type_name(), "Block");
}

#[test]
fn type_name_exception() {
    assert_eq!(Object::exception("E", "m").type_name(), "Exception");
}

#[test]
fn type_name_set() {
    assert_eq!(Object::empty_set().type_name(), "Set");
}

#[test]
fn type_name_result_ok() {
    assert_eq!(Object::ok(Object::Nil).type_name(), "Result");
}

#[test]
fn type_name_result_err() {
    assert_eq!(Object::err(Object::Nil).type_name(), "Result");
}

#[test]
fn type_name_native_function() {
    assert_eq!(
        Object::NativeFunction("puts".to_string()).type_name(),
        "NativeFunction"
    );
}

#[test]
fn type_name_range() {
    let r = Object::Range {
        start: Box::new(Object::Int(1)),
        end: Box::new(Object::Int(10)),
        exclusive: false,
    };
    assert_eq!(r.type_name(), "Range");
}

#[test]
fn type_name_binding() {
    use metorex::object::Binding;
    let b = Rc::new(Binding::new(HashMap::new()));
    assert_eq!(Object::Binding(b).type_name(), "Binding");
}

#[test]
fn type_name_compiled_function() {
    use metorex::object::CompiledFunction;
    let f = Rc::new(CompiledFunction::new("f".to_string(), 0));
    assert_eq!(Object::CompiledFunction(f).type_name(), "CompiledFunction");
}

// ── Display ─────────────────────────────────────────────────────────────────

#[test]
fn display_nil() {
    assert_eq!(format!("{}", Object::Nil), "nil");
}

#[test]
fn display_bool() {
    assert_eq!(format!("{}", Object::Bool(true)), "true");
    assert_eq!(format!("{}", Object::Bool(false)), "false");
}

#[test]
fn display_int() {
    assert_eq!(format!("{}", Object::Int(42)), "42");
}

#[test]
fn display_float() {
    assert_eq!(format!("{}", Object::Float(3.14)), "3.14");
}

#[test]
fn display_string() {
    assert_eq!(format!("{}", Object::string("hello")), "hello");
}

#[test]
fn display_symbol() {
    let s = Object::Symbol(Rc::new("foo".to_string()));
    assert_eq!(format!("{}", s), ":foo");
}

#[test]
fn display_array_empty() {
    assert_eq!(format!("{}", Object::empty_array()), "[]");
}

#[test]
fn display_array_elements() {
    let arr = Object::array(vec![Object::Int(1), Object::Int(2), Object::Int(3)]);
    assert_eq!(format!("{}", arr), "[1, 2, 3]");
}

#[test]
fn display_dict_empty() {
    assert_eq!(format!("{}", Object::empty_dict()), "{}");
}

#[test]
fn display_dict_elements() {
    let mut m = HashMap::new();
    m.insert("a".to_string(), Object::Int(1));
    let d = Object::dict(m);
    let s = format!("{}", d);
    assert!(s.contains("a: 1"));
}

#[test]
fn display_instance() {
    let class = Rc::new(Class::new("Dog", None));
    let inst = Object::instance(class);
    assert_eq!(format!("{}", inst), "<Dog instance>");
}

#[test]
fn display_class() {
    let c = Rc::new(Class::new("Cat", None));
    assert_eq!(format!("{}", Object::Class(c)), "<class Cat>");
}

#[test]
fn display_module() {
    let m = Rc::new(Class::new("MyMod", None));
    assert_eq!(format!("{}", Object::Module(m)), "<module MyMod>");
}

#[test]
fn display_method() {
    let m = Rc::new(Method::new("foo".to_string(), vec![], vec![]));
    assert_eq!(format!("{}", Object::Method(m)), "<method foo>");
}

#[test]
fn display_block() {
    use metorex::object::BlockStatement;
    let b = Rc::new(BlockStatement::new(vec![], vec![], HashMap::new()));
    assert_eq!(format!("{}", Object::Block(b)), "<block>");
}

#[test]
fn display_exception() {
    let e = Object::exception("RuntimeError", "boom");
    assert_eq!(format!("{}", e), "RuntimeError: boom");
}

#[test]
fn display_set() {
    let s = Object::empty_set();
    assert_eq!(format!("{}", s), "#{}");
}

#[test]
fn display_result_ok() {
    let r = Object::ok(Object::Int(42));
    assert_eq!(format!("{}", r), "Ok(42)");
}

#[test]
fn display_result_err() {
    let r = Object::err(Object::string("bad"));
    assert_eq!(format!("{}", r), "Err(bad)");
}

#[test]
fn display_native_function() {
    let n = Object::NativeFunction("puts".to_string());
    assert_eq!(format!("{}", n), "<native function puts>");
}

#[test]
fn display_range_inclusive() {
    let r = Object::Range {
        start: Box::new(Object::Int(1)),
        end: Box::new(Object::Int(5)),
        exclusive: false,
    };
    assert_eq!(format!("{}", r), "1..5");
}

#[test]
fn display_range_exclusive() {
    let r = Object::Range {
        start: Box::new(Object::Int(1)),
        end: Box::new(Object::Int(5)),
        exclusive: true,
    };
    assert_eq!(format!("{}", r), "1...5");
}

#[test]
fn display_binding() {
    use metorex::object::Binding;
    let b = Rc::new(Binding::new(HashMap::new()));
    assert_eq!(format!("{}", Object::Binding(b)), "<Binding with 0 vars>");
}

#[test]
fn display_compiled_function() {
    use metorex::object::CompiledFunction;
    let f = Rc::new(CompiledFunction::new("my_fn".to_string(), 0));
    assert_eq!(format!("{}", Object::CompiledFunction(f)), "<fn my_fn>");
}

// ── operations: is_truthy ───────────────────────────────────────────────────

#[test]
fn is_truthy_nil_false() {
    assert!(!Object::Nil.is_truthy());
}

#[test]
fn is_truthy_false_false() {
    assert!(!Object::Bool(false).is_truthy());
}

#[test]
fn is_truthy_true_true() {
    assert!(Object::Bool(true).is_truthy());
}

#[test]
fn is_truthy_int_zero_true() {
    assert!(Object::Int(0).is_truthy());
}

#[test]
fn is_truthy_string_empty_true() {
    assert!(Object::string("").is_truthy());
}

// ── operations: equals ──────────────────────────────────────────────────────

#[test]
fn equals_nil_nil() {
    assert!(Object::Nil.equals(&Object::Nil));
}

#[test]
fn equals_bool() {
    assert!(Object::Bool(true).equals(&Object::Bool(true)));
    assert!(!Object::Bool(true).equals(&Object::Bool(false)));
}

#[test]
fn equals_int() {
    assert!(Object::Int(42).equals(&Object::Int(42)));
    assert!(!Object::Int(1).equals(&Object::Int(2)));
}

#[test]
fn equals_float_epsilon() {
    assert!(Object::Float(1.0).equals(&Object::Float(1.0)));
    assert!(!Object::Float(1.0).equals(&Object::Float(2.0)));
}

#[test]
fn equals_string() {
    assert!(Object::string("a").equals(&Object::string("a")));
    assert!(!Object::string("a").equals(&Object::string("b")));
}

#[test]
fn equals_array() {
    let a = Object::array(vec![Object::Int(1), Object::Int(2)]);
    let b = Object::array(vec![Object::Int(1), Object::Int(2)]);
    let c = Object::array(vec![Object::Int(1)]);
    assert!(a.equals(&b));
    assert!(!a.equals(&c));
}

#[test]
fn equals_dict() {
    let mut m1 = HashMap::new();
    m1.insert("x".to_string(), Object::Int(1));
    let mut m2 = HashMap::new();
    m2.insert("x".to_string(), Object::Int(1));
    let mut m3 = HashMap::new();
    m3.insert("y".to_string(), Object::Int(1));
    assert!(Object::dict(m1.clone()).equals(&Object::dict(m2)));
    assert!(!Object::dict(m1).equals(&Object::dict(m3)));
}

#[test]
fn equals_set() {
    let mut s1 = HashSet::new();
    s1.insert(ObjectHash::from_object(&Object::Int(1)).unwrap());
    let mut s2 = HashSet::new();
    s2.insert(ObjectHash::from_object(&Object::Int(1)).unwrap());
    let a = Object::Set(Rc::new(RefCell::new(s1)));
    let b = Object::Set(Rc::new(RefCell::new(s2)));
    assert!(a.equals(&b));
}

#[test]
fn equals_set_different_size() {
    let mut s1 = HashSet::new();
    s1.insert(ObjectHash::from_object(&Object::Int(1)).unwrap());
    let s2 = HashSet::new();
    let a = Object::Set(Rc::new(RefCell::new(s1)));
    let b = Object::Set(Rc::new(RefCell::new(s2)));
    assert!(!a.equals(&b));
}

#[test]
fn equals_result_ok() {
    assert!(Object::ok(Object::Int(1)).equals(&Object::ok(Object::Int(1))));
    assert!(!Object::ok(Object::Int(1)).equals(&Object::ok(Object::Int(2))));
}

#[test]
fn equals_result_err() {
    assert!(Object::err(Object::Int(1)).equals(&Object::err(Object::Int(1))));
}

#[test]
fn equals_result_mixed() {
    assert!(!Object::ok(Object::Int(1)).equals(&Object::err(Object::Int(1))));
}

#[test]
fn equals_different_types() {
    assert!(!Object::Int(1).equals(&Object::string("1")));
    assert!(!Object::Nil.equals(&Object::Bool(false)));
}

#[test]
fn equals_instance_same_rc() {
    let class = Rc::new(Class::new("X", None));
    let inst = Object::instance(class);
    // Same Rc → equal
    if let Object::Instance(rc) = &inst {
        let inst2 = Object::Instance(Rc::clone(rc));
        assert!(inst.equals(&inst2));
    }
}

#[test]
fn equals_class_same_rc() {
    let class = Rc::new(Class::new("X", None));
    let a = Object::Class(Rc::clone(&class));
    let b = Object::Class(class);
    assert!(a.equals(&b));
}

#[test]
fn equals_class_different_rc() {
    let a = Object::Class(Rc::new(Class::new("X", None)));
    let b = Object::Class(Rc::new(Class::new("X", None)));
    assert!(!a.equals(&b));
}

#[test]
fn equals_exception_same_rc() {
    let e = Object::exception("E", "m");
    if let Object::Exception(rc) = &e {
        let e2 = Object::Exception(Rc::clone(rc));
        assert!(e.equals(&e2));
    }
}

// ── operations: hash ────────────────────────────────────────────────────────

#[test]
fn hash_nil() {
    assert!(Object::Nil.hash().is_some());
}

#[test]
fn hash_bool() {
    assert!(Object::Bool(true).hash().is_some());
}

#[test]
fn hash_int() {
    assert!(Object::Int(42).hash().is_some());
}

#[test]
fn hash_float() {
    assert!(Object::Float(3.14).hash().is_some());
}

#[test]
fn hash_string() {
    assert!(Object::string("hello").hash().is_some());
}

#[test]
fn hash_array_none() {
    assert!(Object::empty_array().hash().is_none());
}

// ── constructors ────────────────────────────────────────────────────────────

#[test]
fn constructor_string() {
    let s = Object::string("test");
    assert_eq!(format!("{}", s), "test");
}

#[test]
fn constructor_empty_array() {
    if let Object::Array(arr) = Object::empty_array() {
        assert!(arr.borrow().is_empty());
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn constructor_array() {
    let a = Object::array(vec![Object::Int(1), Object::Int(2)]);
    if let Object::Array(arr) = a {
        assert_eq!(arr.borrow().len(), 2);
    }
}

#[test]
fn constructor_empty_dict() {
    if let Object::Dict(d) = Object::empty_dict() {
        assert!(d.borrow().is_empty());
    }
}

#[test]
fn constructor_dict() {
    let mut m = HashMap::new();
    m.insert("k".to_string(), Object::Int(1));
    let d = Object::dict(m);
    if let Object::Dict(d) = d {
        assert_eq!(d.borrow().len(), 1);
    }
}

#[test]
fn constructor_empty_set() {
    if let Object::Set(s) = Object::empty_set() {
        assert!(s.borrow().is_empty());
    }
}

#[test]
fn constructor_instance() {
    let class = Rc::new(Class::new("Foo", None));
    let inst = Object::instance(class);
    assert_eq!(inst.type_name(), "Instance");
}

#[test]
fn constructor_exception() {
    let e = Object::exception("RuntimeError", "oops");
    if let Object::Exception(exc) = e {
        assert_eq!(exc.borrow().exception_type, "RuntimeError");
    }
}

#[test]
fn constructor_ok() {
    let r = Object::ok(Object::Int(42));
    if let Object::Result(Ok(v)) = r {
        assert_eq!(*v, Object::Int(42));
    }
}

#[test]
fn constructor_err() {
    let r = Object::err(Object::string("bad"));
    if let Object::Result(Err(v)) = r {
        assert_eq!(v.type_name(), "String");
    }
}

#[test]
fn constructor_exception_with_cause() {
    let cause = Object::string("original");
    let e = Object::exception_with_cause("Error", "wrapped", cause);
    if let Object::Exception(exc) = e {
        assert!(exc.borrow().cause.is_some());
    }
}
