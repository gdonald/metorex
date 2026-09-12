# Every string the interpreter compiles fires a `:script_compiled` event, and
# the trace reads the source back out of it.
compiled = []

trace = TracePoint.new(:script_compiled) do |event|
  compiled << event.eval_script
end

trace.enable do
  eval("def greeting\n  'hello'\nend\n")
  eval("1 + 1")
end

p(compiled)
p(greeting)
