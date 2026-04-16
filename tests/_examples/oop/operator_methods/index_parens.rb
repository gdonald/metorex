class Grid
  def initialize(width, height)
    @width = width
    @height = height
    @data = []
    i = 0
    while i < width * height
      @data.push(0)
      i = i + 1
    end
  end

  def [](index)
    @data[index]
  end

  def []=(index, value)
    @data[index] = value
  end
end

g = Grid.new(3, 3)
g[0] = 42
g[4] = 99

puts(g[0])
puts(g[4])
puts(g[1])
