// Coverage tests for the etc library, the coverage library, and a paren-less
// call carrying a `key => value` pair.

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

/// One expression, with the etc library loaded first.
fn with_etc(body: &str) -> Option<Object> {
    run(&format!("require 'etc'\n{body}"))
}

fn text(value: &str) -> Option<Object> {
    Some(Object::string(value))
}

fn yes() -> Option<Object> {
    Some(Object::Bool(true))
}

// ── The password and group databases ───────────────────────────────────────

#[test]
fn the_password_database_answers_for_this_account() {
    assert_eq!(with_etc("Etc.getpwuid.class.name"), text("Etc::Passwd"));
    assert_eq!(with_etc("Etc.getpwuid.uid == Process.uid"), yes());
    assert_eq!(
        with_etc("Etc.getpwuid(Process.uid).gid == Process.gid"),
        yes()
    );
    assert_eq!(
        with_etc("held = Etc.getpwuid\nEtc.getpwnam(held.name).uid == held.uid"),
        yes()
    );
    assert_eq!(
        with_etc("Etc.getpwuid.members.inspect"),
        text("[:name, :passwd, :uid, :gid, :gecos, :dir, :shell]")
    );
}

#[test]
fn the_group_database_answers_for_this_group() {
    assert_eq!(with_etc("Etc.getgrgid.class.name"), text("Etc::Group"));
    assert_eq!(with_etc("Etc.getgrgid.gid == Process.gid"), yes());
    assert_eq!(with_etc("Etc.getgrgid.mem.is_a?(Array)"), yes());
    assert_eq!(
        with_etc("held = Etc.getgrgid\nEtc.getgrnam(held.name).gid == held.gid"),
        yes()
    );
}

#[test]
fn a_lookup_refuses_an_argument_of_the_wrong_kind() {
    assert_eq!(
        with_etc("begin\n  Etc.getpwuid('me')\nrescue TypeError => error\n  error.message\nend"),
        text("no implicit conversion of String into Integer")
    );
    assert_eq!(
        with_etc("begin\n  Etc.getgrnam(1)\nrescue TypeError => error\n  error.message\nend"),
        text("no implicit conversion of Integer into String")
    );
    assert_eq!(
        with_etc(
            "begin\n  Etc.getgrgid(987654)\nrescue ArgumentError => error\n  error.class.name\nend"
        ),
        text("ArgumentError")
    );
}

#[test]
fn a_walk_through_a_database_is_refused_inside_another() {
    assert_eq!(
        with_etc("counted = 0\nEtc.passwd { |entry| counted += 1 }\ncounted > 0"),
        yes()
    );
    assert_eq!(
        with_etc(
            "begin\n  Etc.group { |outer| Etc.group { |inner| } }\nrescue RuntimeError => error\n  error.class.name\nend"
        ),
        text("RuntimeError")
    );
    assert_eq!(with_etc("Etc.endpwent"), Some(Object::Nil));
    assert_eq!(with_etc("Etc.endgrent"), Some(Object::Nil));
}

// ── What the system says about itself ──────────────────────────────────────

#[test]
fn the_system_reports_its_own_settings() {
    assert_eq!(
        with_etc("Etc.uname.keys.inspect"),
        text("[:sysname, :nodename, :release, :version, :machine]")
    );
    assert_eq!(with_etc("Etc.nprocessors >= 1"), yes());
    assert_eq!(
        with_etc("Etc.sysconf(Etc::SC_OPEN_MAX).is_a?(Integer)"),
        yes()
    );
    assert_eq!(with_etc("Etc.sysconf(-1)"), Some(Object::Nil));
    assert_eq!(with_etc("Etc.confstr(Etc::CS_PATH).is_a?(String)"), yes());
    assert_eq!(
        with_etc(
            "begin\n  Etc.confstr(-1)\nrescue Errno::EINVAL => error\n  error.class.name\nend"
        ),
        text("Errno::EINVAL")
    );
    assert_eq!(with_etc("Etc.sysconfdir"), text("/etc"));
    assert_eq!(with_etc("Etc.systmpdir.is_a?(String)"), yes());
}

// ── Coverage measurement ───────────────────────────────────────────────────

#[test]
fn coverage_reports_whether_it_is_running() {
    let with_coverage = |body: &str| run(&format!("require 'coverage'\n{body}"));
    assert_eq!(
        with_coverage("Coverage.running?"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        with_coverage("Coverage.start\nCoverage.running?"),
        Some(Object::Bool(true))
    );
    assert_eq!(with_coverage("Coverage.start"), Some(Object::Nil));
    assert_eq!(
        with_coverage("Coverage.start\nCoverage.result\nCoverage.running?"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        with_coverage(
            "Coverage.start\nbegin\n  Coverage.start\nrescue RuntimeError => error\n  error.message\nend"
        ),
        text("coverage measurement is already setup")
    );
    assert_eq!(
        with_coverage(
            "begin\n  Coverage.supported?('lines')\nrescue TypeError => error\n  error.message\nend"
        ),
        text("wrong argument type String (expected Symbol)")
    );
    assert_eq!(
        with_coverage("Coverage.supported?(:lines)"),
        Some(Object::Bool(false))
    );
}

// ── A pair written without parentheses ─────────────────────────────────────

#[test]
fn a_call_without_parentheses_carries_a_rocket_pair() {
    assert_eq!(
        run("def show(*args)\n  args.inspect\nend\nshow \"a\" => 1"),
        text("[{\"a\" => 1}]")
    );
    assert_eq!(
        run("def show(*args)\n  args.inspect\nend\nshow 1, \"b\" => 2"),
        text("[1, {\"b\" => 2}]")
    );
    assert_eq!(
        run("def show(*args)\n  args.inspect\nend\nshow Object.new.class => \"x\""),
        text("[{Object => \"x\"}]")
    );
    assert_eq!(
        run("held = {}\nheld.update \"k\" => \"v\"\nheld.inspect"),
        text("{\"k\" => \"v\"}")
    );
}

// ── A pack format nobody knows ─────────────────────────────────────────────

#[test]
fn an_unknown_pack_directive_is_an_argument_error() {
    assert_eq!(
        run("begin\n  [1].pack('%')\nrescue ArgumentError => error\n  error.message\nend"),
        text("unknown pack directive '%' in '%'")
    );
}
