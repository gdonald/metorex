# Test nested array operations and matrix operations combined

# 1. Create a matrix using nested arrays
matrix = [[1, 2, 3], [4, 5, 6]]
puts "Original matrix:"
puts matrix

# 2. Access nested elements
puts "Element at [0][0]:"
puts matrix[0][0]
puts "Element at [1][2]:"
puts matrix[1][2]

# 3. Map over rows
doubled = matrix.map do |row|
  row.map do |x|
    x * 2
  end
end
puts "Doubled matrix:"
puts doubled

# 4. Transpose and then map
transposed = matrix.transpose
result = transposed.map do |col|
  col.reduce do |a, b|
    a + b
  end
end
puts "Sum of each column:"
puts result

# 5. Filter rows based on condition
matrix2 = [[1, 2], [3, 4], [5, 6]]
filtered = matrix2.filter do |row|
  row[0] > 2
end
puts "Rows where first element > 2:"
puts filtered
