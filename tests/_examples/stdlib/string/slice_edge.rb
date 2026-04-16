# Slice with 2 args
s = "hello"
puts(s[1, 3])
puts(s[0, 2])
puts(s[-3, 2])

# Slice past end returns empty
puts(s[100, 2])
puts("done")

# String + method
a = "foo"
b = a + "bar"
puts(b)
