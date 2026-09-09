# Integer bit operations: shifts, complement, and bit_length.

shifts = [1 << 3, 1 << 35, 8 >> 2, -8 >> 2]
puts shifts.inspect

# A negative count shifts the other way.
reversed = [5 << -1, 5 >> -1]
puts reversed.inspect

# A left shift is exact however wide the result grows, and a right shift past
# the width leaves 0, or the sign bit for a negative value.
widened = [1 << 200, -1 >> 200, 1 >> 200]
puts widened.inspect

negative_one = -1
complements = [~5, ~0, ~negative_one]
puts complements.inspect

lengths = [255.bit_length, 256.bit_length, 0.bit_length, negative_one.bit_length]
puts lengths.inspect

masks = [1 & 3, 1 | 2, 1 ^ 3]
puts masks.inspect

round_trip = (1 << 35) * 42 / (1 << 35)
puts round_trip.to_s
puts 42.equal?(round_trip).to_s
