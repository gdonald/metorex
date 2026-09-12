# Two strings are equal when their bytes match and their encodings can be
# compared. Text that is nothing but ASCII goes with anything.
plain = "hello".dup.force_encoding("utf-8")
p plain == "hello".dup.force_encoding("iso-8859-1")
p plain.eql?("hello")
p plain === "hello"

# An array compares element by element, asking equal? before ==, so an array
# holding NaN equals another holding the same NaN.
p [Float::NAN] == [Float::NAN]
p [1, [2, 3]] == [1, [2, 3]]

# A Range reads a value with === the way cover? does.
p((1..5) === 3)
p((1...5).cover?(5))
p (1..5).send(:===, 6)

# The largest value of a range that leaves its end out is the whole number
# before it, and a non-integer end has none.
p (1...5).max
p (1..5).max
begin
  (1.0...5.0).max
rescue TypeError => error
  p error.message
end

# Two ranges overlap when they share a value. One holding nothing shares none.
p (1..5).overlap?(4..9)
p (1..5).overlap?(6..9)
p (2..0).overlap?(1..3)

# Every symbol the program has spelled is in the table, run or not.
p Symbol.all_symbols.include?(:spelled_but_never_run)
