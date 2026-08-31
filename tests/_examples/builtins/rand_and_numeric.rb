srand 12345

value = rand
puts value.is_a?(Float)
puts (value >= 0.0 && value < 1.0)

puts rand(77).is_a?(Integer)
puts rand(1.3).is_a?(Integer)
puts rand(0).is_a?(Float)
puts rand(0.999).is_a?(Float)

within = 1000.times.all? { |i| (0...100).include?(rand(100)) }
puts within

negative_stays_in_range = 1000.times.all? { |i| (0...4).include?(rand(-4)) }
puts negative_stays_in_range

puts rand(4...6).is_a?(Integer)
puts rand(4...6.5).is_a?(Float)
puts rand(3.5..6).is_a?(Float)
puts rand(1..0).inspect
puts rand(42..42).inspect
puts rand(1.5..1.5).inspect

class Limit
  def to_int
    7
  end
end
puts rand(Limit.new).is_a?(Integer)

begin
  rand("hello")
rescue TypeError => error
  puts "#{error.class}: #{error.message}"
end

puts 5.is_a?(Numeric)
puts 0.5.is_a?(Numeric)
puts Integer.superclass.name
puts Float.superclass.name

unit = (0...1)
puts unit.include?(0.38)
mixed = (4...6.5)
puts mixed.include?(6.12)
float_start = (3.5..6)
puts float_start.include?(5.93)
puts 0.38 <=> 0
puts 5 <=> 5.5

puts Kernel.private_instance_methods(false).include?(:rand)
