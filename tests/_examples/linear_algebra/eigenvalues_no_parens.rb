# A matrix answers its eigenvalues as Floats, with a conjugate pair where
# the roots are not real.
require "matrix"

real = Matrix[[14, 16], [-6, -6]]
p real.eigensystem.eigenvalues
p real.eigensystem.eigenvalue_matrix

turning = Matrix[[1, 1], [-1, 1]]
p turning.eigensystem.eigenvalues
p turning.eigensystem.eigenvalue_matrix

p Matrix[[1, 2, 3], [4, 5, 6], [7, 8, 9]].eigensystem.eigenvalues.map { |value| value.round 6 }
