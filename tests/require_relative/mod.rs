// Tests for require_relative native function

use metorex::object::Object;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

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
    let test_file = temp_dir.join("require_test_args.mx");

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
    let test_file = temp_dir.join("require_test_type.mx");

    // Test with integer
    fs::write(&test_file, "require_relative(42)").unwrap();
    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&test_file));
    let _ = fs::remove_file(&test_file);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("expects a String argument"));

    // Test with boolean
    fs::write(&test_file, "require_relative(true)").unwrap();
    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&test_file));
    let _ = fs::remove_file(&test_file);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("expects a String argument"));
}

#[test]
fn require_relative_with_invalid_path() {
    use std::fs;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("require_test_invalid.mx");

    fs::write(
        &test_file,
        "require_relative(\"nonexistent_file_12345.mx\")",
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
    let main_file = temp_dir.join("require_test_main.mx");
    let helper_file = temp_dir.join("require_test_helper.mx");

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
    let main_file = temp_dir.join("require_test_main2.mx");
    let helper_file = temp_dir.join("require_test_helper2.mx");

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
    let main_file = temp_dir.join("require_test_main3.mx");
    let helper_file = temp_dir.join("require_test_helper3.mx");

    fs::write(&main_file, "require_relative(\"require_test_helper3\")").unwrap();
    fs::write(&helper_file, "shared_var = \"from helper\"").unwrap();

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(Path::new(&main_file));

    // Clean up temp files
    let _ = fs::remove_file(&main_file);
    let _ = fs::remove_file(&helper_file);

    assert!(result.is_ok());

    // Check that the variable from the helper file is accessible
    let var = vm.environment().get("shared_var");
    assert_eq!(
        var,
        Some(Object::String(Rc::new(String::from("from helper"))))
    );
}
