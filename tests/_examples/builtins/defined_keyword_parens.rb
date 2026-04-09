# Local variable
x = 42
r = defined? x
puts r
r2 = defined? y
puts r2

# Method/function
def hello
  "hi"
end
r3 = defined? hello
puts r3

# Constant/class
r4 = defined? String
puts r4

# Literals
r5 = defined? 42
puts r5
