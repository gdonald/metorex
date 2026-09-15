# The `module` keyword opens a module of its own when the name stands for
# something else, and refuses a name already bound to something that is not a
# module.
module Holder
  Named = "a string"
  class Inner; end
end

begin
  module Holder::Named; end
rescue TypeError => problem
  puts(problem.message)
end

begin
  module Holder::Inner; end
rescue TypeError => problem
  puts(problem.message)
end

module Mixed
  module Shared; end
end

class Object
  include Mixed
end

module Shared; end
p(Mixed::Shared.equal?(Object::Shared))

first = Module.new
second = Module.new
first::Second = second
::NamedRoot = first
p(first.name)
p(second.name)
