# AST Manipulation
# Demonstrates eval, parse, and runtime code generation

# 1. eval — execute code from a string
puts "=== eval ==="
puts eval("1 + 2 + 3")
x = 10
puts eval("x * 5")
eval("y = x + 20")
puts y

# 2. eval to define methods at runtime
puts ""
puts "=== eval define method ==="
eval("def double(n)\n  n * 2\nend")
puts double(7)

# 3. eval to define classes at runtime
puts ""
puts "=== eval define class ==="
eval("class Point\n  def initialize(x, y)\n    @x = x\n    @y = y\n  end\n  def to_s\n    \"(\" + @x.to_s + \", \" + @y.to_s + \")\"\n  end\nend")
p = Point.new(3, 4)
puts p.to_s

# 4. parse — get AST representation of code
puts ""
puts "=== parse ==="
ast = parse("1 + 2")
puts ast.class
puts ast.length

# 5. Code generation with eval
puts ""
puts "=== code generation ==="
eval("def add(a, b)\n  a + b\nend")
eval("def subtract(a, b)\n  a - b\nend")
eval("def multiply(a, b)\n  a * b\nend")
puts add(3, 4)
puts subtract(10, 3)
puts multiply(5, 6)

# 6. Modify class at runtime with define_method
puts ""
puts "=== runtime modification ==="
class Calculator
  def compute(a, b)
    a + b
  end
end

calc = Calculator.new()
puts calc.compute(5, 3)

Calculator.define_method("compute") do |a, b|
  a * b
end

puts calc.compute(5, 3)
