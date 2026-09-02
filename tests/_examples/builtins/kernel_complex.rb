c = Complex(3, 4)
puts c.real
puts c.imaginary
puts c.to_s
p c

puts Complex(1, 2) == Complex(1, 2)
puts Complex(5) == 5
puts Complex(1, 0) == 1.0

puts Complex("20") == Complex(20)
puts Complex("-3") == Complex(-3)
puts Complex("2/3") == Complex(Rational(2, 3))
puts Complex("4+2.3i") == Complex(4, 2.3)
puts Complex("35i") == Complex(0, 35)
puts Complex("i") == Complex(0, 1)
puts Complex("-i") == Complex(0, -1)
puts Complex("79-i") == Complex(79, -1)
puts Complex("79+4J") == Complex(79, 4)
puts Complex("2e3+2e4i") == Complex(2e3, 2e4)
puts Complex("7_9+4_0i") == Complex(79, 40)
puts Complex("  79+4i  ") == Complex(79, 4)

puts Complex(Complex(3, 4), Complex(5, 6)) == Complex(3 - 6, 4 + 5)
puts Complex(Complex(1, 2)) == Complex(1, 2)

begin
  Complex("ruby")
rescue ArgumentError => error
  puts error.message
end

begin
  Complex(nil)
rescue TypeError => error
  puts error.message
end

p Complex("ruby", exception: false)
puts Complex(1).frozen?
