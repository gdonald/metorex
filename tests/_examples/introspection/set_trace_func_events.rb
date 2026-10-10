# set_trace_func hands each line, call, return and raise to a proc with the
# event's name, file, line, method, binding and class, until it is set to nil.
events = []
tracer = proc do |event, file, line, id, binding, klass|
  next unless %w[line call return raise].include?(event)

  events << [event, File.basename(file), line, id, klass, binding.class]
end

def add_one(number)
  total = number + 1
  total
end

def refuse(reason)
  raise(ArgumentError, reason)
rescue ArgumentError
  :refused
end

p(set_trace_func(tracer).equal?(tracer))
add_one(1)
refuse("bad")
set_trace_func(nil)
add_one(2)
events.each { |event| p(event) }

begin
  set_trace_func(:not_a_proc)
rescue TypeError => error
  p(error.message)
end
