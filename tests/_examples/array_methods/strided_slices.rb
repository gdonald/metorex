# An arithmetic sequence subscript takes every nth element of a run, reading
# the bounds the way a range does and refusing a span wider than the array.
held = [0, 1, 2, 3, 4, 5]
p held[(0..5).step(2)]
p held[(1..).step(2)]
p held[(..4).step(3)]
p held[(0...5).step(2)]
p held[(5..0).step(-2)]
p held[(5..).step(-2)]
p held[(6..).step(2)]

begin
  held[(0..6).step(2)]
rescue RangeError => problem
  puts problem.message
end
