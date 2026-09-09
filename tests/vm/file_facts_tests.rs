// Coverage tests for File::Stat and the questions File asks about a file

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

/// A directory of its own for the tests that make and unmake files, named
/// after the test so two running at once cannot collide.
fn in_a_folder(name: &str, body: &str) -> Option<Object> {
    let folder = std::env::temp_dir().join(format!("metorex_file_facts_{name}"));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).expect("could not make the folder");
    let path = folder.to_string_lossy().to_string();
    let answer = run(&format!("folder = {path:?}\n{body}"));
    let _ = std::fs::remove_dir_all(&folder);
    answer
}

// ── Reading what the operating system keeps ────────────────────────────────

#[test]
fn a_stat_reports_the_kind_of_thing_the_path_names() {
    assert_eq!(
        run("File.stat('README.md').class.name"),
        Some(Object::string("File::Stat"))
    );
    assert_eq!(
        run("File.stat('README.md').ftype"),
        Some(Object::string("file"))
    );
    assert_eq!(
        run("File.stat('README.md').file?"),
        Some(Object::Bool(true))
    );
    assert_eq!(run("File.stat('src').directory?"), Some(Object::Bool(true)));
    assert_eq!(
        run("File.stat('README.md').symlink?"),
        Some(Object::Bool(false))
    );
    assert_eq!(run("File.ftype('src')"), Some(Object::string("directory")));
}

#[test]
fn a_stat_reports_the_numbers_a_file_is_kept_under() {
    assert_eq!(
        run("File.stat('README.md').size == File.size('README.md')"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("File.stat('README.md').uid == Process.uid"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("File.stat('README.md').gid == Process.gid"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("File.stat('README.md').owned?"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("File.stat('README.md').nlink >= 1"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("File.stat('README.md').blksize > 0"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("File.stat('README.md').ino > 0"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("File.stat('README.md').dev_major == ((File.stat('README.md').dev >> 24) & 0xff)"),
        Some(Object::Bool(true))
    );
}

#[test]
fn the_times_a_stat_reports_are_times() {
    assert_eq!(
        run("File.stat('README.md').atime.class.name"),
        Some(Object::string("Time"))
    );
    assert_eq!(
        run("File.stat('README.md').mtime.class.name"),
        Some(Object::string("Time"))
    );
    assert_eq!(
        run("File.stat('README.md').ctime.class.name"),
        Some(Object::string("Time"))
    );
}

#[test]
fn a_stat_reads_the_permission_bits() {
    assert_eq!(
        run("(File.stat('README.md').mode & 0777).to_s(8)"),
        Some(Object::string("644"))
    );
    assert_eq!(
        run("File.stat('README.md').readable?"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("File.stat('README.md').world_writable?"),
        Some(Object::Nil)
    );
    assert_eq!(
        run("File.stat('README.md').setuid?"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run("File.stat('README.md').sticky?"),
        Some(Object::Bool(false))
    );
}

#[test]
fn a_path_with_nothing_behind_it_is_refused_or_answered_plainly() {
    assert!(run_err("File.stat('no_such_file_at_all')").contains("No such file"));
    assert_eq!(
        run("File.zero?('no_such_file_at_all')"),
        Some(Object::Bool(false))
    );
    assert_eq!(run("File.size?('no_such_file_at_all')"), Some(Object::Nil));
    assert_eq!(run("File.zero?('README.md')"), Some(Object::Bool(false)));
}

#[test]
fn a_stat_is_made_from_anything_that_names_a_path() {
    assert_eq!(
        run("File::Stat.new('README.md').ftype"),
        Some(Object::string("file"))
    );
    assert_eq!(
        run("held = Object.new\ndef held.to_path\n  'README.md'\nend\nFile::Stat.new(held).ftype"),
        Some(Object::string("file"))
    );
    assert!(run_err("File::Stat.new(42)").contains("no implicit conversion"));
}

// ── Links ──────────────────────────────────────────────────────────────────

#[test]
fn a_symbolic_link_points_at_another_name() {
    let answer = in_a_folder(
        "symlink",
        r#"
target = folder + "/target.txt"
link = folder + "/link.txt"
File.write(target, "hello")
File.symlink(target, link)
[File.symlink?(link), File.symlink?(target), File.readlink(link) == target,
 File.lstat(link).ftype, File.stat(link).ftype, File.stat(link).size,
 File.identical?(link, target)].inspect
"#,
    );
    assert_eq!(
        answer,
        Some(Object::string(
            "[true, false, true, \"link\", \"file\", 5, true]"
        ))
    );
}

#[test]
fn a_hard_link_is_another_name_for_the_same_file() {
    let answer = in_a_folder(
        "link",
        r#"
target = folder + "/target.txt"
copy = folder + "/copy.txt"
File.write(target, "hello")
File.link(target, copy)
[File.stat(copy).nlink, File.identical?(copy, target),
 File.stat(copy).ino == File.stat(target).ino].inspect
"#,
    );
    assert_eq!(answer, Some(Object::string("[2, true, true]")));
}

#[test]
fn utime_sets_the_times_a_file_reports() {
    let answer = in_a_folder(
        "utime",
        r#"
target = folder + "/target.txt"
File.write(target, "hello")
File.utime(1000000000, 1200000000, target)
[File.stat(target).atime.to_i, File.stat(target).mtime.to_i].inspect
"#,
    );
    assert_eq!(answer, Some(Object::string("[1000000000, 1200000000]")));
}

#[test]
fn two_stats_compare_by_the_time_the_files_were_last_written() {
    let answer = in_a_folder(
        "compare",
        r#"
older = folder + "/older.txt"
newer = folder + "/newer.txt"
File.write(older, "a")
File.write(newer, "b")
File.utime(1000000000, 1000000000, older)
File.utime(1000000000, 1200000000, newer)
[File.stat(older) <=> File.stat(newer), File.stat(newer) <=> File.stat(older),
 File.stat(older) <=> File.stat(older), File.stat(older) <=> 42].inspect
"#,
    );
    assert_eq!(answer, Some(Object::string("[-1, 1, 0, nil]")));
}

// ── The name a call was made with ──────────────────────────────────────────

#[test]
fn method_missing_is_handed_the_name_and_the_arguments_the_way_ruby_hands_them() {
    assert_eq!(
        run(
            "class K\n  def method_missing(name, *args)\n    [name, args]\n  end\nend\nK.new.thing(1, 2).inspect"
        ),
        Some(Object::string("[:thing, [1, 2]]"))
    );
    assert_eq!(
        run("class K\n  def method_missing(name, one)\n    one\n  end\nend\nK.new.thing('x')"),
        Some(Object::string("x"))
    );
    assert!(
        run_err("class K\n  def method_missing(name)\n    name\n  end\nend\nK.new.thing(1)")
            .contains("wrong number of arguments")
    );
}
