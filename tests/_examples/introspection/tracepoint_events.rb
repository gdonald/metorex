class Greeter
  def greet(name, punctuation: "!")
    "hello #{name}#{punctuation}"
  end

  def fail_out
    raise ArgumentError, "no name given"
  end
end

seen = []

tracer = TracePoint.new(:call, :return) do |point|
  seen.push([point.event, point.method_id, point.defined_class.to_s, point.parameters])
end

tracer.enable
Greeter.new.greet("world", punctuation: "?")
tracer.disable

seen.each { |row| puts(row.inspect) }

raised = nil
watcher = TracePoint.new(:raise) do |point|
  raised = point.raised_exception
end

watcher.enable
begin
  Greeter.new.fail_out
rescue ArgumentError
end
watcher.disable

puts(raised.class)
puts(raised.message)

names = nil
scope = TracePoint.new(:return) do |point|
  names = point.binding.local_variables.sort
end
scope.enable
Greeter.new.greet("again")
scope.disable
puts(names.inspect)
