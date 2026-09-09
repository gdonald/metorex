# A DateTime is a calendar day with a time of day and an offset from UTC. The
# time is held as an exact fraction of a day, so arithmetic loses nothing.
require 'date'

meeting = DateTime.new 2012, 12, 24, 13, 45, 30, "+03:00"

p meeting.year
p meeting.month
p meeting.day
p meeting.hour
p meeting.min
p meeting.sec
p meeting.sec_fraction
p meeting.offset
p meeting.zone
p meeting.to_s
p meeting.day_fraction

p meeting.new_offset("-05:00").to_s
p meeting.new_offset(0).to_s
p meeting.to_date.to_s
p meeting.to_date.class
p DateTime.new(2012, 12, 24).to_date.to_datetime.to_s

p meeting.strftime "%Y-%m-%d %H:%M:%S %z"
p meeting.strftime "%I:%M %p"
p meeting.strftime "%A, %-d %B %Y"
p meeting.strftime "%::z"
p DateTime.new(2001, 2, 3, 4, 5, 6.25).strftime("%H:%M:%S.%6N")
p DateTime.new(2001, 2, 3, 4, 5, 6.25).strftime("%L")

p DateTime.parse("2012-11-08T15:43:59").to_s
p DateTime.strptime("2012-11-08 15:43:59", "%Y-%m-%d %H:%M:%S").to_s
p DateTime.iso8601("2018-01-01").to_s

# A day is one unit, so a quarter of a day is six hours.
p (meeting + Rational(1, 4)).to_s
p (meeting - Rational(1, 2)).to_s
p (DateTime.new(2012, 1, 2) - DateTime.new(2012, 1, 1))
p (DateTime.new(2012, 1, 1, 6) - DateTime.new(2012, 1, 1))
p DateTime.new(2012, 1, 1, 6) > DateTime.new(2012, 1, 1)

# An hour of 24 names midnight on the day after.
rolled = DateTime.new 2012, 12, 24, 24
p rolled.day
p rolled.hour

moment = Time.utc 2012, 12, 31, 23, 58, 59
p moment.to_datetime.to_s
p moment.to_date.to_s
p DateTime.new(2012, 12, 31, 23, 58, 59).to_time.utc.to_s
