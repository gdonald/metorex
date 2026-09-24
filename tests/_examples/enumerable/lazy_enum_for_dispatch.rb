class Enumerator::Lazy
  def doubled_pairs(offset)
    return to_enum(:doubled_pairs, offset) unless block_given?
    each { |value| yield value * 2, offset }
  end
end

numbers = (1..Float::INFINITY).lazy
paired = numbers.doubled_pairs(7)
puts paired.class
p paired.first(3)
p paired.map { |doubled, offset| doubled + offset }.first(2)

indexed = numbers.enum_for(:each_with_index)
puts indexed.class
p indexed.first(2)
p indexed.select { |value, index| index.even? }.first(2)

sized = numbers.enum_for(:each_slice, 3) { 99 }
puts sized.size
p sized.first(2)

(1..7).lazy.each_slice(3) { |slice| p slice }
(1..3).lazy.each_with_index { |value, index| p [value, index] }
