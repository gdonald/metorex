# AST Inspection API - inspect method bodies at runtime

class Calculator
  def add(a, b)
    a + b
  end

  def double(n)
    n * 2
  end
end

calc = Calculator.new

# get_source returns a Method object
m = calc.get_source(:add)
puts m.name
puts m.arity

# Method#body returns an array of AST node dicts
body = m.body
puts body.length

# Each node is a dict with a "type" key
node = body[0]
puts node["type"]

# Inspect the operands of a BinaryOp
puts node["op"]

# Method with single statement
m2 = calc.get_source(:double)
body2 = m2.body
puts body2.length
node2 = body2[0]
puts node2["type"]
puts node2["op"]

# Block#statements via a captured block
def capture_block(&block)
  block
end

b = capture_block do |x|
  x + 1
end
stmts = b.statements
puts stmts.length
puts b.arity
