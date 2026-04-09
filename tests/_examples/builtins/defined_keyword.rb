# Local variable
x = 42
puts(defined?(x))
puts(defined?(y))

# Method/function
def hello
  "hi"
end
puts(defined?(hello))

# Constant/class
puts(defined?(String))
puts(defined?(NonexistentClass))

# Global variable
$foo = "bar"
puts(defined?($foo))
puts(defined?($nonexistent))

# Literals
puts(defined?(42))
puts(defined?("hello"))
puts(defined?(nil))
puts(defined?(true))

# Without parens on defined? itself
r = defined? x
puts(r)
