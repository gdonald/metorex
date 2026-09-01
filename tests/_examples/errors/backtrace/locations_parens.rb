# `Exception#backtrace_locations` answers Thread::Backtrace::Location objects,
# nil until the exception is raised, and the same Array on every call.

unraised = RuntimeError.new("never raised")
puts(unraised.backtrace_locations.inspect)

begin
  raise("raised here")
rescue RuntimeError => error
  locations = error.backtrace_locations
  puts(locations.class.to_s)
  puts(locations.first.instance_of?(Thread::Backtrace::Location).to_s)
  puts(locations.equal?(error.backtrace_locations).to_s)
  puts(locations.first.path.end_with?(".rb").to_s)
  puts((locations.first.lineno > 0).to_s)
end

# `each_with_index` yields the element and its position.
labelled = []
["a", "b"].each_with_index do |value, index|
  labelled.push("#{index}:#{value}")
end
puts(labelled.inspect)
single = ["only"]
puts(single.each_with_index.class.to_s)
