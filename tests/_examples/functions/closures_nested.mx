def make_multiplier(factor)
  lambda do |value|
    factor * value
  end
end

double = make_multiplier(2)
triple = make_multiplier(3)

puts double.call(5)
puts triple.call(4)
