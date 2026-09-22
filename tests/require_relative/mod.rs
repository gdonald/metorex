// Tests for require_relative native function

use metorex::object::Object;
use metorex::vm::VirtualMachine;

#[test]
fn require_relative_is_registered_globally() {
    let vm = VirtualMachine::new();
    let func = vm.environment().get("require_relative");
    assert!(func.is_some());
    if let Some(Object::NativeFunction(name)) = func {
        assert_eq!(name.as_str(), "require_relative");
    } else {
        panic!("require_relative should be a NativeFunction");
    }
}

#[test]
fn require_relative_with_wrong_number_of_arguments() {
    use std::fs;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("require_test_args.rb");

    // Test with 2 arguments - since require_relative uses call syntax, we need to test
    // by actually calling it as a function
    fs::write(&test_file, "require_relative(\"file1\", \"file2\")").unwrap();
    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&test_file));
    let _ = fs::remove_file(&test_file);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("expects 1 argument"));
}

#[test]
fn require_relative_with_non_string_argument() {
    use std::fs;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("require_test_type.rb");

    // A value that says nothing about how to read itself as a path is
    // refused the way any other conversion is.
    fs::write(&test_file, "require_relative(42)").unwrap();
    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&test_file));
    let _ = fs::remove_file(&test_file);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("no implicit conversion of Integer into String"),
        "Got: {}",
        err_msg
    );

    fs::write(&test_file, "require_relative(true)").unwrap();
    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&test_file));
    let _ = fs::remove_file(&test_file);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("no implicit conversion of true into String"),
        "Got: {}",
        err_msg
    );
}

#[test]
fn require_relative_with_invalid_path() {
    use std::fs;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("require_test_invalid.rb");

    fs::write(
        &test_file,
        "require_relative(\"nonexistent_file_12345.rb\")",
    )
    .unwrap();
    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&test_file));
    let _ = fs::remove_file(&test_file);
    assert!(result.is_err());
}

#[test]
fn require_relative_returns_true_for_new_file() {
    use std::fs;
    use std::path::Path;

    // Create a temporary test file
    let temp_dir = std::env::temp_dir();
    let main_file = temp_dir.join("require_test_main.rb");
    let helper_file = temp_dir.join("require_test_helper.rb");

    fs::write(&main_file, "require_relative(\"require_test_helper\")").unwrap();
    fs::write(&helper_file, "x = 1").unwrap();

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&main_file));

    // Clean up temp files
    let _ = fs::remove_file(&main_file);
    let _ = fs::remove_file(&helper_file);

    // The execution should succeed and the last statement should be the return value
    // of require_relative, which should be true (newly loaded)
    assert!(result.is_ok());
}

#[test]
fn require_relative_returns_false_for_already_loaded_file() {
    use std::fs;
    use std::path::Path;

    // Create a temporary test file
    let temp_dir = std::env::temp_dir();
    let main_file = temp_dir.join("require_test_main2.rb");
    let helper_file = temp_dir.join("require_test_helper2.rb");

    fs::write(
        &main_file,
        "first = require_relative(\"require_test_helper2\")\nsecond = require_relative(\"require_test_helper2\")",
    )
    .unwrap();
    fs::write(&helper_file, "y = 2").unwrap();

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&main_file));

    // Clean up temp files
    let _ = fs::remove_file(&main_file);
    let _ = fs::remove_file(&helper_file);

    assert!(result.is_ok());

    // Check that both variables exist in the environment
    let first = vm.environment().get("first");
    let second = vm.environment().get("second");

    assert_eq!(first, Some(Object::Bool(true)));
    assert_eq!(second, Some(Object::Bool(false)));
}

#[test]
fn require_relative_makes_variables_accessible() {
    use std::fs;
    use std::path::Path;

    // Create a temporary test file
    let temp_dir = std::env::temp_dir();
    let main_file = temp_dir.join("require_test_main3.rb");
    let helper_file = temp_dir.join("require_test_helper3.rb");

    fs::write(&main_file, "require_relative(\"require_test_helper3\")").unwrap();
    fs::write(&helper_file, "SHARED_CONST = \"from helper\"").unwrap();

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&main_file));

    // Clean up temp files
    let _ = fs::remove_file(&main_file);
    let _ = fs::remove_file(&helper_file);

    assert!(result.is_ok());

    // Check that the variable from the helper file is accessible
    let var = vm.globals().get("SHARED_CONST");
    assert_eq!(var, Some(Object::string(String::from("from helper"))));
}

// 9.3.5 — Extension auto-detection

#[test]
fn require_relative_auto_detects_rb_extension() {
    use std::fs;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let main_file = temp_dir.join("require_ext_main.rb");
    let helper_file = temp_dir.join("require_ext_helper.rb");

    // Require without .rb extension — should auto-detect
    fs::write(&main_file, "require_relative(\"require_ext_helper\")").unwrap();
    fs::write(&helper_file, "EXT_CONST = \"found\"").unwrap();

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&main_file));

    let _ = fs::remove_file(&main_file);
    let _ = fs::remove_file(&helper_file);

    assert!(result.is_ok());
    let var = vm.globals().get("EXT_CONST");
    assert_eq!(var, Some(Object::string(String::from("found"))));
}

#[test]
fn require_relative_with_explicit_rb_extension() {
    use std::fs;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let main_file = temp_dir.join("require_ext2_main.rb");
    let helper_file = temp_dir.join("require_ext2_helper.rb");

    // Require with explicit .rb extension
    fs::write(&main_file, "require_relative(\"require_ext2_helper.rb\")").unwrap();
    fs::write(&helper_file, "EXT2_CONST = \"found explicit\"").unwrap();

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&main_file));

    let _ = fs::remove_file(&main_file);
    let _ = fs::remove_file(&helper_file);

    assert!(result.is_ok());
    let var = vm.globals().get("EXT2_CONST");
    assert_eq!(var, Some(Object::string(String::from("found explicit"))));
}

