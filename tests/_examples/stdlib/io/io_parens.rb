# IO and print functions demonstration (with parentheses)

# puts adds newline
puts("Hello from puts")

# print does not add newline
print("Hello ")
print("from ")
print("print")
puts("")

# p shows inspect representation
p(42)

# File operations
path = "tests/_examples/stdlib/io/io_temp_parens.txt"
File.write(path, "Hello from file!")
puts(File.read(path))
puts(File.exist?(path))

# Cleanup
File.write(path, "")
