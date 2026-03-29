use super::run_example;

#[test]
fn test_algorithms_factorial_iterative_execution() {
    let expected = "720\n";
    let output = run_example("algorithms/factorial_iterative.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_average_temperature_execution() {
    let expected = "69.9\n";
    let output = run_example("algorithms/average_temperature.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_primes_under_fifty_execution() {
    let expected = "[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]\n";
    let output = run_example("algorithms/primes_under_fifty.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_filter_even_numbers_execution() {
    let expected = "[2, 4, 6]\n";
    let output = run_example("algorithms/filter_even_numbers.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_character_counter_execution() {
    let output = run_example("algorithms/character_counter.rb");
    assert!(
        output.contains("b")
            && output.contains("a")
            && output.contains("n")
            && output.contains(": 1")
            && output.contains(": 3")
            && output.contains(": 2"),
        "Expected output to contain all characters (b:1, a:3, n:2), but got: {}",
        output
    );
}

#[test]
fn test_algorithms_zip_merger_execution() {
    let expected = "[[Ann, 88], [Ben, 93]]\n";
    let output = run_example("algorithms/zip_merger.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_matrix_transpose_execution() {
    let expected = "[[1, 4], [2, 5], [3, 6]]\n";
    let output = run_example("algorithms/matrix_transpose.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_matrix_transpose_comprehensive_execution() {
    let expected = r#"Basic 2x3 matrix:
[[1, 4], [2, 5], [3, 6]]
Double transpose (3x2 matrix):
[[1, 2], [3, 4], [5, 6]]
Single row matrix:
[[1], [2], [3], [4]]
Single column matrix:
[[1, 2, 3]]
Square 3x3 matrix:
[[1, 4, 7], [2, 5, 8], [3, 6, 9]]
"#;
    let output = run_example("algorithms/matrix_transpose_comprehensive.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_matrix_nested_ops_execution() {
    let expected = r#"Original matrix:
[[1, 2, 3], [4, 5, 6]]
Element at [0][0]:
1
Element at [1][2]:
6
Doubled matrix:
[[2, 4, 6], [8, 10, 12]]
Sum of each column:
[5, 7, 9]
Rows where first element > 2:
[[3, 4], [5, 6]]
"#;
    let output = run_example("algorithms/matrix_nested_ops.rb");
    assert_eq!(output, expected);
}
