# The directive machine `pack` writes with and `unpack` reads back.
p([65, 66, 67].pack("C*"))
p("abc".unpack("C*"))
p("abc".unpack("C2"))
p("ab".unpack("v"))
p("ab".unpack("n"))
p("ab".unpack("S<"))
p("ab".unpack("S>"))
p("abcdefgh".unpack("Q<"))
p(["ab"].pack("a4").length)
p(["ab"].pack("A4"))
p("abc\x00def".unpack("Z*Z*"))
p("\x80".unpack("B*"))
p("\x8f".unpack("H2"))
p([300].pack("w").length)
p("abc".unpack1("C"))
p("abcd".unpack("C3X*C"))
p([1].pack("l!").length)
p([1].pack("i!").length)

# A character named by an escape.
p(?\001)
p(?\n)
p(?\x41)

# A trace is handed each event it asked for.
seen = []
tracer = TracePoint.new(:line) { |point| seen.push([point.event, point.lineno]) }
p(tracer.enabled?)
tracer.enable
answer = 1
tracer.disable
p(tracer.enabled?)
p(seen.length)
p(seen.first[0])

calls = []
def traced_method
  7
end
watcher = TracePoint.new(:call, :return) { |point| calls.push([point.event, point.method_id]) }
watcher.enable { traced_method }
p(calls)

p(Marshal::MAJOR_VERSION)
p(Marshal::MINOR_VERSION)
walk = 1.upto("A")
p(walk.class)
begin
  walk.size
rescue ArgumentError => problem
  p problem.message
end
p(5.upto(10).size)
