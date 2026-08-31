# A method body and a class body each have their own locals. An assignment
# inside one never reaches a same-named variable outside it.

shared = "outer"

def shadows
  shared = "inner"
  shared
end

puts shadows
puts shared

def runs_a_block
  shared = ["method", "local"]
  yield
end

runs_a_block { puts shared }
puts shared

class Body
  shared = "class body"
  BODY_VALUE = shared
end

puts Body::BODY_VALUE
puts shared

def counts
  total = 0
  [1, 2, 3].each { |value| total = total + value }
  total
end

puts counts
