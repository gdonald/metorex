// Coverage tests for Pathname

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

/// Loading and running the Pathname library nests deeper than the stack a
/// test thread is given, so each program runs on a thread sized like the one
/// the binary itself uses.
fn on_a_deep_stack(work: impl FnOnce() -> String + Send + 'static) -> String {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(work)
        .expect("thread failed")
        .join()
        .expect("thread panicked")
}

/// The value a program answers, written the way `inspect` writes it.
fn shown(code: &str) -> String {
    let held = format!("__answered__ = begin\n{code}\nend\n__answered__.inspect");
    on_a_deep_stack(move || {
        let tokens = Lexer::new(&held).tokenize();
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        let mut vm = VirtualMachine::new();
        match vm.execute_program(&stmts).expect("execution failed") {
            Some(Object::String(written)) => written.to_string(),
            other => panic!("expected a string, got {other:?}"),
        }
    })
}

/// The text a program answers, with the quotes `inspect` puts around a
/// string taken back off.
fn quoted(code: &str) -> String {
    let written = shown(code);
    written
        .strip_prefix('"')
        .and_then(|held| held.strip_suffix('"'))
        .expect("expected a string")
        .to_string()
}

fn run_err(code: &str) -> String {
    let held = code.to_string();
    on_a_deep_stack(move || {
        let tokens = Lexer::new(&held).tokenize();
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        let mut vm = VirtualMachine::new();
        vm.execute_program(&stmts).unwrap_err().to_string()
    })
}

const HERE: &str = "require 'pathname'\nhere = Pathname.new('/usr/local/bin')\n";

// ── Making one ─────────────────────────────────────────────────────────────

#[test]
fn a_pathname_is_made_from_a_string_or_something_that_names_a_path() {
    assert_eq!(quoted(&format!("{HERE}here.to_s")), "/usr/local/bin");
    assert_eq!(
        quoted(&format!("{HERE}here.inspect")),
        "#<Pathname:/usr/local/bin>"
    );
    assert_eq!(
        shown("require 'pathname'\nPathname('/tmp').class"),
        "Pathname"
    );
    assert_eq!(
        shown("require 'pathname'\nheld = Pathname.new('/tmp')\nPathname(held).equal?(held)"),
        "true"
    );
    assert!(run_err("require 'pathname'\nPathname.new(nil)").contains("TypeError"));
    assert!(run_err("require 'pathname'\nPathname.new(\"a\\0b\")").contains("null byte"));
}

#[test]
fn two_pathnames_naming_the_same_path_are_equal() {
    assert_eq!(
        shown("require 'pathname'\nPathname.new('/a') == Pathname.new('/a')"),
        "true"
    );
    assert_eq!(
        shown("require 'pathname'\nPathname.new('/a') == Pathname.new('/b')"),
        "false"
    );
    assert_eq!(
        shown("require 'pathname'\nPathname.new('/a') == '/a'"),
        "false"
    );
    assert_eq!(
        shown("require 'pathname'\nPathname.new('/a').hash == '/a'.hash"),
        "true"
    );
    assert_eq!(
        shown(
            "require 'pathname'\n[Pathname.new('/b'), Pathname.new('/a')].sort.map { |one| one.to_s }"
        ),
        "[\"/a\", \"/b\"]"
    );
}

// ── Reading the shape of a path ────────────────────────────────────────────

#[test]
fn a_path_says_whether_it_starts_from_the_root() {
    assert_eq!(shown(&format!("{HERE}here.absolute?")), "true");
    assert_eq!(shown(&format!("{HERE}here.relative?")), "false");
    assert_eq!(
        shown("require 'pathname'\nPathname.new('fish/dog').relative?"),
        "true"
    );
    assert_eq!(shown("require 'pathname'\nPathname.new('/').root?"), "true");
    assert_eq!(shown("require 'pathname'\nPathname.new('').root?"), "false");
    assert_eq!(
        shown("require 'pathname'\nPathname.new('/usr/..').root?"),
        "false"
    );
}

