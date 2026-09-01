# An Integer grows past the machine word rather than saturating or turning
# into a Float. Ruby draws no line between the two sizes.

powers = [2 ** 64, 2 ** 100]
puts(powers.inspect)

literal = 18446744073709551616
puts(literal.class.to_s)
puts((literal == 2 ** 64).to_s)

sums = [9223372036854775807 + 1, 9223372036854775807 * 2, -9223372036854775807 - 2]
puts(sums.inspect)

big = 2 ** 64
round_trip = big * 42 / big
puts(round_trip.to_s)
difference = big - big
puts(difference.to_s)
puts(difference.class.to_s)

puts((big > 2 ** 63).to_s)
puts((big <=> 2 ** 63).to_s)
mixed = [big, 1, -big, 3]
puts(mixed.sort.inspect)

puts(big.abs.to_s)
negative = -big
puts(negative.abs.to_s)
puts(big.even?.to_s)
puts(big.succ.to_s)
puts(big.bit_length.to_s)
puts(big.divmod(1000).inspect)
puts((big >> 32).to_s)
puts((~big).to_s)

truncated = 2e100.to_i
puts(truncated.class.to_s)
puts(Integer("340282366920938463463374607431768211456").to_s)

ratio = Rational(big, 3)
puts(ratio.numerator.to_s)
puts(ratio.denominator.to_s)

# Two separately built values of the same size are separate objects.
first = 2e100.to_i
second = 2e100.to_i
puts((first == second).to_s)
puts(first.equal?(second).to_s)
puts(big.frozen?.to_s)
