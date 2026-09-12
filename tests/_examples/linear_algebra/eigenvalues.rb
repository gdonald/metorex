# A two by two matrix answers its eigenvalues, exact where the roots come out
# whole and a conjugate pair where they do not.
require "matrix"

real = Matrix[[14, 16], [-6, -6]]
p(real.eigensystem.eigenvalues)
p(real.eigensystem.eigenvalue_matrix)

turning = Matrix[[1, 1], [-1, 1]]
p(turning.eigensystem.eigenvalues)
p(turning.eigensystem.eigenvalue_matrix)

begin
  Matrix[[1, 2, 3], [4, 5, 6], [7, 8, 9]].eigensystem
rescue ExceptionForMatrix::ErrOperationNotImplemented => refused
  puts("a larger matrix is not worked out")
end
