# A stabby lambda whose body is one expression in braces
f = ->(x) { x + 1 }
puts f.call(4)
