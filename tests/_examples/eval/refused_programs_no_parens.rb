# Code the interpreter could read but Ruby's grammar refuses raises a
# SyntaxError from eval, laid out as prism lays it out.
def report source
  eval source
rescue SyntaxError => error
  puts error.message.sub /\A\(eval at [^)]*\)/, "(eval)"
end

report "1 2"
report "first = 1\nsecond = 2 3"
report "def pair(left, left); end"
total = 4
puts eval "total -2"
