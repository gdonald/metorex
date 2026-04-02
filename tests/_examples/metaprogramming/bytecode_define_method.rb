# Example: define_method for runtime method definition
#
# define_method allows creating methods dynamically at runtime.

class Greeter
  define_method(:hello, lambda do |name|
    "Hello, " + name
  end)
end

g = Greeter.new
puts g.hello("World")
