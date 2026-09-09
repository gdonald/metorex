require 'matrix'

# A subscript may be written across several lines.
wide = Matrix[
  [1, 2, 3],
  [4, 5, 6]
]
p wide.row_size
p wide.column_size

# An empty matrix keeps the shape it was built with.
empty_rows = Matrix.columns([[], [], []])
p empty_rows.inspect
p empty_rows.row_size
p empty_rows.column_size
p empty_rows.transpose.inspect
p Matrix.empty(0, 42).eql?(Matrix.empty(0, 6))

column = Matrix.column_vector([])
p [column.row_size, column.column_size]

p wide.minor(0, 1, 0, 2).inspect
p wide.minor(1, 20, 1, 1).inspect
p wide.minor(2, 10, 1, 10).inspect
p wide.minor(0, 1, 0, -1)

p wide.find_index(5)
p wide.find_index(:diagonal).to_a
p wide.find_index { |value| value > 4 }

counted = Matrix.build(2, 3) { |row, column| row * 3 + column }
p counted.inspect
p Matrix.build(0, 3) { 1 }.column_size

flat = Matrix.empty(0, 2) * Matrix.build(2, 4) { 1 }
p [flat.row_size, flat.column_size]

pairs = []
Vector[1, 2, 3].each2([7, 8, 9]) { |mine, theirs| pairs.push([mine, theirs]) }
p pairs
p Vector[Complex(1, 2)].inner_product(Vector[Complex(3, 4)])

p Matrix.respond_to?(:new)
begin
  Matrix.new([[1]])
rescue NoMethodError => error
  p error.class
end
