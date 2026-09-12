# A `for` loop runs in the scope holding it, so the names it and its body bind
# are still readable once it has run.
for number in 1..3
  doubled = number * 2
end
p([number, doubled])

pairs = [[1, 2], [3, 4]]
for left, right in pairs
  p([left, right])
end

for first, in pairs
  p(first)
end

for head, *tail in [[1, 2, 3]]
  p([head, tail])
end

class Holder
  attr_accessor :seen
end

holder = Holder.new
for holder.seen in [7, 8]
end
p(holder.seen)

collected = []
for value in [1, 2, 3]
  next if value == 2
  collected.push(value)
end
p(collected)

p((for value in 1..3 do end))
p((for value in 1..3 do break 10 end))

counted = 0
turns = 0
for value in 1..3
  counted += 1
  turns += 1
  redo if value == 2 && turns < 5
end
p([counted, turns])
