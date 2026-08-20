Point = Struct.new(:x, :y)

p1 = Point.new(1, 2)
puts p1.x
puts p1.y
puts p1.to_a.inspect
puts p1.members.inspect
puts p1.size
puts p1[0]
puts p1[:y]
puts p1["x"]
p1.x = 10
puts p1.x
p1[:y] = 20
puts p1.y
puts p1.inspect
puts p1 == Point.new(10, 20)
puts p1 == Point.new(1, 1)
puts Point.new(1).y.inspect
puts Point.members.inspect
puts p1.values_at(0, 1).inspect

p1.each do |value|
  puts value
end

p1.each_pair do |name, value|
  puts "#{name}=#{value}"
end

Named = Struct.new(:value) do
  def doubled
    value * 2
  end
end

puts Named.new(21).doubled
puts Named.new(3).class.ancestors.include?(Struct)

Options = Struct.new(:host, :port, keyword_init: true)
opts = Options.new(host: "example.com", port: 80)
puts opts.host
puts opts.port

begin
  Point.new(1, 2, 3)
rescue ArgumentError => e
  puts e.message
end

begin
  p1[:missing]
rescue NameError => e
  puts e.message
end

nested = Struct.new(:inner).new({ "key" => [1, 2] })
puts nested.dig(:inner, "key", 0)
puts p1.to_h[:x]
puts p1.deconstruct.inspect

Pair = Struct.new :left, :right
pair = Pair.new "a", "b"
puts pair.left
both = pair.values_at 0, 1
puts both.inspect
puts pair.inspect
