# The two maps ObjectSpace carries, and what the object space can say about
# the size of an object.
require 'objspace'

map = ObjectSpace::WeakMap.new
p ObjectSpace::WeakMap.include? Enumerable
p map.size

first = Object.new
second = Object.new
map[first] = "one"
map[second] = "two"
p map.size
p map[first]
p map.key? second
p map.values.sort
# `key` looks its value up by identity, so a fresh literal holding the same
# text is a different value and finds nothing.
p map.key("two")
p map.key(map[second]).equal? second

seen = []
map.each do |key, value|
  seen.push value
end
p seen.sort

names = []
map.each_key do |key|
  names.push key.class.name
end
p names

p map.delete(first)
p map.size
p(map.delete(first) { |key| "gone" })

# A weak-key map compares its keys by value, and refuses a key that lives for
# the whole run.
keyed = ObjectSpace::WeakKeyMap.new
keyed["a".upcase] = 1
p keyed["a".upcase]
p keyed.key? "A"
p keyed.getkey "A"
p keyed.size
keyed["b".upcase] = 2
p keyed.size
p keyed.delete "B"
p keyed.clear.size

begin
  keyed[42] = "x"
rescue ArgumentError => error
  p error.message
end
p keyed[42]
p keyed.key? :a
p keyed.getkey nil

p ObjectSpace.memsize_of nil
p ObjectSpace.memsize_of 42
p ObjectSpace.memsize_of :name
p ObjectSpace.memsize_of(Object.new) > 0
