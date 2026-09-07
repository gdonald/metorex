// Examples covering Matrix and Vector

use super::run_example;

/// The expected output of both `linear_algebra/matrices_and_vectors`
/// variants, which differ only in whether the calls are written with
/// parentheses.
const MATRICES_AND_VECTORS_OUTPUT: &str = "2\n2\nVector[1, 2]\nVector[2, 4]\nVector[3, 4]\nnil\n2\n[[1, 2], [3, 4]]\n[Vector[1, 2], Vector[3, 4]]\n[Vector[1, 3], Vector[2, 4]]\nMatrix[[2, 3], [4, 5]]\nMatrix[[0, 1], [2, 3]]\nMatrix[[2, 1], [4, 3]]\nMatrix[[2, 4], [6, 8]]\nMatrix[[7, 10], [15, 22]]\nMatrix[[-1, -2], [-3, -4]]\nMatrix[[1, 3], [2, 4]]\nMatrix[[1, 3], [2, 4]]\n-2\n5\n2\nMatrix[[(-2/1), (1/1)], [(3/2), (-1/2)]]\ntrue\nMatrix[[1, 0], [0, 1]]\nMatrix[[0, 0], [0, 0]]\nMatrix[[1, 0], [0, 2]]\nMatrix[[5, 0], [0, 5]]\nMatrix[[1, 2]]\nMatrix[[1], [2]]\nMatrix[[0, 1], [2, 3]]\n\"Matrix.empty(0, 2)\"\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\nMatrix[[10, 20], [30, 40]]\n[1, 0]\nMatrix[[1]]\nMatrix[[4]]\nExceptionForMatrix::ErrDimensionMismatch\nExceptionForMatrix::ErrOperationNotDefined\nVector[5, 7, 9]\nVector[-3, -3, -3]\nVector[2, 4, 6]\n32\nVector[-3, 6, -3]\n3\n2\n[1, 2, 3]\nVector[2, 3, 4]\n5.0\nVector[0.6, 0.8]\nVector[0, 0]\nVector[0, 1, 0]\ntrue\nMatrix[[1, 2, 3]]\n";

#[test]
fn test_linear_algebra_matrices_and_vectors_execution() {
    let output = run_example("linear_algebra/matrices_and_vectors.rb");
    assert_eq!(output, MATRICES_AND_VECTORS_OUTPUT);
}

#[test]
fn test_linear_algebra_matrices_and_vectors_no_parens_execution() {
    let output = run_example("linear_algebra/matrices_and_vectors_no_parens.rb");
    assert_eq!(output, MATRICES_AND_VECTORS_OUTPUT);
}
