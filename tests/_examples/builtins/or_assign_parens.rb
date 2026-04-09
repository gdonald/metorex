# ||=
x = nil
x ||= 42
puts x
x ||= 99
puts x

# &&=
y = true
y &&= false
puts y
z = false
z &&= true
puts z
w = "hello"
w &&= "world"
puts w
