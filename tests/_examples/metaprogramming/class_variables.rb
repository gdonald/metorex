# Module#class_variables — written without parentheses. Returns class
# variable names as symbols in definition order. The optional `inherit`
# argument controls whether class variables from ancestors are included.
class Base
  @@base = 1
  @@shared = 2
end

class Derived < Base
  @@derived = 3
end

puts Base.class_variables.inspect
puts Derived.class_variables.inspect

own = Derived.class_variables false
puts own.inspect

module Flags
  @@flag = :on
end

puts Flags.class_variables.inspect
