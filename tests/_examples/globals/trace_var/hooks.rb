# Kernel#trace_var runs a hook whenever the named global is assigned.

$traced = 0

trace_var :$traced, proc { |value| puts "proc saw #{value}" }
$traced = 1

untrace_var :$traced
$traced = 2

trace_var :$watched do |value|
  puts "block saw #{value}"
end
$watched = "here"
untrace_var :$watched

recorded = []
hook = proc { |value| recorded.push value }
trace_var "$counted", hook
$counted = 10
$counted = 20
untrace_var :$counted, hook
$counted = 30
puts recorded.inspect

trace_var :$evaluated, "puts 'string hook ran'"
$evaluated = 1
untrace_var :$evaluated

begin
  trace_var :$missing
rescue ArgumentError => error
  puts error.message
end
