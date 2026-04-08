class Value
  def initialize(v)
    @v = v
  end

  def <=>(other)
    @v <=> other.val()
  end

  def <<(other)
    Value.new(@v * 2 + other.val())
  end

  def val
    @v
  end

  def to_s
    @v.to_s
  end
end

a = Value.new(3)
b = Value.new(5)

puts(a <=> b)
puts(b <=> a)
puts(a <=> a)

e = a << b
puts(e)
