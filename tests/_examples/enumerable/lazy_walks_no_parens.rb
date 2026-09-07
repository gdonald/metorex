# A lazy walk applies its steps one element at a time, so a source with no
# end still answers first and force.
counting = (1..Float::INFINITY).lazy

p counting.map { |number| number * 2 }.first 4
p counting.select { |number| number.even? }.first 3
p counting.reject { |number| number.even? }.first 3
p counting.filter_map { |number| number * 3 if number.odd? }.first 3
p counting.take(4).force
p counting.take_while { |number| number < 5 }.force
p counting.drop(3).first 3
p counting.drop_while { |number| number < 4 }.first 3
p counting.with_index(10).first 3
p counting.zip([:a, :b, :c]).first 3

p [1, nil, 2, nil].lazy.compact.force
p [1, 1, 2, 2, 3].lazy.uniq.force
p [1, 2, 3, 4].lazy.grep(2..3).force
p [1, 2, 3, 4].lazy.grep_v(2..3).force
p [[1, 2], [3]].lazy.flat_map { |pair| pair }.force

p [1, 2, 4, 5].lazy.chunk_while { |left, right| right == left + 1 }.force
p [1, 2, 4, 5].lazy.slice_when { |left, right| right != left + 1 }.force
p [1, 2, 3, 4].lazy.slice_before { |number| number.odd? }.force
p [1, 2, 3, 4].lazy.slice_after { |number| number.even? }.force

sized = (1..10).lazy
p sized.size
p sized.map { |number| number }.size
p sized.take(4).size
p sized.drop(4).size
p sized.select { |number| number.odd? }.size

p [1, 2].lazy.eager.class
p [1, 2].lazy.lazy.equal?([1, 2].lazy.lazy) == false

chained = [1, 2].chain [3], 4..5
p chained.to_a
p chained.size
p chained.inspect
p ([1].each + [2].each).to_a
