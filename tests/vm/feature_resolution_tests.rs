// What `$LOAD_PATH.resolve_feature_path` reports a `require` would load.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&statements).expect("execution failed")
}

/// A directory of its own under the OS temp dir, holding the named files.
fn directory_with(label: &str, files: &[&str]) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "metorex_feature_resolution_{}_{}",
        label,
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("create the directory");
    for file in files {
        std::fs::write(directory.join(file), "").expect("write the file");
    }
    directory.canonicalize().expect("a real directory")
}

fn answer(kind: &str, path: &std::path::Path) -> Option<Object> {
    Some(Object::array(vec![
        Object::symbol(kind.to_string()),
        Object::string(path.to_string_lossy().into_owned()),
    ]))
}

#[test]
fn a_ruby_file_on_the_load_path_is_found_as_rb() {
    let directory = directory_with("rb", &["widget.rb"]);
    let code = format!(
        "$LOAD_PATH.unshift({:?})\n$LOAD_PATH.resolve_feature_path('widget')",
        directory.to_string_lossy()
    );
    assert_eq!(run(&code), answer("rb", &directory.join("widget.rb")));
}

#[test]
fn a_native_extension_on_the_load_path_is_found_as_so() {
    let directory = directory_with("so", &["gadget.bundle"]);
    let code = format!(
        "$LOAD_PATH.unshift({:?})\n$LOAD_PATH.resolve_feature_path('gadget')",
        directory.to_string_lossy()
    );
    assert_eq!(run(&code), answer("so", &directory.join("gadget.bundle")));
}

#[test]
fn a_path_named_outright_is_not_searched_for() {
    let directory = directory_with("outright", &["direct.rb"]);
    let file = directory.join("direct.rb");
    let code = format!(
        "$LOAD_PATH.resolve_feature_path({:?})",
        file.to_string_lossy()
    );
    assert_eq!(run(&code), answer("rb", &file));
}

#[test]
fn a_library_metorex_carries_is_found_inside_the_interpreter() {
    assert_eq!(
        run("[$LOAD_PATH.resolve_feature_path('pp'), $LOAD_PATH.resolve_feature_path('set.rb')]"),
        Some(Object::array(vec![
            Object::array(vec![
                Object::symbol("rb".to_string()),
                Object::string("<metorex>/pp.rb")
            ]),
            Object::array(vec![
                Object::symbol("rb".to_string()),
                Object::string("<metorex>/set.rb")
            ]),
        ]))
    );
}

#[test]
fn a_feature_that_cannot_be_found_answers_nil() {
    assert_eq!(
        run("$LOAD_PATH.resolve_feature_path('no_such_feature_anywhere')"),
        Some(Object::Nil)
    );
}

#[test]
fn a_name_that_is_not_a_path_is_refused() {
    assert_eq!(
        run(
            "begin\n  $LOAD_PATH.resolve_feature_path(1)\nrescue TypeError => error\n  error.class.name\nend"
        ),
        Some(Object::string("TypeError"))
    );
}
