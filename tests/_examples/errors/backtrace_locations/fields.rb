def inner
  raise "boom"
end

def outer
  inner
end

begin
  outer
rescue RuntimeError => error
  locations = error.backtrace_locations
  puts locations.size == error.backtrace.size
  puts locations[0].label
  puts locations[0].lineno
  puts locations[0].path.end_with? ".rb"
  puts locations[0].absolute_path.start_with? "/"
  puts locations[0].to_s == error.backtrace[0]
  puts locations.last.label
  puts locations[0].base_label
end

copy = RuntimeError.new "copied"
p copy.backtrace_locations

source = begin
  outer
rescue RuntimeError => error
  error.backtrace_locations
end

copy.set_backtrace source
puts copy.backtrace_locations.size == source.size
puts copy.backtrace_locations[0].label
puts copy.backtrace[0] == source[0].to_s
