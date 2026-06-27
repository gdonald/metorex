# Module#class_variable_set — written with parentheses. Sets a class
# variable by name, returns the assigned value, raises FrozenError when the
# receiver is frozen, and mixes in a duplicated module via `include(Mod.dup)`.
module Flags
  @@flag = :off
end

klass = Class.new { include(Flags.dup) }
puts(klass.class_variable_set("@@flag", :on))
puts(klass.class_variable_get(:@@flag))

other = Class.new
puts(other.class_variable_set(:@@count, 3))
puts(other.class_variable_get(:@@count))

begin
  Class.new.freeze.class_variable_set(:@@x, 1)
rescue FrozenError
  puts("frozen Class raises FrozenError")
end

begin
  Module.new.freeze.class_variable_set(:@@x, 1)
rescue FrozenError
  puts("frozen Module raises FrozenError")
end
