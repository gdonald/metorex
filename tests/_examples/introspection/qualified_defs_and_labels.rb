# `def Named.method` names the object it says rather than the class the body
# opens, wherever it is written.
module Outer
  module Inner
    def Outer.from_a_nested_body
      :from_the_nested_body
    end
  end
end

class Holder
  def Outer.from_a_class_body
    :from_the_class_body
  end
end

puts Outer.from_a_nested_body.inspect
puts Outer.from_a_class_body.inspect
puts Holder.respond_to?(:from_a_class_body).inspect

# A def written outside every class belongs to Object, which is the name a
# backtrace gives it however it is reached.
LABEL = -> { caller_locations(1, 1)[0].label }

def written_at_the_top_level
  LABEL.call
end

class Caller
  def reach
    written_at_the_top_level
  end
end

puts Caller.new.reach

# The core library's own Ruby source is no file of the program's.
puts Kernel.instance_method(:tap).source_location.first.start_with?("<internal:")
puts Object.const_source_location(:String).inspect
puts BasicObject.instance_method(:instance_exec).source_location.inspect
