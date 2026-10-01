# The code inside #{...} is a run of statements whose value is the last
# one's, so a modifier, a semicolon, and a line break mean there what they
# mean anywhere else.
count = 1
p("keyword#{"s" if count > 1}")
p("#{count += 1; count * 10}")
p("total: #{
  doubled = count * 2
  doubled + 1
}")
p("#{"inner #{count}" unless count.zero?}")
p("#{begin; raise "no"; rescue; "rescued"; end}")
p(:"symbol#{1 if true}")
