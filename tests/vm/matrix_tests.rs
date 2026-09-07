// Coverage tests for Matrix and Vector

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

const GRID: &str = "require 'matrix'\ngrid = Matrix[[1, 2], [3, 4]]\n";

// ── Building a matrix ──────────────────────────────────────────────────────

#[test]
fn a_matrix_reports_its_shape_and_elements() {
    assert_eq!(run(&format!("{GRID}grid.row_size")), Some(Object::Int(2)));
    assert_eq!(
        run(&format!("{GRID}grid.column_size")),
        Some(Object::Int(2))
    );
    assert_eq!(run(&format!("{GRID}grid[0, 1]")), Some(Object::Int(2)));
    assert_eq!(
        run(&format!("{GRID}grid[0, 5].inspect")),
        Some(Object::string("nil"))
    );
    assert_eq!(
        run(&format!("{GRID}grid.to_a.inspect")),
        Some(Object::string("[[1, 2], [3, 4]]"))
    );
}

#[test]
fn a_matrix_answers_its_rows_and_columns_as_vectors() {
    assert_eq!(
        run(&format!("{GRID}grid.row(0).inspect")),
        Some(Object::string("Vector[1, 2]"))
    );
    assert_eq!(
        run(&format!("{GRID}grid.column(1).inspect")),
        Some(Object::string("Vector[2, 4]"))
    );
    assert_eq!(
        run(&format!("{GRID}grid.row(-1).inspect")),
        Some(Object::string("Vector[3, 4]"))
    );
    assert_eq!(
        run(&format!("{GRID}grid.row(5).inspect")),
        Some(Object::string("nil"))
    );
    assert_eq!(
        run(&format!(
            "{GRID}collected = []\ngrid.row(0) {{ |value| collected.push(value) }}\ncollected.inspect"
        )),
        Some(Object::string("[1, 2]"))
    );
}

#[test]
fn a_matrix_is_built_from_rows_columns_or_a_block() {
    assert_eq!(
        run("require 'matrix'\nMatrix.rows([[1, 2]]).inspect"),
        Some(Object::string("Matrix[[1, 2]]"))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix.columns([[1, 2]]).inspect"),
        Some(Object::string("Matrix[[1], [2]]"))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix.build(2, 2) { |row, column| row * 2 + column }.inspect"),
        Some(Object::string("Matrix[[0, 1], [2, 3]]"))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix.identity(2).inspect"),
        Some(Object::string("Matrix[[1, 0], [0, 1]]"))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix.diagonal(1, 2).inspect"),
        Some(Object::string("Matrix[[1, 0], [0, 2]]"))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix.scalar(2, 5).inspect"),
        Some(Object::string("Matrix[[5, 0], [0, 5]]"))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix.zero(2).inspect"),
        Some(Object::string("Matrix[[0, 0], [0, 0]]"))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix.row_vector([1, 2]).inspect"),
        Some(Object::string("Matrix[[1, 2]]"))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix.column_vector([1, 2]).inspect"),
        Some(Object::string("Matrix[[1], [2]]"))
    );
}

#[test]
fn a_matrix_takes_vectors_as_its_rows() {
    assert_eq!(
        run("require 'matrix'\nMatrix[Vector[1, 2], Vector[3, 4]].inspect"),
        Some(Object::string("Matrix[[1, 2], [3, 4]]"))
    );
}

#[test]
fn a_matrix_refuses_rows_it_cannot_read() {
    let err = run_err("require 'matrix'\nMatrix[5]");
    assert!(err.contains("expected Array"), "Error was: {}", err);
    let err = run_err("require 'matrix'\nMatrix[[0], [0, 1]]");
    assert!(err.contains("ErrDimensionMismatch"), "Error was: {}", err);
}

// ── Arithmetic ─────────────────────────────────────────────────────────────

#[test]
fn matrices_add_subtract_and_multiply() {
    assert_eq!(
        run(&format!("{GRID}(grid + Matrix[[1, 1], [1, 1]]).inspect")),
        Some(Object::string("Matrix[[2, 3], [4, 5]]"))
    );
    assert_eq!(
        run(&format!("{GRID}(grid - Matrix[[1, 1], [1, 1]]).inspect")),
        Some(Object::string("Matrix[[0, 1], [2, 3]]"))
    );
    assert_eq!(
        run(&format!("{GRID}(grid * Matrix[[0, 1], [1, 0]]).inspect")),
        Some(Object::string("Matrix[[2, 1], [4, 3]]"))
    );
    assert_eq!(
        run(&format!("{GRID}(grid * 2).inspect")),
        Some(Object::string("Matrix[[2, 4], [6, 8]]"))
    );
    assert_eq!(
        run(&format!("{GRID}(grid ** 2).inspect")),
        Some(Object::string("Matrix[[7, 10], [15, 22]]"))
    );
    assert_eq!(
        run(&format!("{GRID}(-grid).inspect")),
        Some(Object::string("Matrix[[-1, -2], [-3, -4]]"))
    );
}

