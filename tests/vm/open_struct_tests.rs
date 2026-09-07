// Coverage tests for OpenStruct, Prime, and the method_missing fallbacks

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

const HELD: &str = "require 'ostruct'\nheld = OpenStruct.new(name: \"Ada\", year: 1843)\n";

// ── Fields decided as they are set ─────────────────────────────────────────

#[test]
fn an_open_struct_reads_the_fields_it_was_built_with() {
    assert_eq!(
        run(&format!("{HELD}held.name")),
        Some(Object::string("Ada"))
    );
    assert_eq!(run(&format!("{HELD}held.year")), Some(Object::Int(1843)));
    assert_eq!(
        run(&format!("{HELD}held.missing.inspect")),
        Some(Object::string("nil"))
    );
}

#[test]
fn an_open_struct_takes_a_field_it_never_had() {
    assert_eq!(
        run(&format!("{HELD}held.city = \"London\"\nheld.city")),
        Some(Object::string("London"))
    );
    assert_eq!(
        run(&format!("{HELD}held[:city] = \"London\"\nheld[:city]")),
        Some(Object::string("London"))
    );
}

#[test]
fn an_open_struct_reads_a_field_by_name() {
    assert_eq!(
        run(&format!("{HELD}held[:name]")),
        Some(Object::string("Ada"))
    );
    assert_eq!(
        run(&format!("{HELD}held[\"name\"]")),
        Some(Object::string("Ada"))
    );
}

#[test]
fn an_open_struct_drops_a_field_and_its_accessors() {
    assert_eq!(
        run(&format!(
            "{HELD}held.delete_field(:name)\nheld[:name].inspect"
        )),
        Some(Object::string("nil"))
    );
    assert_eq!(
        run(&format!(
            "{HELD}held.delete_field(:name)\nheld.respond_to?(:name)"
        )),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run(&format!(
            "{HELD}held.delete_field(:name)\nheld.respond_to?(:name=)"
        )),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run(&format!("{HELD}held.respond_to?(:name)")),
        Some(Object::Bool(true))
    );
}

#[test]
fn an_open_struct_refuses_to_drop_a_field_it_never_had() {
    let err = run_err(&format!("{HELD}held.delete_field(:nope)"));
    assert!(err.contains("no field"), "Error was: {}", err);
}

#[test]
fn an_open_struct_answers_its_fields_as_a_hash() {
    assert_eq!(
        run(&format!("{HELD}held.to_h.inspect")),
        Some(Object::string("{name: \"Ada\", year: 1843}"))
    );
    assert_eq!(
        run(&format!(
            "{HELD}held.to_h {{ |name, value| [name.to_s, value] }}.keys.inspect"
        )),
        Some(Object::string("[\"name\", \"year\"]"))
    );
    assert_eq!(
        run(&format!(
            "{HELD}copied = held.to_h\ncopied[:year] = 0\nheld.year"
        )),
        Some(Object::Int(1843))
    );
}

#[test]
fn an_open_struct_walks_its_pairs() {
    assert_eq!(
        run(&format!(
            "{HELD}collected = []\nheld.each_pair {{ |name, value| collected.push(name) }}\ncollected.inspect"
        )),
        Some(Object::string("[:name, :year]"))
    );
}

#[test]
fn an_open_struct_digs_through_what_it_holds() {
    assert_eq!(
        run("require 'ostruct'\nheld = OpenStruct.new(a: {b: 1})\nheld.dig(:a, :b)"),
        Some(Object::Int(1))
    );
}

// ── Comparing and rendering ────────────────────────────────────────────────

