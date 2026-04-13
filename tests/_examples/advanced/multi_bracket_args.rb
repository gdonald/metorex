# Test obj[a, b, c] syntax (multi-arg bracket indexing)
class Grid
  def [](row, col, depth)
    "#{row},#{col},#{depth}"
  end
end
g = Grid.new
puts g[1, 2, 3]
