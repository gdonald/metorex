# Slice with 2 args
s = "hello"
puts(s[1, 3])
puts(s[0, 2])
puts(s[-3, 2])

# A start past the end names no substring, where a start at the end
# names the empty one
p(s[100, 2])
p(s[5, 2])
puts("done")

# String + method
a = "foo"
b = a + "bar"
puts(b)