#[test]
fn two_open_structs_match_when_their_fields_do() {
    assert_eq!(
        run("require 'ostruct'\nOpenStruct.new(a: 1) == OpenStruct.new(a: 1)"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("require 'ostruct'\nOpenStruct.new(a: 1) == OpenStruct.new(a: 2)"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run("require 'ostruct'\nOpenStruct.new(a: 1) == 5"),
        Some(Object::Bool(false))
    );
}

#[test]
fn an_open_struct_shows_its_class_and_fields() {
    assert_eq!(
        run(&format!("{HELD}held.inspect")),
        Some(Object::string("#<OpenStruct name=\"Ada\", year=1843>"))
    );
    assert_eq!(
        run("require 'ostruct'\nOpenStruct.new.to_s"),
        Some(Object::string("#<OpenStruct>"))
    );
}

#[test]
fn an_open_struct_that_reaches_itself_is_shown_as_its_class() {
    assert_eq!(
        run("require 'ostruct'\nheld = OpenStruct.new\nheld.self = held\nheld.inspect"),
        Some(Object::string("#<OpenStruct self=#<OpenStruct ...>>"))
    );
}

#[test]
fn a_frozen_open_struct_may_be_read_but_not_written() {
    let frozen = format!("{HELD}held.freeze\n");
    assert_eq!(
        run(&format!("{frozen}held.name")),
        Some(Object::string("Ada"))
    );
    let err = run_err(&format!("{frozen}held.year = 0"));
    assert!(err.contains("can't modify frozen"), "Error was: {}", err);
    let err = run_err(&format!("{frozen}held.city = \"London\""));
    assert!(err.contains("can't modify frozen"), "Error was: {}", err);
}

#[test]
fn an_open_struct_refuses_a_setter_with_the_wrong_number_of_arguments() {
    let err = run_err(&format!("{HELD}held.send(:city=)"));
    assert!(
        err.contains("wrong number of arguments"),
        "Error was: {}",
        err
    );
    let err = run_err(&format!("{HELD}held.name(1, 2)"));
    assert!(
        err.contains("wrong number of arguments"),
        "Error was: {}",
        err
    );
    let err = run_err(&format!("{HELD}held.nope(1, 2)"));
    assert!(err.contains("undefined method"), "Error was: {}", err);
}

// ── An unknown name reaches method_missing ─────────────────────────────────

#[test]
fn an_unknown_setter_reaches_method_missing() {
    assert_eq!(
        run(
            "class Held\n  def method_missing(name, *args)\n    [name.to_s, args]\n  end\nend\nHeld.new.anything = 5"
        ),
        Some(Object::Int(5))
    );
}

#[test]
fn send_of_an_unknown_name_reaches_method_missing() {
    assert_eq!(
        run(
            "class Held\n  def method_missing(name, *args)\n    name.to_s\n  end\nend\nHeld.new.send(:anything)"
        ),
        Some(Object::string("anything"))
    );
}

// ── The primes ─────────────────────────────────────────────────────────────

#[test]
fn prime_says_which_numbers_are_prime() {
    assert_eq!(
        run("require 'prime'\nPrime.prime?(2)"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("require 'prime'\nPrime.prime?(15)"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run("require 'prime'\nPrime.prime?(1)"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run("require 'prime'\nPrime.prime?(2**31 - 1)"),
        Some(Object::Bool(true))
    );
    assert_eq!(run("require 'prime'\n7.prime?"), Some(Object::Bool(true)));
}

#[test]
fn prime_walks_the_primes_in_order() {
    assert_eq!(
        run("require 'prime'\nPrime.first(5).inspect"),
        Some(Object::string("[2, 3, 5, 7, 11]"))
    );
    assert_eq!(
        run(
            "require 'prime'\ncollected = []\nPrime.each(11) { |found| collected.push(found) }\ncollected.inspect"
        ),
        Some(Object::string("[2, 3, 5, 7, 11]"))
    );
    assert_eq!(
        run("require 'prime'\nwalk = Prime.instance.each\nwalk.next\nwalk.next"),
        Some(Object::Int(3))
    );
    assert_eq!(
        run("require 'prime'\nwalk = Prime.instance.each\nwalk.next\nwalk.rewind.next"),
        Some(Object::Int(2))
    );
}

#[test]
fn prime_factors_a_number_and_multiplies_it_back() {
    assert_eq!(
        run("require 'prime'\nPrime.prime_division(360).inspect"),
        Some(Object::string("[[2, 3], [3, 2], [5, 1]]"))
    );
    assert_eq!(
        run("require 'prime'\nPrime.prime_division(1).inspect"),
        Some(Object::string("[]"))
    );
    assert_eq!(
        run("require 'prime'\nPrime.prime_division(-10).inspect"),
        Some(Object::string("[[-1, 1], [2, 1], [5, 1]]"))
    );
    assert_eq!(
        run("require 'prime'\nPrime.int_from_prime_division([[2, 3], [3, 2]])"),
        Some(Object::Int(72))
    );
    assert_eq!(
        run("require 'prime'\n12.prime_division.inspect"),
        Some(Object::string("[[2, 2], [3, 1]]"))
    );
    assert_eq!(
        run("require 'prime'\nInteger.from_prime_division([[2, 2], [5, 1]])"),
        Some(Object::Int(20))
    );
}

#[test]
fn prime_refuses_to_factor_zero() {
    let err = run_err("require 'prime'\nPrime.prime_division(0)");
    assert!(err.contains("divided by 0"), "Error was: {}", err);
}

#[test]
fn prime_has_one_instance() {
    assert_eq!(
        run("require 'prime'\nPrime.instance.equal?(Prime.instance)"),
        Some(Object::Bool(true))
    );
}
