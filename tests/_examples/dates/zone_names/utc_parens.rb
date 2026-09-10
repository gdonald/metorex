# A zone written as UTC, as Z, or as a negative zero offset names UTC itself,
# and the name a time reports is written in ASCII.
p(Time.utc(2022).zone)
p(Time.new(2022, 1, 1, 0, 0, 0, "UTC").zone)
p(Time.new(2022, 1, 1, 0, 0, 0, "Z").zone)
p(Time.new(2022, 1, 1, 0, 0, 0, "-00:00").zone)
p(Time.new(2022, 1, 1, 0, 0, 0, in: "UTC").zone)
p(Time.at(Time.utc(2022), in: "Z").zone)
p(Time.utc(2022).zone.encoding)
p(Time.utc(2022).localtime("UTC").utc?)
p(Time.new(2022, 1, 1, 0, 0, 0, "+05:00").zone)
