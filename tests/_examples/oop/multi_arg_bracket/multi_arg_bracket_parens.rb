class Matrix
  def initialize
    @data = [[1, 2], [3, 4]]
  end

  def [](row, col)
    @data[row][col]
  end
end

m = Matrix.new
puts m[0, 0]
puts m[0, 1]
puts m[1, 0]
puts m[1, 1]
