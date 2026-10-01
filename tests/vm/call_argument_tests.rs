// Handing a call its arguments and its block, and the warning about a block
// the method never uses.

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

/// Run `code` with `$stderr` gathered, and answer what was written there.
fn warnings_from(code: &str) -> String {
    let wrapped = format!(
        "gathered = []\ncollector = Object.new\ncollector.define_singleton_method(:write) {{ |text| gathered << text }}\n$stderr = collector\n{code}\n$stderr = STDERR\ngathered.join"
    );
    match run(&wrapped) {
        Some(Object::String(text)) => text.as_str().to_string(),
        other => panic!("expected the warnings, got {other:?}"),
    }
}

#[test]
fn a_verbose_run_warns_about_a_block_the_method_never_uses() {
    let written = warnings_from("$VERBOSE = true\ndef ignores\n  42\nend\nignores { }");
    assert!(written.contains("warning: the block passed to 'Object#ignores' defined at"));
}

#[test]
fn a_method_warns_once_about_an_ignored_block() {
    let written = warnings_from("$VERBOSE = true\ndef ignores\nend\nignores { }\nignores { }");
    assert_eq!(written.matches("may be ignored").count(), 1);
}

#[test]
fn a_run_that_is_not_verbose_warns_only_when_the_category_is_on() {
    let quiet = warnings_from("def ignores\nend\nignores { }");
    let asked =
        warnings_from("Warning[:strict_unused_block] = true\ndef ignores\nend\nignores { }");
    assert_eq!(
        (quiet.is_empty(), asked.contains("may be ignored")),
        (true, true)
    );
}

#[test]
fn turning_the_category_off_silences_a_verbose_run() {
    let written = warnings_from(
        "$VERBOSE = true\nWarning[:strict_unused_block] = false\ndef ignores\nend\nignores { }",
    );
    assert!(written.is_empty());
}

#[test]
fn a_method_that_uses_its_block_is_not_warned_about() {
    let written = warnings_from(
        "$VERBOSE = true\nclass Parent\n  def hand_on; end\nend\nclass Child < Parent\n  def yields\n    yield\n  end\n  def named(&block); end\n  def hand_on\n    super\n  end\n  def initialize; end\nend\nChild.new { }\nheld = Child.new\nheld.yields { }\nheld.named { }\nheld.hand_on { }\nblock = proc { }\ndef ignores; end\nignores(&block)",
    );
    assert!(written.is_empty(), "warned: {written}");
}

#[test]
fn a_singleton_method_of_a_named_class_is_named_by_the_class() {
    let written =
        warnings_from("$VERBOSE = true\nclass Named\n  def self.build; end\nend\nNamed.build { }");
    assert!(written.contains("'Named.build'"));
}

#[test]
fn a_splatted_object_whose_to_a_answers_nil_is_passed_whole() {
    let code = "held = Object.new\ndef held.to_a\n  nil\nend\ndef gather(*values)\n  values\nend\ngather(*held).first.equal?(held)";
    assert_eq!(run(code), Some(Object::Bool(true)));
}

#[test]
fn a_writer_symbol_is_not_read_as_assigning_a_local() {
    let code = "def m(*args)\n  args\nend\nsymbols = [:m=, 1]\nm (1), (2)";
    assert_eq!(
        run(code),
        Some(Object::array(vec![Object::Int(1), Object::Int(2)]))
    );
}

#[test]
fn a_method_defined_on_a_receiver_is_not_read_as_a_local() {
    let code = "held = Object.new\ndef held.value=(given); end\ndef self.value; end\ndef gather(*args)\n  args\nend\ngather (3), (4)";
    assert_eq!(
        run(code),
        Some(Object::array(vec![Object::Int(3), Object::Int(4)]))
    );
}

#[test]
fn a_keyword_shorthand_at_the_end_of_a_line_reads_the_local() {
    let code = "def pair(first:, second:)\n  [first, second]\nend\nfirst = 1\nsecond = 2\nanswer = pair first:, second:\nanswer";
    assert_eq!(
        run(code),
        Some(Object::array(vec![Object::Int(1), Object::Int(2)]))
    );
}