#[test]
fn the_parts_of_a_path_read_back_off_it() {
    assert_eq!(quoted(&format!("{HERE}here.parent.to_s")), "/usr/local");
    assert_eq!(quoted(&format!("{HERE}here.basename.to_s")), "bin");
    assert_eq!(quoted(&format!("{HERE}here.dirname.to_s")), "/usr/local");
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/a/lib.tar.gz').extname"),
        ".gz"
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/a/lib').extname"),
        ""
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/a/lib.rb').basename('.rb').to_s"),
        "lib"
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/').parent.to_s"),
        "/"
    );
    assert_eq!(
        shown(&format!("{HERE}here.each_filename.to_a")),
        "[\"usr\", \"local\", \"bin\"]"
    );
}

// ── Joining ────────────────────────────────────────────────────────────────

#[test]
fn joining_lays_one_path_after_another() {
    assert_eq!(
        quoted("require 'pathname'\n(Pathname.new('/usr') + 'bin/ruby').to_s"),
        "/usr/bin/ruby"
    );
    assert_eq!(
        quoted("require 'pathname'\n(Pathname.new('/usr') / 'bin').to_s"),
        "/usr/bin"
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/usr').join('local', 'bin').to_s"),
        "/usr/local/bin"
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/usr').join('/etc').to_s"),
        "/etc"
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/usr/local').join('../share').to_s"),
        "/usr/share"
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('.').join('./foo', 'bar').to_s"),
        "foo/bar"
    );
}

#[test]
fn cleanpath_works_out_the_dot_segments_a_path_carries() {
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/a/b/../c/./d').cleanpath.to_s"),
        "/a/c/d"
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('a/./b').cleanpath.to_s"),
        "a/b"
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('.').cleanpath.to_s"),
        "."
    );
}

#[test]
fn relative_path_from_gives_the_way_from_one_path_to_another() {
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/usr/bin/ls').relative_path_from('/usr').to_s"),
        "bin/ls"
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/usr').relative_path_from('/').to_s"),
        "usr"
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('a').relative_path_from('b').to_s"),
        "../a"
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/usr').relative_path_from('/usr').to_s"),
        "."
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/usr').relative_path_from('/stuff/..').to_s"),
        "usr"
    );
    assert!(
        run_err("require 'pathname'\nPathname.new('/usr').relative_path_from('foo')")
            .contains("different prefix")
    );
    assert!(
        run_err("require 'pathname'\nPathname.new('a').relative_path_from('..')")
            .contains("may not contain")
    );
}

#[test]
fn descend_and_ascend_walk_the_path_a_part_at_a_time() {
    assert_eq!(
        shown("require 'pathname'\nPathname.new('/usr/local').descend.map { |one| one.to_s }"),
        "[\"/\", \"/usr\", \"/usr/local\"]"
    );
    assert_eq!(
        shown("require 'pathname'\nPathname.new('/usr/local').ascend.map { |one| one.to_s }"),
        "[\"/usr/local\", \"/usr\", \"/\"]"
    );
}

#[test]
fn sub_replaces_part_of_the_path_and_answers_a_new_one() {
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/usr/local/bin/').sub(/local/, 'fish').to_s"),
        "/usr/fish/bin/"
    );
    assert_eq!(
        quoted("require 'pathname'\nPathname.new('/a/lib.rb').sub_ext('.so').to_s"),
        "/a/lib.so"
    );
}

// ── Reaching the filesystem ────────────────────────────────────────────────

#[test]
fn the_questions_that_need_the_filesystem_are_asked_of_it() {
    assert_eq!(
        shown("require 'pathname'\nPathname.new('README.md').exist?"),
        "true"
    );
    assert_eq!(
        shown("require 'pathname'\nPathname.new('README.md').file?"),
        "true"
    );
    assert_eq!(
        shown("require 'pathname'\nPathname.new('src').directory?"),
        "true"
    );
    assert_eq!(
        shown("require 'pathname'\nPathname.new('no_such_file_here').exist?"),
        "false"
    );
    assert_eq!(
        shown("require 'pathname'\nPathname.new('README.md').size > 0"),
        "true"
    );
    assert_eq!(
        shown("require 'pathname'\nPathname.new('README.md').empty?"),
        "false"
    );
    assert_eq!(shown("require 'pathname'\nPathname.pwd.class"), "Pathname");
    assert_eq!(
        shown(
            "require 'pathname'\nPathname.new('src').children.all? { |one| one.to_s.start_with?('src/') }"
        ),
        "true"
    );
}
