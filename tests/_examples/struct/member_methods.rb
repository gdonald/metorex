Sizes = Struct.new(:length, :size)
sizes = Sizes.new(42, 7)
puts sizes.length
puts sizes.size
puts sizes.instance_variables.inspect

Point = Struct.new(:x, :y)
point = Point.new(1, 2)
puts point.each.to_a.inspect
puts point.each.size
puts point.each_pair.map { |pair| pair.inspect }.inspect
puts point.select { |value| value > 1 }.inspect
puts point.filter { |value| value.odd? }.inspect
puts point.values_at(0..1).inspect
puts point.values_at(0..3).inspect
puts point.deconstruct_keys([:x]).inspect
puts point.deconstruct_keys(nil).inspect
puts point.to_h { |name, value| [name.to_s, value * 10] }.inspect
puts Point.keyword_init?.inspect
puts Sizes.new(1, 2).hash == Point.new(1, 2).hash
puts "empty:#{nil}:"

frozen = Point.new(1, 2)
frozen.freeze
begin
  frozen.x = 5
rescue FrozenError => error
  puts error.class
end

Cyclic = Struct.new(:link, :label)
left = Cyclic.new(nil, "same")
right = Cyclic.new(nil, "same")
left.link = left
right.link = right
puts left == right
right.label = "other"
puts left == right
