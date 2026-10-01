// An Integer range that ends at a Float, and what walking it gives.

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

fn ints(values: &[i64]) -> Option<Object> {
    Some(Object::array(
        values.iter().map(|value| Object::Int(*value)).collect(),
    ))
}

#[test]
fn an_integer_range_to_a_float_stops_at_the_last_whole_number_inside() {
    assert_eq!(run("(2..3.5).to_a"), ints(&[2, 3]));
}

#[test]
fn an_exclusive_range_to_a_whole_float_leaves_the_end_out() {
    assert_eq!(run("(1...2.0).to_a"), ints(&[1]));
    assert_eq!(run("(1..2.0).to_a"), ints(&[1, 2]));
}

#[test]
fn an_integer_range_to_infinity_cannot_be_collected() {
    assert_eq!(
        run("begin\n  (1..Float::INFINITY).to_a\nrescue RangeError => error\n  error.message\nend"),
        Some(Object::string("cannot convert endless range to an array"))
    );
}

#[test]
fn map_walks_a_range_of_any_kind() {
    assert_eq!(
        run("[(2...4.0).map { |value| value * 10 }, ('a'..'c').map(&:upcase)]"),
        Some(Object::array(vec![
            ints(&[20, 30]).expect("numbers"),
            Object::array(vec![
                Object::string("A"),
                Object::string("B"),
                Object::string("C")
            ])
        ]))
    );
}

#[test]
fn alias_takes_keyword_names() {
    let code = "class Counter\n  def step\n    :stepped\n  end\n  alias next step\n  alias redo step\n  alias retry step\n  alias for step\n  alias raise step\n  alias begin step\n  alias lambda step\n  alias yield step\n  alias return step\n  alias break step\n  alias true step\n  alias false step\n  alias nil step\nend\nheld = Counter.new\n%i[next redo retry for raise begin lambda yield return break true false nil].map { |name| held.public_send(name) }.uniq";
    assert_eq!(
        run(code),
        Some(Object::array(vec![Object::symbol("stepped".to_string())]))
    );
}

#[test]
fn a_compound_assignment_is_the_subject_of_a_case() {
    assert_eq!(
        run(
            "seen = Hash.new(0)\nanswers = []\n2.times do\n  case seen[:word] += 1\n  when 1 then answers << :first\n  when 2 then answers << :second\n  end\nend\nanswers"
        ),
        Some(Object::array(vec![
            Object::symbol("first".to_string()),
            Object::symbol("second".to_string())
        ]))
    );
}

#[test]
fn a_compound_assignment_is_a_loop_condition() {
    assert_eq!(
        run("count = 0\nwhile (count += 1) < 3 do end\ncount"),
        Some(Object::Int(3))
    );
}

#[test]
fn abbrev_drops_a_beginning_two_words_share() {
    assert_eq!(
        run("require 'abbrev'\n[Abbrev.abbrev(%w[car cone]).to_a, Abbrev.abbrev(%w[car cone], 'co').keys]").map(|answer| answer.to_string()),
        run("[[['car', 'car'], ['ca', 'car'], ['cone', 'cone'], ['con', 'cone'], ['co', 'cone']], ['cone', 'con', 'co']]").map(|answer| answer.to_string())
    );
}

#[test]
fn float_of_a_rational_is_its_value() {
    assert_eq!(run("Float(Rational(1, 4))"), Some(Object::Float(0.25)));
}

#[test]
fn rational_coerce_follows_the_kind_of_the_other_number() {
    assert_eq!(
        run(
            "[Rational(3, 4).coerce(1.5), Rational(3, 4).coerce(10), Rational(3, 4).coerce(Rational(1, 2)), Rational(3, 4).coerce(Complex(5)), Rational(3, 4).coerce(Complex(5, 1))].inspect"
        ),
        Some(Object::string(
            "[[1.5, 0.75], [(10/1), (3/4)], [(1/2), (3/4)], [(5/1), (3/4)], [(5+1i), ((3/4)+0i)]]"
        ))
    );
}

#[test]
fn rational_coerce_refuses_what_is_not_a_number() {
    assert_eq!(
        run("begin\n  Rational(3, 4).coerce('x')\nrescue TypeError => error\n  error.message\nend"),
        Some(Object::string("String can't be coerced into Rational"))
    );
}

#[test]
fn a_spaced_parenthesis_after_a_receiver_call_is_the_first_argument() {
    assert_eq!(
        run(
            "held = Object.new\ndef held.take(*values)\n  values\nend\nfirst = held.take (1), 2\nsecond = held.take (1) + 2\n[first, second].inspect"
        ),
        Some(Object::string("[[1, 2], [3]]"))
    );
}

#[test]
fn a_library_required_under_two_names_runs_once() {
    assert_eq!(
        run(
            "require 'bigdecimal'\nfirst = BigDecimal::VERSION.object_id\nrequire 'bigdecimal/util'\nBigDecimal::VERSION.object_id == first"
        ),
        Some(Object::Bool(true))
    );
}

#[test]
fn bigdecimal_round_answers_an_integer_only_without_a_rule() {
    assert_eq!(
        run(
            "require 'bigdecimal'\nvalue = BigDecimal('2.5')\n[value.round, value.round(0), value.round(0, :half_even), value.round(half: :even), value.round(half: nil), BigDecimal('25').round(-1)].inspect"
        ),
        Some(Object::string("[3, 3, 0.2e1, 0.2e1, 0.3e1, 30]"))
    );
}

#[test]
fn bigdecimal_round_refuses_an_unknown_half_rule_and_extra_arguments() {
    assert_eq!(
        run(
            "require 'bigdecimal'\nvalue = BigDecimal('2.5')\n[(value.round(half: :bad) rescue $!.message), (value.round(0, 1, 2) rescue $!.message)]"
        ),
        Some(Object::array(vec![
            Object::string("invalid rounding mode (bad)"),
            Object::string("wrong number of arguments (given 3, expected 0..2)")
        ]))
    );
}

#[test]
fn bigdecimal_sqrt_carries_the_digits_asked_for() {
    assert_eq!(
        run(
            "require 'bigdecimal'\n[BigDecimal('2').sqrt(20), BigDecimal('0.0004').sqrt(0), (BigDecimal('-1').sqrt(5) rescue $!.message)].inspect"
        ),
        Some(Object::string(
            "[0.14142135623730950488e1, 0.2e-1, \"sqrt of negative value\"]"
        ))
    );
}

#[test]
fn deflating_less_than_the_lookahead_answers_only_the_header() {
    assert_eq!(
        run("require 'zlib'\nheld = Zlib::Deflate.new\n[held.deflate('hello').bytes, held.finish.bytesize]").map(|answer| answer.to_string()),
        run("[[120, 156], 11]").map(|answer| answer.to_string())
    );
}

#[test]
fn a_dictionary_deflater_hands_on_its_header_and_dictionary_id() {
    assert_eq!(
        run(
            "require 'zlib'\nheld = Zlib::Deflate.new\nheld.set_dictionary('hello')\nheld.deflate('hello hello').bytesize"
        ),
        Some(Object::Int(6))
    );
}
