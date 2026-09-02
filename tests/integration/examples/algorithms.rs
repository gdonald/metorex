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
    let expected = "2\n3\n5\n7\n11\n13\n17\n19\n23\n29\n31\n37\n41\n43\n47\n";
    let output = run_example("algorithms/primes_under_fifty.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_filter_even_numbers_execution() {
    let expected = "2\n4\n6\n";
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
            && output.contains("=> 1")
            && output.contains("=> 3")
            && output.contains("=> 2"),
        "Expected output to contain all characters (b:1, a:3, n:2), but got: {}",
        output
    );
}

#[test]
fn test_algorithms_zip_merger_execution() {
    let expected = "Ann\n88\nBen\n93\n";
    let output = run_example("algorithms/zip_merger.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_matrix_transpose_execution() {
    let expected = "1\n4\n2\n5\n3\n6\n";
    let output = run_example("algorithms/matrix_transpose.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_matrix_transpose_comprehensive_execution() {
    let expected = "Basic 2x3 matrix:\n1\n4\n2\n5\n3\n6\nDouble transpose (3x2 matrix):\n1\n2\n3\n4\n5\n6\nSingle row matrix:\n1\n2\n3\n4\nSingle column matrix:\n1\n2\n3\nSquare 3x3 matrix:\n1\n4\n7\n2\n5\n8\n3\n6\n9\n";
    let output = run_example("algorithms/matrix_transpose_comprehensive.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_matrix_nested_ops_execution() {
    let expected = "Original matrix:\n1\n2\n3\n4\n5\n6\nElement at [0][0]:\n1\nElement at [1][2]:\n6\nDoubled matrix:\n2\n4\n6\n8\n10\n12\nSum of each column:\n5\n7\n9\nRows where first element > 2:\n3\n4\n5\n6\n";
    let output = run_example("algorithms/matrix_nested_ops.rb");
    assert_eq!(output, expected);
}
