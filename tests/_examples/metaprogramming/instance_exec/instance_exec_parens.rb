# instance_exec example (parens)
class Box
  def initialize(val)
    @value = val
  end

  def value
    @value
  end

  def transform(&blk)
    instance_exec(&blk)
  end

  def doubled
    instance_exec do
      @value * 2
    end
  end
end

b = Box.new(5)
puts(b.doubled)

b2 = Box.new(10)
puts(b2.doubled)

puts(b.transform { @value + 100 })
