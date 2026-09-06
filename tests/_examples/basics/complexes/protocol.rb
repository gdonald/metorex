# `Complex::I` is the imaginary unit every other Complex is measured against.
p Complex::I
p Complex::I * Complex::I

# The numerator scales both parts to the denominator they share.
p Complex(Rational(7, 8), Rational(8, 4)).numerator
p Complex(Rational(3, 8), Rational(3, 4)).denominator

# Two Complexes with nothing on the imaginary axis order by their real parts,
# and anything else has no order at all.
p(Complex(5) <=> Complex(2))
p(Complex(5) <=> 2)
p(Complex(5, 1) <=> Complex(2))

# `eql?` compares the parts by class as well as by value.
p Complex(1, 2).eql? Complex(1, 2)
p Complex(1, 2).eql? Complex(1, 2.0)

# A Complex names no point on the number line, so it answers neither question
# a real number does.
p Complex(1, 2).respond_to? :positive?

# A number is not built by hand, and a clone cannot ask for an unfrozen one.
begin
  Float.new 1.0
rescue NoMethodError => error
  puts error.message
end

begin
  1.clone freeze: false
rescue ArgumentError => error
  puts error.message
end

# Numeric answers the parts of the fraction it stands for.
p 3.numerator
p 0.5.denominator
