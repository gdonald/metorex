# Array#fill writes over a stretch: everything, from an index on, a run of a
# given length, or the span a Range names.
p [1, 2, 3, 4].fill(:x)
p [1, 2, 3, 4].fill(:x, 2)
p [1, 2, 3, 4].fill(:x, 1, 2)
p [1, 2, 3, 4].fill(:x, 1..2)
p [1, 2, 3, 4].fill { |index| index * 2 }
p [1, 2, 3, 4].fill(2) { |index| index * 10 }

begin
  [1, 2].fill(:x, "two")
rescue TypeError => refused
p refused.message
end

# Array#hash comes from what the array holds, and a list holding itself is
# answered from its length rather than followed forever.
looping = []
looping << looping
p looping.hash == [looping].hash
p [1, 2].hash == [1, 2].hash
p [1, 2].hash == [2, 1].hash
