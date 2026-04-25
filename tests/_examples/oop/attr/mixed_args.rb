class Mixed
  attr_accessor :a, "b"

  def initialize
    @a = 1
    @b = 2
  end
end

m = Mixed.new
puts m.a
puts m.b

m.a = 10
m.b = 20
puts m.a
puts m.b

# Bare identifier (variable) — evaluated and coerced via to_str.
class FromVar
  name = "value"
  attr_reader name
end

fv = FromVar.new
fv.instance_variable_set(:@value, 42)
puts fv.value
