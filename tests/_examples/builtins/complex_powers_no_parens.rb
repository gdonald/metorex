# A complex number raised to a power that is not a whole number, worked in
# polar form, and one raised by an object that coerces the pair.
p Complex(2, 1)**2
p Complex(3, 4)**0.0
p (Complex(3, 4)**2.5).rectangular.map { |part| part.round 6 }
p (Complex(2, 1)**Rational(3, 4)).rectangular.map { |part| part.round 6 }
p (Complex(2, 1)**Complex(2, 1)).rectangular.map { |part| part.round 6 }

class TwoAndFive
  def coerce(other)
    [2, 5]
  end
end

p Complex(3, 9)**TwoAndFive.new