#[test]
fn a_matrix_refuses_arithmetic_it_has_no_meaning_for() {
    let err = run_err(&format!("{GRID}grid + Matrix[[1]]"));
    assert!(err.contains("ErrDimensionMismatch"), "Error was: {}", err);
    let err = run_err(&format!("{GRID}grid + 2"));
    assert!(err.contains("ErrOperationNotDefined"), "Error was: {}", err);
    let err = run_err(&format!("{GRID}grid + \"x\""));
    assert!(err.contains("wrong argument type"), "Error was: {}", err);
}

#[test]
fn a_matrix_turns_on_its_side() {
    assert_eq!(
        run(&format!("{GRID}grid.transpose.inspect")),
        Some(Object::string("Matrix[[1, 3], [2, 4]]"))
    );
    assert_eq!(
        run(&format!("{GRID}grid.t.inspect")),
        Some(Object::string("Matrix[[1, 3], [2, 4]]"))
    );
}

// ── Numbers a matrix stands for ────────────────────────────────────────────

#[test]
fn a_matrix_answers_its_determinant_trace_and_rank() {
    assert_eq!(
        run(&format!("{GRID}grid.determinant")),
        Some(Object::Int(-2))
    );
    assert_eq!(run(&format!("{GRID}grid.det")), Some(Object::Int(-2)));
    assert_eq!(run(&format!("{GRID}grid.trace")), Some(Object::Int(5)));
    assert_eq!(run(&format!("{GRID}grid.rank")), Some(Object::Int(2)));
    assert_eq!(
        run("require 'matrix'\nMatrix[[1, 2, 3], [2, 4, 6]].rank"),
        Some(Object::Int(1))
    );
}

#[test]
fn a_matrix_answers_the_one_that_undoes_it() {
    assert_eq!(
        run(&format!(
            "{GRID}(grid * grid.inverse == Matrix.identity(2))"
        )),
        Some(Object::Bool(true))
    );
    let err = run_err("require 'matrix'\nMatrix[[1, 1], [1, 1]].inverse");
    assert!(err.contains("ErrNotRegular"), "Error was: {}", err);
}

#[test]
fn a_matrix_says_what_shape_it_has() {
    assert_eq!(
        run(&format!("{GRID}grid.square?")),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!("{GRID}grid.symmetric?")),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix[[1, 2], [2, 1]].symmetric?"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix[[1, 0], [0, 1]].diagonal?"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix[[0, 1], [1, 0]].permutation?"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix[[1, 2], [0, 1]].upper_triangular?"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix.zero(2).zero?"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix.empty(0, 2).empty?"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!("{GRID}grid.regular?")),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!("{GRID}grid.singular?")),
        Some(Object::Bool(false))
    );
}

// ── Walking and reshaping ──────────────────────────────────────────────────

#[test]
fn a_matrix_walks_its_elements() {
    assert_eq!(
        run(&format!(
            "{GRID}collected = []\ngrid.each {{ |value| collected.push(value) }}\ncollected.inspect"
        )),
        Some(Object::string("[1, 2, 3, 4]"))
    );
    assert_eq!(
        run(&format!(
            "{GRID}collected = []\ngrid.each(:diagonal) {{ |value| collected.push(value) }}\ncollected.inspect"
        )),
        Some(Object::string("[1, 4]"))
    );
    assert_eq!(
        run(&format!(
            "{GRID}collected = []\ngrid.each_with_index {{ |value, row, column| collected.push(row) }}\ncollected.inspect"
        )),
        Some(Object::string("[0, 0, 1, 1]"))
    );
    assert_eq!(
        run(&format!(
            "{GRID}grid.collect {{ |value| value * 10 }}.inspect"
        )),
        Some(Object::string("Matrix[[10, 20], [30, 40]]"))
    );
    assert_eq!(
        run(&format!("{GRID}grid.find_index(3).inspect")),
        Some(Object::string("[1, 0]"))
    );
}

#[test]
fn a_matrix_answers_a_piece_of_itself() {
    assert_eq!(
        run(&format!("{GRID}grid.minor(0, 1, 0, 1).inspect")),
        Some(Object::string("Matrix[[1]]"))
    );
    assert_eq!(
        run(&format!("{GRID}grid.first_minor(0, 0).inspect")),
        Some(Object::string("Matrix[[4]]"))
    );
    assert_eq!(
        run(&format!("{GRID}grid.cofactor(0, 0)")),
        Some(Object::Int(4))
    );
}

