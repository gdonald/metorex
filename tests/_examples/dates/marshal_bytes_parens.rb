# `Time#_dump` writes the eight bytes Marshal stores a time as: the date and
# hour in one word, the rest of the clock in another, both read in UTC.
stamp = Time.at(946812800).gmtime
written = stamp.send(:_dump)
p(written.bytes)

high, low = written.unpack("VV")
marker = high >> 31
in_utc = (high >> 30) & 1
year = ((high >> 14) & 0xffff) + 1900
minute = low >> 26
second = (low >> 20) & 0x3f
p(marker)
p(in_utc)
p(year)
p(minute)
p(second)

local = Time.at(946812800)
local_high = local.send(:_dump).unpack("VV").first
local_utc = (local_high >> 30) & 1
p(local_utc)
