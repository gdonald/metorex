# A class or module body names itself in a backtrace by the class or module
# alone, without the namespace it was written in.
module Outer
  module Inner
    p(caller_locations(0, 1)[0].base_label)
  end

  class Held
    p(caller_locations(0, 1)[0].base_label)
  end

  class << Object.new
    p(caller_locations(0, 1)[0].base_label)
  end
end

def named_frame
  caller_locations(0, 1)[0].base_label
end

p(named_frame)
