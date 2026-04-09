class Stack
  def initialize
    @items = [1, 2, 3]
  end

  def []
    @items.length
  end
end

s = Stack.new
puts(s[])
