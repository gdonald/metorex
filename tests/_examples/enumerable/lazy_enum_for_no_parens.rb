infinite = (0..Float::INFINITY).lazy

puts Enumerator::Lazy.instance_method(:enum_for).arity < 0

wrapped = infinite.enum_for
puts wrapped.instance_of? Enumerator::Lazy
puts wrapped.equal? infinite

puts Enumerator::Lazy.new(Object.new, 100) {}.enum_for.size.inspect
puts Enumerator::Lazy.new(Object.new, 100) {}.enum_for { 30 }.size

p infinite.enum_for(:with_index, 10).first 3
p infinite.to_enum(:each_slice, 2).map { |pair| pair.first * pair.last }.first 4

{ each_with_index: [],
  with_index: [],
  cycle: [1],
  each_with_object: [Object.new],
  with_object: [Object.new],
  each_slice: [2],
  each_entry: [],
  each_cons: [2]
}.each_pair do |name, args|
  puts "#{name} #{infinite.send(name, *args).instance_of? Enumerator::Lazy}"
end

range = 0..Float::INFINITY
p range.lazy.enum_for(:with_index).first 5
p range.first(5).to_enum.enum_for(:with_index).to_a
