# A time may be built from its calendar parts, with a numeral spelled out
# reading as base ten and the month named in words or in digits.
puts Time.utc(2000, 1, 1, 20, 15, 1).to_s
puts Time.utc("2000", "08", "08", "08", "08", "08").to_s
puts Time.utc(2000, "dec").to_s

# Ten arguments name the parts the way the C library writes them, seconds
# first and the year sixth.
puts Time.utc(1, 15, 20, 1, 1, 2000, 0, 0, false, "UTC").to_s

# A count of microseconds stands for the whole fraction of the second.
made = Time.utc(2000, 1, 1, 20, 15, 1.75, 2)
puts made.sec.to_s
puts made.usec.to_s

begin
  Time.utc(2008, 16, 31)
rescue ArgumentError => problem
  puts problem.message
end
