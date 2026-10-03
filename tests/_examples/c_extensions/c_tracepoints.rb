# A C extension making a TracePoint whose hook is a C function, and turning
# TracePoints on and off.
require "tmpdir"
require_relative "build_helper"

def traced
  1
end

directory = Dir.mktmpdir
require(build_extension("c_tracepoints.c", "c_tracepoints", directory))
tracepoints = CTracepoints.new

seen = []
calls = tracepoints.trace_new(CTracepoints::CALL | CTracepoints::RETURN, seen)
p(calls.class)
p(calls.enabled?)
p(tracepoints.enable(calls))
traced
p(tracepoints.disable(calls))
traced
p(seen)

lines = []
line_trace = tracepoints.trace_new(CTracepoints::LINE, lines)
line_trace.enable
total = 1 + 1
line_trace.disable
p(lines.uniq)

nothing = []
silent = tracepoints.trace_new(CTracepoints::NONE, nothing)
silent.enable
traced
silent.disable
p(nothing)

FileUtils.rm_rf(directory)
