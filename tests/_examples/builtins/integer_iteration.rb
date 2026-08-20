1.upto(3) do |n|
  puts n
end

3.downto 1 do |n|
  puts n
end

puts 1.upto(3).inspect
puts 3.downto(1).inspect

half = 3.quo(2)
puts half.to_s
puts half.to_i
puts half.to_f
puts half.numerator
puts half.denominator
puts Rational(8, 3).to_i
puts 4.quo(2).to_s
