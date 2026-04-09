class MyArray
  def initialize
    @items = [10, 20, 30]
  end

  def each
    i = 0
    while i < @items.length
      yield @items[i]
      i = i + 1
    end
  end

  def map
    result = []
    i = 0
    while i < @items.length
      result << yield(@items[i])
      i = i + 1
    end
    result
  end
end

arr = MyArray.new

arr.each do |x|
  puts x
end

mapped = arr.map do |x|
  x * 2
end
puts mapped
