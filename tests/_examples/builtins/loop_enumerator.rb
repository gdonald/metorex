counted = 0
result = loop do
  counted += 1
  break counted if counted >= 3
end
puts result

generated = Enumerator.new do |yielder|
  yielder << 1
  yielder << 2
  :finished
end

puts generated.next
puts generated.next
puts loop { generated.next }

stopped = loop do
  raise StopIteration
end
puts stopped.inspect

endless = loop
puts endless.instance_of? Enumerator
puts endless.size

seen = 0
answer = endless.each do |*args|
  seen += 1
  break seen if seen >= 4
end
puts answer

pairs = Enumerator.new do |yielder|
  yielder.yield 1, 2
  yielder.yield 3, 4
end
p pairs.to_a
