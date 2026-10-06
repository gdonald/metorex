# String literals written next to each other are one string, with any
# interpolation in them kept.
count = 3
p("a" "b")
p("a" "#{count}")
p("#{count}" "b", count)
p("a" "#{count}" 'c', count)
p('x' "#{count * 2}" "y" "#{count}")
long = "first part, " \
  "second part #{count}"
p(long)
