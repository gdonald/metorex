# A program runs against an ordinary object at the top level. Ruby calls it
# main, and that is what it says of itself.
p self
p self.class
p self.to_s

# Its singleton class is where a `def self.name` written outside every class
# lands, so nothing else answers to that name.
def self.written_on_main
  :here
end

p written_on_main
p self.singleton_methods.sort
p Object.new.respond_to?(:written_on_main)

# Every Kernel function is a private method of every object, so a caller
# outside the object reaches one only by sending it.
held = Object.new
p held.respond_to?(:require)
p held.respond_to?(:require, true)
p held.send(:require, "tmpdir")
