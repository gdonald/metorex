# An enumerator over pairs, such as `each_with_index` makes, yields each pair
# as two values: `map` and `each` spread them for the block, and `select`
# hands them over as one Array.
p [1, 2].each_with_index.map { |*pair| pair }
p [1, 2].each_with_index.map { |first| first }
p [1, 2].each_with_index.map { |item, index| item * index }
p [1, 2].each_with_index.select { |*packed| packed.size == 1 }
p [1, 2].each_with_index.select { |pair| pair[1] == 1 }
[1, 2].each_with_index.each { |*pair| p pair }
p [1, 2].each_with_object([]).map { |*pair| pair }
p [1, 2].each.with_index.map { |*pair| pair }
p [1, 2].each.with_index.select { |item, index| index.zero? }
begin
  [1, 2].each_with_index.map(&:to_s)
rescue ArgumentError => error
  p error.message
end
