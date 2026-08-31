puts sprintf("%s and %s", "one", "two")
puts sprintf("%d", 42)
puts format("%s", :symbol)

class Template
  def to_str
    "converted %s"
  end
end
puts sprintf(Template.new, "format")

begin
  sprintf(42, "value")
rescue TypeError => error
  puts "#{error.class}: #{error.message}"
end

begin
  x = 42 % "not a format"
  puts x.inspect
rescue TypeError => error
  puts error.class
end

puts Float::INFINITY.inspect
puts (Float::INFINITY > 1e308)
puts (-Float::INFINITY < -1e308)
puts (Float::NAN == Float::NAN)
puts Float::DIG
puts Float::MANT_DIG
puts (Float::EPSILON > 0)
puts (Float::MAX > Float::MIN)
