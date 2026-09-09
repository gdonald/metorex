// Coverage tests for empty matrices, the pieces taken out of a matrix, a
// subscript written across several lines, and a private `new`.

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

/// The answer of one expression, with matrix loaded first.
fn with_matrix(body: &str) -> Option<Object> {
    run(&format!("require 'matrix'\n{body}"))
}

fn number(value: i64) -> Option<Object> {
    Some(Object::Int(value))
}

// ── The shape of an empty matrix ───────────────────────────────────────────

#[test]
fn an_empty_matrix_keeps_the_shape_it_was_built_with() {
    assert_eq!(
        with_matrix("Matrix.columns([[], [], []]).column_size"),
        number(3)
    );
    assert_eq!(
        with_matrix("Matrix.columns([[], [], []]).row_size"),
        number(0)
    );
    assert_eq!(with_matrix("Matrix[[], [], []].row_size"), number(3));
    assert_eq!(
        with_matrix("Matrix.column_vector([]).column_size"),
        number(1)
    );
    assert_eq!(
        with_matrix("Matrix.build(0, 4) { 1 }.column_size"),
        number(4)
    );
    assert_eq!(
        with_matrix("(Matrix.empty(0, 2) * Matrix.build(2, 5) { 1 }).column_size"),
        number(5)
    );
}

#[test]
fn turning_an_empty_matrix_over_swaps_its_two_sizes() {
    assert_eq!(
        with_matrix("Matrix.columns([[], [], []]).transpose.row_size"),
        number(3)
    );
    assert_eq!(
        with_matrix("Matrix[[], [], []].inspect"),
        Some(Object::string("Matrix.empty(3, 0)"))
    );
}

#[test]
fn two_empty_matrices_of_different_widths_are_not_equal() {
    assert_eq!(
        with_matrix("Matrix.empty(0, 42).eql?(Matrix.empty(0, 6))"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        with_matrix("Matrix.empty(0, 6).eql?(Matrix.empty(0, 6))"),
        Some(Object::Bool(true))
    );
}

// ── The piece taken out of a matrix ────────────────────────────────────────

#[test]
fn a_minor_reads_a_rectangle_and_answers_nil_when_it_cannot() {
    assert_eq!(
        with_matrix("Matrix[[1, 2], [3, 4], [5, 6]].minor(1, 20, 1, 1).inspect"),
        Some(Object::string("Matrix[[4], [6]]"))
    );
    assert_eq!(
        with_matrix("Matrix[[1, 2], [3, 4], [5, 6]].minor(1..2, 1..2).inspect"),
        Some(Object::string("Matrix[[4], [6]]"))
    );
    assert_eq!(
        with_matrix("Matrix[[1, 2], [3, 4], [5, 6]].minor(0, 1, 0, -1)"),
        Some(Object::Nil)
    );
    assert_eq!(
        with_matrix("Matrix[[1, 2], [3, 4], [5, 6]].minor(4, 0, 0, 10)"),
        Some(Object::Nil)
    );
    assert_eq!(
        with_matrix("Matrix[[1, 2], [3, 4], [5, 6]].minor(3, 10, 1, 10).column_size"),
        number(1)
    );
}

#[test]
fn find_index_reads_a_selector_a_value_or_a_block() {
    assert_eq!(
        with_matrix("Matrix[[1, 2], [3, 4]].find_index(:diagonal).to_a.inspect"),
        Some(Object::string("[1, 4]"))
    );
    assert_eq!(
        with_matrix("Matrix[[1, 2], [3, 4]].find_index(4).inspect"),
        Some(Object::string("[1, 1]"))
    );
    assert_eq!(
        with_matrix("Matrix[[1, 2], [3, 4]].find_index(3, :strict_lower).inspect"),
        Some(Object::string("[1, 0]"))
    );
    assert_eq!(
        with_matrix("Matrix[[1, 2], [3, 4]].find_index { |value| value > 2 }.inspect"),
        Some(Object::string("[1, 0]"))
    );
    assert_eq!(
        with_matrix("Matrix[[1, 2], [3, 4]].find_index { false }"),
        Some(Object::Nil)
    );
    assert_eq!(
        with_matrix("Matrix[[1, 2], [3, 4]].find_index.to_a.inspect"),
        Some(Object::string("[1, 2, 3, 4]"))
    );
}

// ── Building a matrix ──────────────────────────────────────────────────────

#[test]
fn a_size_given_to_build_goes_through_to_int() {
    assert_eq!(
        with_matrix(
            "counted = Object.new\ndef counted.to_int\n  2\nend\nMatrix.build(counted, counted) { 7 }.row_size"
        ),
        number(2)
    );
    assert_eq!(
        with_matrix(
            "begin\n  Matrix.build('two') { 7 }\nrescue TypeError => error\n  error.class.name\nend"
        ),
        Some(Object::string("TypeError"))
    );
}

#[test]
fn a_matrix_is_made_through_its_named_builders_rather_than_new() {
    assert_eq!(
        with_matrix("Matrix.respond_to?(:new)"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        with_matrix(
            "begin\n  Matrix.new([[1]])\nrescue NoMethodError => error\n  error.class.name\nend"
        ),
        Some(Object::string("NoMethodError"))
    );
    assert_eq!(with_matrix("Matrix.rows([[1, 2]]).column_size"), number(2));
}

// ── A subscript written across several lines ───────────────────────────────

#[test]
fn a_subscript_may_be_written_across_several_lines() {
    assert_eq!(
        run("held = [10, 20, 30]\nheld[\n  1\n]"),
        Some(Object::Int(20))
    );
    assert_eq!(
        with_matrix("Matrix[\n  [1, 2],\n  [3, 4]\n].column_size"),
        number(2)
    );
}

// ── Pairing a vector with a plain Array ────────────────────────────────────

#[test]
fn a_vector_pairs_with_anything_sized_and_indexable() {
    assert_eq!(
        with_matrix(
            "seen = []\nVector[1, 2, 3].each2([7, 8, 9]) { |mine, theirs| seen.push(mine + theirs) }\nseen.inspect"
        ),
        Some(Object::string("[8, 10, 12]"))
    );
    assert_eq!(
        with_matrix("Vector[1, 2].collect2([3, 4]) { |mine, theirs| mine * theirs }.inspect"),
        Some(Object::string("[3, 8]"))
    );
}

#[test]
fn an_inner_product_takes_the_conjugate_of_its_argument() {
    assert_eq!(
        with_matrix("Vector[Complex(1, 2)].inner_product(Vector[Complex(3, 4)]).to_s"),
        Some(Object::string("11+2i"))
    );
    assert_eq!(
        with_matrix("Vector[1, 2].inner_product(Vector[3, 4])"),
        number(11)
    );
}

// ── What an enumerator collects ────────────────────────────────────────────

#[test]
fn a_walk_collected_by_to_a_does_not_look_like_a_truthy_block() {
    assert_eq!(
        run("[1, 2, 3].each_entry.to_a.inspect"),
        Some(Object::string("[1, 2, 3]"))
    );
    assert_eq!(
        run("held = [4, 5, 6].each\nheld.to_a.inspect"),
        Some(Object::string("[4, 5, 6]"))
    );
}
