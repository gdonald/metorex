result = -> { 5 }.call == 5
p result
p(-> (value) { value * 2 }.call(3) == 6)
p -> { :sym }.call == :sym
p((-> { 1 }.call + 1) == 2)
