# Every BEGIN body runs before the rest of the file it was written in, in the
# order they were written.
BEGIN { $order = ["first"] }
BEGIN { $order << "second" }
$order << "body"
puts $order.inspect

# The body leaves its locals behind for the rest of the unit.
BEGIN { written_in_begin = "held" }
puts written_in_begin

# BEGIN belongs to the top level of a code unit.
begin
  eval "1.times { BEGIN { 1 } }"
rescue SyntaxError
  puts "SyntaxError"
end

# Code handed to eval is its own unit, so its BEGIN runs before the rest of
# the string.
puts eval("$seen = 'from eval'; BEGIN { $seen = 'from begin' }; $seen")
