class Widget
end

first = Widget.new
second = Widget.new

puts first.object_id.is_a? Integer
puts (first.object_id == first.object_id)
puts (first.object_id == second.object_id)
puts (first.object_id == first.dup.object_id)

puts (1.object_id == 1.object_id)
puts (1.object_id == 2.object_id)
puts (:hello.object_id == :hello.object_id)
puts (:hello.object_id == :goodbye.object_id)
puts (true.object_id == true.object_id)
puts (false.object_id == false.object_id)
puts (nil.object_id == nil.object_id)
puts (3.14.object_id == 3.14.object_id)

puts nil.object_id
puts true.object_id
puts false.object_id
puts 1.object_id

puts ((-1).object_id == (2 ** 30 - 1).object_id)
puts ((-1).object_id == (2 ** 62 - 1).object_id)
