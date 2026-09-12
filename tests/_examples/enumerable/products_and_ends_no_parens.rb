# Every combination of several walks, the two ends of a range at once, and a
# walk with no end zipped against one that has a length.
walk = Enumerator.product 1..2, ["a", "b"]
p walk.class
p walk.to_a
p walk.size

endless = Enumerator.product 1.., ["x"]
p endless.first 3

p (3..9).minmax
p (9..3).minmax
p (1..4).minmax { |left, right| right <=> left }

p [1, 2].zip(10.upto(Float::INFINITY))
p 10.upto(Float::INFINITY).first 3

p [[:a, 1], [:b, 2]].to_h
begin
  [:a].to_h
rescue TypeError => refused
  p refused.message
end

p [7, 6, 5, 4].slice_before { |value| value.even? }.to_a

letters = %w[a b c]
p letters.each.with_index(1.9).to_a
