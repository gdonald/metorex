# Code run through a binding runs where the binding was taken: a class it
# opens is nested there, `__method__` names the method, and the refinements
# in force there apply.
module Vault
  class Keeper
    def capture
      binding
    end
  end

  module AddShout
    refine String do
      def shout
        upcase
      end
    end
  end

  class Refined
    using AddShout

    def self.capture
      binding
    end
  end
end

context = Vault::Keeper.new.capture
context.eval "class Inner; end"
puts context.eval "Inner.name"
puts context.eval("__method__").inspect
puts Vault::Refined.capture.eval "'quiet'.shout"

# A binding with no filename of its own names the place the eval was written,
# and that place holds no directory.
here = binding
puts here.eval("__FILE__").start_with?("(eval at ")
puts here.eval("__dir__").inspect
puts here.eval "__FILE__", "given.rb"
