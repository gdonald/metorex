# A Rational raised to a power stays exact where the answer is a fraction,
# reads as a Float where no fraction stands for it, and turns around the
# circle into a complex number where the base is negative.
p Rational(3, 4) ** 4
p Rational(3, 4) ** -4
p Rational(3, 4) ** 0
p Rational(3, 4) ** Rational(2, 1)
p Rational(3, 4) ** Rational(-1, 1)
p (Rational(3, 4) ** Rational(4, 3)).round(12)
p (Rational(-3, 4) ** Rational(-4, 3)).rectangular.map { |part| part.round(12) }
p Rational(3, 1) ** 3.0
p Rational(1) ** (2 ** 70)
p Rational(-1) ** (2 ** 70)

# Nothing raised to a negative power has no answer.
begin
  Rational(0, 1) ** -1
rescue ZeroDivisionError => problem
  p problem.message
end

# An answer too wide to build is refused rather than attempted.
begin
  Rational(2) ** (2 ** 70)
rescue ArgumentError => problem
  p problem.message
end
