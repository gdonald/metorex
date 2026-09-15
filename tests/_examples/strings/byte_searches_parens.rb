# `byteindex` and `byterindex` answer where a needle sits, counted in bytes,
# reading from an offset forwards or backwards.
p("blablabla".byteindex("bla"))
p("blablabla".byterindex("bla"))
p("blablabla".byteindex("bla", 1))
p("blablabla".byterindex("bla", 5))
p("ありがとう".byteindex("が"))
p("ありがとう".byteindex(/が/))
p("blablabla".byteindex("zz"))
p("ありがとう".index("が"))
p("ありがとう".rindex("が"))
p("helloYOU.".byteindex(/\GYOU/, 5))

begin
  "わ".byteindex("", 1)
rescue IndexError => problem
  puts(problem.message)
end

begin
  "abc".byteindex(97)
rescue TypeError => problem
  puts(problem.message)
end
