# A jump reads as a value wherever one goes, and a loop answers what a break
# carried out of it.
kept = [1, nil, 2].map { |held| held or next(0) }
p(kept)

p((break 123 while true))
p((break until false))

counted = 0
answer = while counted < 3
  counted = counted + 1
end
p(answer)

# A `when` with nothing under it answers nil, and so does an empty `else`.
p((case 1
when 1
when 2
end))

p((case "c"
when "a" then "a"
else
end))

# A pattern stands where a `when` value goes.
p((case "hello"
when /abc/ then false
when /^hell/ then true
end))

# A definition read as a value answers the name it defined.
named = def doubled(number)
  number * 2
end
p(named)
p(doubled(21))
