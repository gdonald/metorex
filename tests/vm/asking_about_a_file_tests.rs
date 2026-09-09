// Coverage tests for the FileTest module, paths named through `to_path`,
// the groups a process belongs to, and the platform Ruby reports.

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

fn text(value: &str) -> Option<Object> {
    Some(Object::string(value))
}

fn yes() -> Option<Object> {
    Some(Object::Bool(true))
}

fn no() -> Option<Object> {
    Some(Object::Bool(false))
}

/// A file of its own holding a line, named after the test so two running at
/// once cannot collide.
fn with_a_file(name: &str, body: &str) -> Option<Object> {
    let path = std::env::temp_dir().join(format!("metorex_filetest_{name}.txt"));
    std::fs::write(&path, "one\n").expect("could not write");
    let named = path.to_string_lossy().to_string();
    let answer = run(&format!("path = {named:?}\n{body}"));
    let _ = std::fs::remove_file(&path);
    answer
}

// ── The questions FileTest asks ────────────────────────────────────────────

#[test]
fn file_test_answers_what_file_answers() {
    assert_eq!(with_a_file("exist", "FileTest.exist?(path)"), yes());
    assert_eq!(with_a_file("file", "FileTest.file?(path)"), yes());
    assert_eq!(with_a_file("directory", "FileTest.directory?(path)"), no());
    assert_eq!(with_a_file("readable", "FileTest.readable?(path)"), yes());
    assert_eq!(with_a_file("writable", "FileTest.writable?(path)"), yes());
    assert_eq!(with_a_file("zero", "FileTest.zero?(path)"), no());
    assert_eq!(with_a_file("empty", "FileTest.empty?(path)"), no());
    assert_eq!(with_a_file("symlink", "FileTest.symlink?(path)"), no());
    assert_eq!(with_a_file("pipe", "FileTest.pipe?(path)"), no());
    assert_eq!(with_a_file("socket", "FileTest.socket?(path)"), no());
    assert_eq!(with_a_file("blockdev", "FileTest.blockdev?(path)"), no());
    assert_eq!(with_a_file("chardev", "FileTest.chardev?(path)"), no());
    assert_eq!(with_a_file("setuid", "FileTest.setuid?(path)"), no());
    assert_eq!(with_a_file("setgid", "FileTest.setgid?(path)"), no());
    assert_eq!(with_a_file("sticky", "FileTest.sticky?(path)"), no());
    assert_eq!(with_a_file("owned", "FileTest.owned?(path)"), yes());
    assert_eq!(with_a_file("grpowned", "FileTest.grpowned?(path)"), yes());
    assert_eq!(
        with_a_file("size", "FileTest.size(path)"),
        Some(Object::Int(4))
    );
    assert_eq!(
        with_a_file("size_maybe", "FileTest.size?(path)"),
        Some(Object::Int(4))
    );
    assert_eq!(
        with_a_file("identical", "FileTest.identical?(path, path)"),
        yes()
    );
    assert_eq!(
        with_a_file(
            "real",
            "FileTest.readable_real?(path) && FileTest.writable_real?(path)"
        ),
        yes()
    );
    assert_eq!(
        with_a_file(
            "executable",
            "FileTest.executable?(path) || FileTest.executable_real?(path)"
        ),
        no()
    );
    assert_eq!(
        with_a_file("world", "FileTest.world_writable?(path)"),
        Some(Object::Nil)
    );
    assert_eq!(
        with_a_file(
            "world_read",
            "FileTest.world_readable?(path) == File.world_readable?(path)"
        ),
        yes()
    );
    assert_eq!(run("FileTest.exist?(\"/no/such/place\")"), no());
}

// ── A path named by something other than a String ──────────────────────────

#[test]
fn a_path_may_be_named_by_anything_that_answers_to_path() {
    let named = "class NamedPath\n  def initialize(path)\n    @path = path\n  end\n  def to_path\n    @path\n  end\nend\n";
    assert_eq!(
        with_a_file(
            "to_path",
            &format!("{named}File.exist?(NamedPath.new(path))")
        ),
        yes()
    );
    assert_eq!(
        with_a_file(
            "to_path_file",
            &format!("{named}File.file?(NamedPath.new(path))")
        ),
        yes()
    );
    assert_eq!(
        with_a_file(
            "to_path_size",
            &format!("{named}File.size(NamedPath.new(path))")
        ),
        Some(Object::Int(4))
    );
    assert_eq!(
        with_a_file(
            "to_path_test",
            &format!("{named}FileTest.symlink?(NamedPath.new(path))")
        ),
        no()
    );
}

#[test]
fn a_path_named_through_to_path_is_asked_for_it_once() {
    let counting = "class CountedPath\n  attr_reader :asked\n  def initialize(path)\n    @path = path\n    @asked = 0\n  end\n  def to_path\n    @asked = @asked + 1\n    @path\n  end\nend\n";
    assert_eq!(
        with_a_file(
            "counted",
            &format!("{counting}held = CountedPath.new(path)\nFile.exist?(held)\nheld.asked")
        ),
        Some(Object::Int(1))
    );
    assert_eq!(
        with_a_file(
            "counted_test",
            &format!(
                "{counting}held = CountedPath.new(path)\nFileTest.blockdev?(held)\nheld.asked"
            )
        ),
        Some(Object::Int(1))
    );
}

// ── What the operating system says about this process ──────────────────────

#[test]
fn the_groups_a_process_belongs_to_are_reported() {
    assert_eq!(run("Process.groups.is_a?(Array)"), yes());
    assert_eq!(
        run("Process.groups.all? { |group| group.is_a?(Integer) }"),
        yes()
    );
}

#[test]
fn the_platform_is_named_the_way_ruby_names_it() {
    assert_eq!(run("RUBY_PLATFORM.include?(\"-\")"), yes());
    assert_eq!(
        run("RUBY_PLATFORM.split(\"-\").length"),
        Some(Object::Int(2))
    );
    assert_eq!(run("RUBY_PLATFORM == \"macos\""), no());
}

// ── When a file was made ───────────────────────────────────────────────────

#[test]
fn a_file_reports_when_it_was_made() {
    assert_eq!(
        with_a_file("birthtime", "File.birthtime(path).class.name"),
        text("Time")
    );
    assert_eq!(
        with_a_file(
            "birthtime_stat",
            "File.birthtime(path) == File.stat(path).birthtime"
        ),
        yes()
    );
}
