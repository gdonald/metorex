half = Rational(1, 2)
puts half
puts half.numerator
puts half.denominator
puts Rational(2, 4)
puts Rational(-3, -5)
puts Rational(7)
puts Rational 3, 9

puts 1/2r
puts 5r
puts 1.5r
puts 1/2r + 1/3r
puts 1/2r * 2/3r
puts 1/2r - 1/4r
puts((1/2r) / (1/4r))
puts 1/2r < 2/3r
puts half.to_f
puts Rational(8, 3).to_i
puts half.inspect
puts Rational(1).frozen?

puts Rational(".52")
puts Rational(".52", ".6")
puts Rational("3/4")
puts "0.6".to_r
puts 3.to_r
puts 0.5.to_r
puts (Rational(1) == Rational(1, 1))
puts (Rational(1) == 1)
puts Rational(1).eql?(1)

begin
  Rational(1, 0)
rescue ZeroDivisionError => e
  puts e.message
end

begin
  Rational(nil)
rescue TypeError => e
  puts e.message
end

puts Rational("abc", exception: false).inspect
