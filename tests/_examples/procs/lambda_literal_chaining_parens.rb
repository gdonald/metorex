puts(-> (value) { value * 2 }.call(3))
puts(->(value) { value * 2 }.call(4))
puts(-> value { value * 2 }.call(5))
puts(-> *values { values.last }.call(6, 7))
puts(-> value do value * 2 end.call(8))
puts(-> { 9 }.call)

def forwards(*args)
  -> *inner { inner.last }.call(*args)
end

puts(forwards(10, 11))
puts(-> (value) { value }.lambda?)
puts(-> (first, second) { first }.arity)