#[test]
fn two_matrices_match_when_their_elements_do() {
    assert_eq!(
        run("require 'matrix'\nMatrix[[1]] == Matrix[[1]]"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix[[1]] == Matrix[[2]]"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix[[1]] == 5"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run("require 'matrix'\nMatrix[[1]].hash == Matrix[[1]].hash"),
        Some(Object::Bool(true))
    );
}

// ── Vectors ────────────────────────────────────────────────────────────────

#[test]
fn vectors_add_subtract_and_scale() {
    let held = "require 'matrix'\nleft = Vector[1, 2, 3]\nright = Vector[4, 5, 6]\n";
    assert_eq!(
        run(&format!("{held}(left + right).inspect")),
        Some(Object::string("Vector[5, 7, 9]"))
    );
    assert_eq!(
        run(&format!("{held}(left - right).inspect")),
        Some(Object::string("Vector[-3, -3, -3]"))
    );
    assert_eq!(
        run(&format!("{held}(left * 2).inspect")),
        Some(Object::string("Vector[2, 4, 6]"))
    );
    assert_eq!(
        run(&format!("{held}left.inner_product(right)")),
        Some(Object::Int(32))
    );
    assert_eq!(
        run(&format!("{held}left.cross_product(right).inspect")),
        Some(Object::string("Vector[-3, 6, -3]"))
    );
}

#[test]
fn a_vector_reports_what_it_holds() {
    let held = "require 'matrix'\nheld = Vector[1, 2, 3]\n";
    assert_eq!(run(&format!("{held}held.size")), Some(Object::Int(3)));
    assert_eq!(run(&format!("{held}held[1]")), Some(Object::Int(2)));
    assert_eq!(
        run(&format!("{held}held.to_a.inspect")),
        Some(Object::string("[1, 2, 3]"))
    );
    assert_eq!(
        run(&format!(
            "{held}held.collect {{ |value| value + 1 }}.inspect"
        )),
        Some(Object::string("Vector[2, 3, 4]"))
    );
    assert_eq!(
        run(&format!("{held}held.covector.inspect")),
        Some(Object::string("Matrix[[1, 2, 3]]"))
    );
}

#[test]
fn a_vector_answers_its_length_and_direction() {
    assert_eq!(
        run("require 'matrix'\nVector[3, 4].magnitude"),
        Some(Object::Float(5.0))
    );
    assert_eq!(
        run("require 'matrix'\nVector[3, 4].norm"),
        Some(Object::Float(5.0))
    );
    assert_eq!(
        run("require 'matrix'\nVector[3, 4].normalize.inspect"),
        Some(Object::string("Vector[0.6, 0.8]"))
    );
    assert_eq!(
        run("require 'matrix'\nVector[0, 0].zero?"),
        Some(Object::Bool(true))
    );
    let err = run_err("require 'matrix'\nVector[0, 0].normalize");
    assert!(err.contains("Zero vectors"), "Error was: {}", err);
}

#[test]
fn a_vector_is_built_from_a_list_a_basis_or_zeros() {
    assert_eq!(
        run("require 'matrix'\nVector.elements([1, 2]).inspect"),
        Some(Object::string("Vector[1, 2]"))
    );
    assert_eq!(
        run("require 'matrix'\nVector.zero(2).inspect"),
        Some(Object::string("Vector[0, 0]"))
    );
    assert_eq!(
        run("require 'matrix'\nVector.basis(size: 3, index: 1).inspect"),
        Some(Object::string("Vector[0, 1, 0]"))
    );
}

#[test]
fn a_vector_walks_two_at_a_time() {
    assert_eq!(
        run(
            "require 'matrix'\ncollected = []\nVector[1, 2].each2(Vector[3, 4]) { |a, b| collected.push(a + b) }\ncollected.inspect"
        ),
        Some(Object::string("[4, 6]"))
    );
    let err = run_err("require 'matrix'\nVector[1].each2(Vector[1, 2]) { |a, b| a }");
    assert!(err.contains("ErrDimensionMismatch"), "Error was: {}", err);
}

// ── A splat spreads across a subscript ─────────────────────────────────────

#[test]
fn a_splat_spreads_across_a_subscript() {
    assert_eq!(
        run(
            "class Held\n  def self.[](*args)\n    args\n  end\nend\nvalues = [1, 2]\nHeld[*values].inspect"
        ),
        Some(Object::string("[1, 2]"))
    );
}

// ── A method may be named for the power operator ───────────────────────────

#[test]
fn a_method_may_be_named_for_the_power_operator() {
    assert_eq!(
        run("class Held\n  def **(count)\n    count * 2\n  end\nend\nHeld.new ** 3"),
        Some(Object::Int(6))
    );
}
