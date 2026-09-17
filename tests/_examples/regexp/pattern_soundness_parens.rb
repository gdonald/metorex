# A pattern has to close every group it opens.
begin
  Regexp.new("(hay(st)ack")
rescue RegexpError => error
  puts(error.message)
end

begin
  Regexp.new("hay)stack")
rescue RegexpError => error
  puts(error.message)
end

# A group's name is a name, so it cannot open with a digit or a minus.
begin
  Regexp.new("(?<1a>a)")
rescue RegexpError => error
  puts(error.message)
end

begin
  Regexp.new("(?<-a>a)")
rescue RegexpError => error
  puts(error.message)
end

# Only the three letters that change how a pattern reads name an option to
# `Regexp.new`, whatever a literal may be written with.
begin
  Regexp.new("Hi", "n")
rescue ArgumentError => error
  puts(error.message)
end

p(Regexp.new("Hi", "mx").source)
