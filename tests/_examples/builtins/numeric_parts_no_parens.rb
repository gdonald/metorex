# The readings a real number answers about where it sits on the complex plane.
p 3.i
p 3.to_c
p 5.polar
p (-5).polar
p 5.rect
p (-4).arg == Math::PI
p 4.angle
p 4.phase
p (-3).abs2

p 5.quo(2)
p 2.quo(2.5)
p 13.remainder(4)
p (-13).remainder(4)
p 13.remainder(-4)
p 100.send(:-@)

p Rational(5404319552844595, 18014398509481984).rationalize(Rational(1, 10))
p 0.3.rationalize(0.001)
p (3382729202.92822).rationalize

p Complex(1, 2) + Complex(3, 4)
p Complex(20, 40) / 0.0
p Complex(3, 9).fdiv(1.5)
p Complex(3, 9) / 3

p [Float::RADIX, Float::MAX_EXP, Float::MIN_EXP]
nan = 0.0 / 0.0
p (nan <=> 1.0)
p (1.0 <=> nan)
