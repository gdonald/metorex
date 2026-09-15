# A StringIO is opened over a buffer in a mode written as a string or as the
# flags File names, and a frozen buffer can only be read.
require "stringio"

held = StringIO.new("example", "r")
p(held.closed_read?)
p(held.closed_write?)

buffer = +"example"
numbered = StringIO.new(buffer, IO::WRONLY)
p(numbered.closed_read?)
p(numbered.closed_write?)

emptied = +"gone"
StringIO.new(emptied, "w")
p(emptied)

begin
  StringIO.new("frozen".freeze, "w")
rescue Errno::EACCES => problem
  puts(problem.class)
end

opening = +"first"
stream = StringIO.new(opening)
replacement = +"second"
stream.reopen(replacement)
p(stream.string)
