# Forms Ruby's grammar does not accept raise SyntaxError when the code is
# read, and a range that ends its line takes its end from the next line.
[
  "->(x) x + 1",
  "-> 42",
  "add_ten = x -> x + 10",
  "add = (x, y) -> x + y",
  "def skipped *\nend",
  "def skipped **\nend",
  "def skipped &\nend",
  "match 1\nwhen 1\nend",
  "-> { |x| x }",
  "-> do |x| x end",
  "->(x) { x + 1 }",
  "def kept *; end",
  "def kept *\nrest\nend"
].each do |source|
  eval source
  puts "read     #{source.inspect}"
rescue SyntaxError
  puts "refused  #{source.inspect}"
end

span = 1..
  2
p span
p (3..
)
