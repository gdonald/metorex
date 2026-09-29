def twice
  first = yield
  second = yield
  [first, second]
end

line_trace = TracePoint.new(:line) { |tp| }
line_trace.enable
p twice { 3 }
line_trace.disable

class Report
  def build
    1.times do
      [2, 1].max
    end
    summarize
  end

  def summarize
    :done
  end
end

report = Report.new
events = []
trace = TracePoint.new :call, :return, :b_call, :c_call, :c_return do |tp|
  events << [tp.event, tp.method_id]
end
trace.enable target: report.method(:build) do
  report.build
end
p events

lines = []
target = proc do
  total = 1
  total += 2
  total * 3
end
_, opened_on = target.source_location
line_trace = TracePoint.new(:line) { |tp| lines << tp.lineno - opened_on }
line_trace.enable target: target, target_line: opened_on + 2 do
  target.call
end
p lines

order = []
outer = TracePoint.new(:b_call) { |tp| order << :outer }
inner = TracePoint.new(:b_call) { |tp| order << :inner }
aimed = -> {}
outer.enable target: aimed do
  inner.enable target: aimed do
    aimed.call
  end
end
p order

refusals = [
  -> { TracePoint.new(:call) {}.enable(target: proc {}) {} },
  -> { TracePoint.new(:call) {}.enable(target: Object.new) {} },
  -> { TracePoint.new(:line) {}.enable(target_line: 3) {} },
  -> { TracePoint.new(:call) {}.enable(target: -> { 1 }, target_line: 1) {} },
  -> { TracePoint.new(:line) {}.enable(target: -> { 1 }, target_line: Object.new) {} },
  -> { TracePoint.new(:line) {}.enable(target: -> {}, target_line: 1) {} },
]
refusals.each do |attempt|
  begin
    attempt.call
  rescue ArgumentError, TypeError => error
    puts "#{error.class}: #{error.message}"
  end
end

nested = TracePoint.new(:b_call) {}
begin
  nested.enable target: -> {} do
    nested.disable {}
  end
rescue ArgumentError => error
  puts error.message
end

inspections = []
TracePoint.new :call, :c_call, :line do |tp|
  inspections << tp.inspect.sub(__FILE__, "FILE") if inspections.size < 3
end.enable do
  report.summarize
end
puts inspections
puts TracePoint.new(:line) {}.inspect

threads_seen = []
worker = nil
TracePoint.new :thread_begin, :thread_end do |tp|
  threads_seen << tp.event if Thread.current == worker
end.enable(target_thread: nil) do
  worker = Thread.new {}
  worker.join
end
p threads_seen

heard = []
current_only = TracePoint.new(:line) { |tp| heard << Thread.current }
current_only.enable do
  Thread.new { :other }.join
end
p heard.uniq == [Thread.current]
