# `cycle` walks the elements over and over. A count says how many rounds,
# and the first round hands out the elements as it reads them.
seen = []
[1, 2, 3].cycle 2 do |element|
  seen.push element
end
p seen

stopped = []
[10, 20, 30].cycle do |element|
  stopped.push element
  break if element == 20
end
p stopped

p [1, 2, 3, 4].cycle(2).size
p [1, 2].cycle.size
p [1, 2, 3].cycle(0) { |element| raise "never" }
