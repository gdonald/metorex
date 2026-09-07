# A Time holds a whole number of seconds since the epoch plus an exact
# fraction of a second, and reads its calendar fields in UTC or in a zone.
moment = Time.utc 2007, 11, 1, 15, 25, 30, 123456

p moment.year
p moment.month
p moment.day
p moment.hour
p moment.min
p moment.sec
p moment.wday
p moment.yday
p moment.usec
p moment.nsec
p moment.subsec
p moment.utc?
p moment.zone
p moment.utc_offset

p moment.to_i
p moment.to_f
p moment.to_r
p moment.to_a

p moment.inspect
p moment.to_s
p moment.asctime
p moment.xmlschema
p moment.xmlschema 3
p moment.strftime "%Y/%m/%d %H:%M:%S"
p moment.deconstruct_keys [:year, :month, :day]

p Time.at(100, 100).to_f
p Time.at(Rational(3, 2)).subsec
p Time.at(10.75).subsec
p (Time.at(100) - Time.at(99))
p (Time.at(100) + 1).to_i
p Time.at(100) == Time.at(100)
p Time.at(100) < Time.at(200)
p Time.at(100).eql? Time.at(100)

p Time.utc(2000, 1, 1).monday?
p Time.utc(2000, 1, 1).saturday?
p Time.utc(1985, 4, 12, 23, 20, 50).round.to_i
p Time.utc(1985, 4, 12, 23, 20, 50).floor.to_i

offset_time = Time.new 2010, 4, 4, 1, 59, 59, 7245
p offset_time.utc_offset
p offset_time.zone
p offset_time.hour
