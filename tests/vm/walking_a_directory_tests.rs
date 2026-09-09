// Coverage tests for an open Dir, ENV's own methods, and Encoding.find

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

/// A directory of its own holding two files and a subdirectory, named after
/// the test so two running at once cannot collide.
fn in_a_folder(name: &str, body: &str) -> Option<Object> {
    let folder = std::env::temp_dir().join(format!("metorex_walking_{name}"));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(folder.join("inside")).expect("could not make the folder");
    std::fs::write(folder.join("one.txt"), "1").expect("could not write");
    std::fs::write(folder.join("two.txt"), "2").expect("could not write");
    let path = folder.to_string_lossy().to_string();
    let answer = run(&format!("folder = {path:?}\n{body}"));
    let _ = std::fs::remove_dir_all(&folder);
    answer
}

// ── An open directory ──────────────────────────────────────────────────────

#[test]
fn an_open_directory_reports_the_path_it_was_opened_with() {
    assert_eq!(
        in_a_folder(
            "path",
            "held = Dir.open(folder)\nanswer = held.path == folder && held.to_path == folder\nheld.close\nanswer"
        ),
        Some(Object::Bool(true))
    );
    // The path stands after the directory is closed, which is what Ruby
    // answers for a closed one.
    assert_eq!(
        in_a_folder(
            "closed_path",
            "held = Dir.open(folder)\nheld.close\nheld.path == folder"
        ),
        Some(Object::Bool(true))
    );
    assert_eq!(
        in_a_folder(
            "inspect",
            "held = Dir.open(folder)\nanswer = held.inspect == \"#<Dir:\" + folder + \">\"\nheld.close\nanswer"
        ),
        Some(Object::Bool(true))
    );
    assert_eq!(run("Dir.include?(Enumerable)"), Some(Object::Bool(true)));
}

#[test]
fn reading_walks_the_names_one_at_a_time() {
    assert_eq!(
        in_a_folder(
            "read",
            r#"
held = Dir.open(folder)
first = held.read
answer = [held.pos, held.read.class.name, first.class.name]
held.close
answer.inspect
"#
        ),
        Some(Object::string("[1, \"String\", \"String\"]"))
    );
    assert_eq!(
        in_a_folder(
            "read_end",
            "held = Dir.open(folder)\n5.times { held.read }\nanswer = held.read\nheld.close\nanswer"
        ),
        Some(Object::Nil)
    );
}

#[test]
fn the_position_moves_and_can_be_moved_back() {
    assert_eq!(
        in_a_folder(
            "seek",
            r#"
held = Dir.open(folder)
mark = held.pos
before = held.read
held.seek(mark)
answer = [held.pos == mark + 1, held.read == before]
held.rewind
answer.push(held.pos)
held.close
answer.inspect
"#
        ),
        Some(Object::string("[false, true, 0]"))
    );
    assert_eq!(
        in_a_folder(
            "pos_set",
            "held = Dir.open(folder)\nheld.read\nheld.pos = 0\nanswer = held.tell\nheld.close\nanswer"
        ),
        Some(Object::Int(0))
    );
}

#[test]
fn walking_the_whole_directory_leaves_the_position_at_the_end() {
    assert_eq!(
        in_a_folder(
            "each",
            r#"
held = Dir.open(folder)
walked = []
answer = held.each { |name| walked.push(name) }.equal?(held)
[answer, held.read, walked.sort].inspect.tap { held.close }
"#
        ),
        Some(Object::string(
            "[true, nil, [\".\", \"..\", \"inside\", \"one.txt\", \"two.txt\"]]"
        ))
    );
    assert_eq!(
        in_a_folder(
            "each_child",
            "held = Dir.open(folder)\nwalked = []\nheld.each_child { |name| walked.push(name) }\nheld.close\nwalked.sort.inspect"
        ),
        Some(Object::string("[\"inside\", \"one.txt\", \"two.txt\"]"))
    );
}

#[test]
fn a_closed_directory_refuses_to_be_walked() {
    assert!(
        in_a_folder("closed", "held = Dir.open(folder)\nheld.close\nbegin\n  held.read\nrescue IOError => problem\n  problem.message\nend")
            .map(|held| format!("{held}"))
            .unwrap_or_default()
            .contains("closed directory")
    );
    assert_eq!(
        in_a_folder(
            "close_twice",
            "held = Dir.open(folder)\n[held.close, held.close].inspect"
        ),
        Some(Object::string("[nil, nil]"))
    );
    assert_eq!(
        in_a_folder(
            "closed_flag",
            "held = Dir.open(folder)\nheld.close\nheld.closed?"
        ),
        Some(Object::Bool(true))
    );
}

