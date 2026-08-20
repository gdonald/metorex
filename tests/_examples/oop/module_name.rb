module Outer
end

module Outer::Inner
end

anonymous = Module.new
module anonymous::Nested; end

puts Module.new.name.inspect
puts Outer::Inner.name.inspect
puts Module.new.singleton_class.name.inspect
puts String.singleton_class.name.inspect
puts anonymous::Nested.name.start_with?("#<Module:0x").inspect
puts anonymous::Nested.name.end_with?("::Nested").inspect

Outer::Inner::Bound = anonymous
puts anonymous.name.inspect
puts anonymous::Nested.name.inspect

module Outer
  Conditional ||= Module.new
end
puts Outer::Conditional.name.inspect

Outer::AlsoConditional ||= Module.new
puts Outer::AlsoConditional.name.inspect

names = ["ab", "cd"]
puts names.one?(/a/).inspect
puts names.none?(/z/).inspect
puts "text".encoding.inspect
