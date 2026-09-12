# The eigenvectors of a two by two matrix, the matrix they form, and the
# factorization that matrix takes part in. A fractional power is taken there
# too, on the diagonal.
require "matrix"

symmetric = Matrix[[1, 2], [2, 1]]
p(symmetric.eigensystem.eigenvalues)
p(symmetric.eigensystem.eigenvectors)
p(symmetric.eigensystem.eigenvector_matrix)

turning = Matrix[[1, 1], [-1, 1]]
p(turning.eigensystem.eigenvectors)

real = Matrix[[14, 16], [-6, -6]]
basis, diagonal, back = real.eigensystem.to_a
p((basis * diagonal * back).map { |entry| entry.round(10) })

rooted = Matrix[[5, 4], [4, 5]]**0.5
p((rooted**2).round(8))