#[test]
fn a_directory_is_opened_from_anything_that_names_a_path() {
    assert_eq!(
        in_a_folder(
            "to_path",
            "named = Object.new\nnamed.define_singleton_method(:to_path) { folder }\nheld = Dir.new(named)\nanswer = held.path == folder\nheld.close\nanswer"
        ),
        Some(Object::Bool(true))
    );
    assert!(run_err("Dir.new(42)").contains("no implicit conversion"));
    assert!(run_err("Dir.new('no_such_folder_at_all')").contains("No such file or directory"));
}

#[test]
fn opening_with_a_block_closes_the_directory_afterwards() {
    assert_eq!(
        in_a_folder(
            "block",
            "held = nil\nDir.open(folder) { |one| held = one }\nheld.closed?"
        ),
        Some(Object::Bool(true))
    );
}

// ── ENV answers a few things its own way ───────────────────────────────────

#[test]
fn env_is_a_hash_the_process_shares_with_the_operating_system() {
    assert_eq!(run("ENV.to_s"), Some(Object::string("ENV")));
    assert_eq!(run("ENV.rehash"), Some(Object::Nil));
    assert!(
        run_err("ENV.dup").contains("Cannot dup ENV, use ENV.to_h to get a copy of ENV as a hash")
    );
    assert!(run_err("ENV.clone").contains("Cannot clone ENV"));
}

#[test]
fn converting_env_to_a_hash_makes_a_copy() {
    assert_eq!(
        run(
            "ENV['METOREX_ENV_TEST'] = 'x'\nheld = ENV.to_h\nENV.delete('METOREX_ENV_TEST')\nheld['METOREX_ENV_TEST']"
        ),
        Some(Object::string("x"))
    );
    assert_eq!(
        run("held = ENV.to_hash\nENV.clear\nanswer = held.size > 0\nENV.replace(held)\nanswer"),
        Some(Object::Bool(true))
    );
}

#[test]
fn setting_an_env_name_to_nil_removes_it() {
    assert_eq!(
        run(
            "ENV['METOREX_GONE'] = 'x'\nbefore = ENV.include?('METOREX_GONE')\nENV['METOREX_GONE'] = nil\n[before, ENV.include?('METOREX_GONE')].inspect"
        ),
        Some(Object::string("[true, false]"))
    );
}

// ── Encodings by name ──────────────────────────────────────────────────────

#[test]
fn an_encoding_is_found_by_the_name_it_goes_by() {
    assert_eq!(
        run("Encoding.find('UTF-8').to_s"),
        Some(Object::string("UTF-8"))
    );
    assert_eq!(
        run("Encoding.find('locale').to_s"),
        Some(Object::string("UTF-8"))
    );
    assert_eq!(
        run("Encoding.find('filesystem').to_s"),
        Some(Object::string("UTF-8"))
    );
    assert_eq!(run("Encoding.find('internal')"), Some(Object::Nil));
    assert_eq!(run("Encoding.default_internal"), Some(Object::Nil));
    // BINARY is a second name for ASCII_8BIT, and the encoding reports the
    // one name whichever constant reached it.
    assert_eq!(
        run("Encoding.find('binary').to_s"),
        Some(Object::string("ASCII-8BIT"))
    );
    assert!(run_err("Encoding.find('nowhere')").contains("unknown encoding name"));
}

#[test]
fn force_encoding_leaves_the_string_as_it_stands() {
    assert_eq!(
        run("'held'.force_encoding(Encoding::BINARY)"),
        Some(Object::string("held"))
    );
    assert_eq!(
        run("'held'.force_encoding('UTF-8').encoding.to_s"),
        Some(Object::string("UTF-8"))
    );
}

// ── Percent lists ──────────────────────────────────────────────────────────

#[test]
fn an_escaped_space_keeps_a_percent_list_word_whole() {
    assert_eq!(
        run(r"%w[a\ b c].inspect"),
        Some(Object::string("[\"a b\", \"c\"]"))
    );
    assert_eq!(
        run(r"%w[one two].inspect"),
        Some(Object::string("[\"one\", \"two\"]"))
    );
    assert_eq!(
        run(r"%i[a\ b c].inspect"),
        Some(Object::string("[:\"a b\", :c]"))
    );
}
