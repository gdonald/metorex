# Rectangular arrays of numbers and the arithmetic over them.
require 'matrix'

grid = Matrix[[1, 2], [3, 4]]

p grid.row_size
p grid.column_size
p grid.row(0)
p grid.column(1)
p grid.row(-1)
p grid.row(5)
p grid[0, 1]
p grid.to_a
p grid.row_vectors
p grid.column_vectors

p grid + Matrix[[1, 1], [1, 1]]
p grid - Matrix[[1, 1], [1, 1]]
p grid * Matrix[[0, 1], [1, 0]]
p grid * 2
p grid ** 2
p (-grid)
p grid.transpose
p grid.t

p grid.determinant
p grid.trace
p grid.rank
p grid.inverse
p grid * grid.inverse == Matrix.identity(2)

p Matrix.identity(2)
p Matrix.zero(2)
p Matrix.diagonal(1, 2)
p Matrix.scalar(2, 5)
p Matrix.row_vector([1, 2])
p Matrix.column_vector([1, 2])
p Matrix.build(2, 2) { |row, column| row * 2 + column }
p Matrix.empty(0, 2).to_s

p grid.square?
p grid.symmetric?
p Matrix[[1, 2], [2, 1]].symmetric?
p Matrix[[1, 0], [0, 1]].diagonal?
p Matrix[[1, 0], [0, 1]].permutation?
p Matrix[[1, 2], [0, 1]].upper_triangular?
p Matrix.zero(2).zero?
p Matrix.empty(0, 2).empty?
p grid.regular?
p grid.collect { |value| value * 10 }
p grid.find_index(3)
p grid.minor(0, 1, 0, 1)
p grid.first_minor(0, 0)

begin
  grid + Matrix[[1]]
rescue Matrix::ErrDimensionMismatch => error
  p error.class
end

begin
  grid + 2
rescue ExceptionForMatrix::ErrOperationNotDefined => error
  p error.class
end

left = Vector[1, 2, 3]
right = Vector[4, 5, 6]

p left + right
p left - right
p left * 2
p left.inner_product(right)
p left.cross_product(right)
p left.size
p left[1]
p left.to_a
p left.collect { |value| value + 1 }
p Vector[3, 4].magnitude
p Vector[3, 4].normalize
p Vector.zero(2)
p Vector.basis(size: 3, index: 1)
p Vector[0, 0].zero?
p left.covector
