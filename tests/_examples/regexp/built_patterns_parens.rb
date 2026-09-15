# `Regexp.new` reads flags written as a number or as the letters a literal
# carries, refuses a pattern Ruby refuses, and reports the encoding the
# pattern is read in.
p(Regexp.new("hi", Regexp::IGNORECASE).options)
p(Regexp.new("hi", "imx").options)
p(Regexp.new("hi").source.encoding)
p(Regexp.new("\u{3042}").encoding)

begin
  Regexp.new("hi", "e")
rescue ArgumentError => problem
  puts(problem.message)
end

begin
  Regexp.new("^[$")
rescue RegexpError => problem
  puts(problem.message)
end

begin
  Regexp.new(:symbol)
rescue TypeError => problem
  puts(problem.message)
end