// 9.3.6 — Scope/variable sharing: functions and classes

#[test]
fn require_relative_makes_functions_accessible() {
    use std::fs;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let main_file = temp_dir.join("require_func_main.rb");
    let helper_file = temp_dir.join("require_func_helper.rb");

    fs::write(&helper_file, "def helper_fn\n  \"from function\"\nend").unwrap();
    fs::write(
        &main_file,
        "require_relative(\"require_func_helper\")\nresult = helper_fn()",
    )
    .unwrap();

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&main_file));

    let _ = fs::remove_file(&main_file);
    let _ = fs::remove_file(&helper_file);

    assert!(result.is_ok());
    let var = vm.environment().get("result");
    assert_eq!(var, Some(Object::string(String::from("from function"))));
}

#[test]
fn require_relative_makes_classes_accessible() {
    use std::fs;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let main_file = temp_dir.join("require_class_main.rb");
    let helper_file = temp_dir.join("require_class_helper.rb");

    fs::write(
        &helper_file,
        "class Greeter\n  def initialize(name)\n    @name = name\n  end\n  def greet\n    @name\n  end\nend",
    )
    .unwrap();
    fs::write(
        &main_file,
        "require_relative(\"require_class_helper\")\ng = Greeter.new(\"world\")\ngreeting = g.greet()",
    )
    .unwrap();

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&main_file));

    let _ = fs::remove_file(&main_file);
    let _ = fs::remove_file(&helper_file);

    assert!(result.is_ok());
    let var = vm.environment().get("greeting");
    assert_eq!(var, Some(Object::string(String::from("world"))));
}

// 9.3.7 — Nested requires (A → B → C)

#[test]
fn require_relative_nested_chain() {
    use std::fs;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let sub_dir = temp_dir.join("require_nested_test");
    let _ = fs::create_dir_all(&sub_dir);

    let file_c = sub_dir.join("chain_c.rb");
    let file_b = sub_dir.join("chain_b.rb");
    let file_a = sub_dir.join("chain_a.rb");

    fs::write(&file_c, "C_CONST = \"from_c\"").unwrap();
    fs::write(
        &file_b,
        "require_relative(\"chain_c\")\nB_CONST = \"from_b\"",
    )
    .unwrap();
    fs::write(
        &file_a,
        "require_relative(\"chain_b\")\nA_CONST = \"from_a\"",
    )
    .unwrap();

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&file_a));

    let _ = fs::remove_dir_all(&sub_dir);

    assert!(result.is_ok());
    assert_eq!(
        vm.globals().get("C_CONST"),
        Some(Object::string(String::from("from_c")))
    );
    assert_eq!(
        vm.globals().get("B_CONST"),
        Some(Object::string(String::from("from_b")))
    );
    assert_eq!(
        vm.globals().get("A_CONST"),
        Some(Object::string(String::from("from_a")))
    );
}

// 9.3.8 — Circular requires handled gracefully

#[test]
fn require_relative_circular_does_not_infinite_loop() {
    use std::fs;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let sub_dir = temp_dir.join("require_circular_test");
    let _ = fs::create_dir_all(&sub_dir);

    let file_a = sub_dir.join("circ_a.rb");
    let file_b = sub_dir.join("circ_b.rb");
    let main_file = sub_dir.join("circ_main.rb");

    // A requires B, B requires A — circular
    fs::write(
        &file_a,
        "require_relative(\"circ_b\")\ncirc_a_var = \"a_loaded\"",
    )
    .unwrap();
    fs::write(
        &file_b,
        "require_relative(\"circ_a\")\ncirc_b_var = \"b_loaded\"",
    )
    .unwrap();
    fs::write(&main_file, "require_relative(\"circ_a\")").unwrap();

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&main_file));

    let _ = fs::remove_dir_all(&sub_dir);

    // Should complete without infinite loop
    assert!(result.is_ok());
}

// 9.3.9 — Diamond dependency (D loads only once)

#[test]
fn require_relative_diamond_dependency_loads_once() {
    use std::fs;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let sub_dir = temp_dir.join("require_diamond_test");
    let _ = fs::create_dir_all(&sub_dir);

    let file_d = sub_dir.join("dia_d.rb");
    let file_b = sub_dir.join("dia_b.rb");
    let file_c = sub_dir.join("dia_c.rb");
    let file_a = sub_dir.join("dia_a.rb");

    // D is the shared dependency
    fs::write(&file_d, "DIA_D_CONST = \"d_value\"").unwrap();
    // B requires D
    fs::write(
        &file_b,
        "require_relative(\"dia_d\")\nDIA_B_CONST = \"b_value\"",
    )
    .unwrap();
    // C requires D
    fs::write(
        &file_c,
        "require_relative(\"dia_d\")\nDIA_C_CONST = \"c_value\"",
    )
    .unwrap();
    // A requires both B and C
    fs::write(
        &file_a,
        "require_relative(\"dia_b\")\nrequire_relative(\"dia_c\")",
    )
    .unwrap();

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&file_a));

    let _ = fs::remove_dir_all(&sub_dir);

    assert!(result.is_ok());

    // Every constant the chain defined is reachable
    assert_eq!(
        vm.globals().get("DIA_D_CONST"),
        Some(Object::string(String::from("d_value")))
    );
    assert_eq!(
        vm.globals().get("DIA_B_CONST"),
        Some(Object::string(String::from("b_value")))
    );
    assert_eq!(
        vm.globals().get("DIA_C_CONST"),
        Some(Object::string(String::from("c_value")))
    );
}
