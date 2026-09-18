# -*- encoding: us-ascii -*-
# A heredoc written in a source of its own encoding carries that encoding,
# interpolation and all.
who = "world"
greeting = <<HERE
hello #{who}
HERE
puts(greeting)
puts(greeting.encoding)

# A backslash at the end of a line joins it to the next.
joined = <<~HERE
  a
  b\
  c
HERE
puts(joined.inspect)

# A quoted terminator has to close on the line it opens.
begin
  eval %{<<"HERE\n"\nno terminator\nHERE}
rescue SyntaxError
  puts("SyntaxError")
end

# A name read inside a heredoc reports the line it was written on.
begin
  <<-HERE.chomp
    a
    #{missing_name}
  HERE
rescue NameError => refused
  puts(refused.backtrace[0].split(":")[-2])
end
