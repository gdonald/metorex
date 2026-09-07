// A lambda written without parentheses declares its parameters the same way
// one written with them does, and takes its arguments the way a method does.

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

#[test]
fn a_paren_less_lambda_parameter_takes_a_default() {
    let result = run(r#"
greet = -> name = "world" { "hello #{name}" }
[greet.call, greet.call("there")].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string(r#"["hello world", "hello there"]"#))
    );
}

#[test]
fn a_paren_less_lambda_mixes_required_and_optional_parameters() {
    let result = run("(-> a, b, c, d = nil, e = nil { [a, b, c, d, e] }).call(1, 2, 3).inspect");
    assert_eq!(result, Some(Object::string("[1, 2, 3, nil, nil]")));
}

#[test]
fn a_paren_less_lambda_takes_a_keyword_parameter_after_a_splat() {
    let result = run(r#"
keyed = -> *rest, tag: :none { [rest, tag] }
[keyed.call(1, 2), keyed.call(1, tag: :marked)].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string("[[[1, 2], :none], [[1], :marked]]"))
    );
}

#[test]
fn a_parenthesised_lambda_parameter_takes_a_default() {
    let result = run("(->(width = 3) { width }).call");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn a_lambda_refuses_the_wrong_number_of_arguments() {
    let error = run_err("(-> a, b { [a, b] }).call(1)");
    assert!(
        error.contains("wrong number of arguments (given 1, expected 2)"),
        "{}",
        error
    );
}

#[test]
fn a_method_declares_that_it_takes_no_keyword_arguments() {
    let result = run(r#"
def no_keywords(value, **nil)
  value
end
no_keywords(7)
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn a_method_handed_over_with_ampersand_becomes_the_block() {
    let result = run(r#"
class Recorder
  attr_reader :seen
  def initialize
    @seen = []
  end
  def record(pair)
    @seen.push(pair)
  end
end
recorder = Recorder.new
{ "a" => 1, "b" => 2 }.each(&recorder.method(:record))
recorder.seen.inspect
"#);
    assert_eq!(result, Some(Object::string(r#"[["a", 1], ["b", 2]]"#)));
}

#[test]
fn match_hands_the_match_data_to_a_block() {
    let result = run(r#"
["hello".match(/l(l)o/) { |found| found[1] }, :hello.match(/l(l)o/) { |found| found[0] }].inspect
"#);
    assert_eq!(result, Some(Object::string(r#"["l", "llo"]"#)));
}

#[test]
fn match_p_takes_the_offset_to_start_at() {
    let result = run(r#"["hello".match?(/l/, 3), "hello".match?(/h/, 1)].inspect"#);
    assert_eq!(result, Some(Object::string("[true, false]")));
}

#[test]
fn symbol_has_no_constructor() {
    for code in ["Symbol.new", "Symbol.allocate"] {
        let error = run_err(code);
        assert!(
            error.contains("undefined method 'new'") || error.contains("allocator undefined"),
            "{}: {}",
            code,
            error
        );
    }
}

#[test]
fn two_sets_holding_the_same_nested_sets_are_equal_whatever_the_order() {
    let result = run(r#"
left = Set.new([Set.new([1, 2]), Set.new([3])])
right = Set.new([Set.new([3]), Set.new([2, 1])])
left == right
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn integer_answers_zero_as_its_own_method() {
    let result = run("42.method(:zero?).owner.to_s");
    assert_eq!(result, Some(Object::string("Integer")));
}

#[test]
fn trap_is_kernels_name_for_signal_trap() {
    let result = run(r#"
trap("USR1") { :caught }
Kernel.private_instance_methods(false).include?(:trap)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Byte-level String readers ───────────────────────────────────────────────

#[test]
fn a_string_reports_its_bytes_as_well_as_its_characters() {
    let result = run(r#"
text = "héllo"
[text.length, text.bytesize, text.bytes, text.getbyte(0), text.getbyte(-1), text.getbyte(99)].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string(
            "[5, 6, [104, 195, 169, 108, 108, 111], 104, 111, nil]"
        ))
    );
}

#[test]
fn each_byte_walks_the_bytes_and_answers_an_enumerator_without_a_block() {
    let result = run(r#"
collected = []
"abc".each_byte { |byte| collected.push(byte) }
[collected, "abc".each_byte.to_a].inspect
"#);
    assert_eq!(result, Some(Object::string("[[97, 98, 99], [97, 98, 99]]")));
}

#[test]
fn a_string_reports_whether_it_is_ascii_and_its_first_character() {
    let result = run(
        r#"["héllo".ascii_only?, "hello".ascii_only?, "héllo".chr, "".chr, "x".valid_encoding?].inspect"#,
    );
    assert_eq!(
        result,
        Some(Object::string(r#"[false, true, "h", "", true]"#))
    );
}

#[test]
fn hex_reads_base_sixteen_and_honors_only_the_hex_prefix() {
    let result = run(
        r#"["0a".hex, "0x1f".hex, "A_BAD_BABE".hex, "0b1010".hex, "not".hex, "-1234".hex].inspect"#,
    );
    assert_eq!(
        result,
        Some(Object::string("[10, 31, 2880289470, 725008, 0, -4660]"))
    );
}

#[test]
fn oct_reads_base_eight_and_honors_every_base_prefix() {
    let result =
        run(r#"["777".oct, "0b1010".oct, "0o17".oct, "0d99".oct, "0xff".oct, "8".oct].inspect"#);
    assert_eq!(result, Some(Object::string("[511, 10, 15, 99, 255, 0]")));
}
