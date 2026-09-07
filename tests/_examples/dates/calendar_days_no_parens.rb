# A Date holds the Julian Day Number a calendar day stands for, together with
# the day the Gregorian calendar takes over from the Julian one.
require 'date'

landing = Date.civil 2008, 1, 16

p landing.jd
p landing.year
p landing.month
p landing.day
p landing.yday
p landing.wday
p landing.cwyear
p landing.cweek
p landing.cwday
p landing.mjd
p landing.ajd
p landing.leap?
p landing.wednesday?
p landing.to_s
p landing.inspect

p Date.jd(2454482) == landing
p Date.ordinal(2008, 16) == landing
p Date.commercial(2008, 3, 3) == landing
p Date.parse("2008-01-16") == landing
p Date.strptime("16/01/2008", "%d/%m/%Y") == landing
p Date.iso8601("20080116") == landing

later = landing + 30
earlier = landing - 30
p later.to_s
p earlier.to_s
p (landing >> 2).to_s
p (landing << 1).to_s
p landing.next_day.to_s
p landing.next_month.to_s
p landing.next_year.to_s
p Date.civil(2008, 3, 1) - Date.civil(2008, 2, 1)

p Date.leap? 2008
p Date.valid_civil? 2008, 2, 29
p Date.valid_civil? 2007, 2, 29
p Date.valid_ordinal? 2008, 366
p Date.valid_commercial? 2008, 52, 7

p landing.strftime "%A, %B %-d, %Y"
p landing.strftime "%a %b %e %Y"
p landing.strftime "%j %U %W %V"
p landing.strftime "%^10b"

# The reform of 1582 removed ten days from the Italian calendar, so the days
# either side of the gap sit next to each other.
reform = Date.civil 1582, 10, 4
crossed = reform + 1
p reform.to_s
p crossed.to_s
p reform.julian?
p crossed.gregorian?
p Date.civil(1582, 10, 4, Date::ENGLAND).julian?

walked = []
limit = Date.civil 2008, 1, 19
landing.upto limit do |day|
  walked.push day.day
end
p walked
p landing.step(Date.civil(2008, 1, 22), 2).map { |day| day.day }
