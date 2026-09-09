# Switches the core classes keep, and the timestamp forms Time reads.
require "time"
require "matrix"

p(GC.stress)
GC.stress = true
p(GC.stress)
GC.stress = false
p(GC.enable)
p(GC.disable)
p(GC.disable)
p(GC.auto_compact)
p(GC.measure_total_time)
p(GC::Profiler.enabled?)
GC::Profiler.enable
p(GC::Profiler.enabled?)
GC::Profiler.disable

queue = Queue.new
p(queue.closed?)
p(queue.close.equal?(queue))
p(queue.closed?)
begin
  queue.push(1)
rescue ClosedQueueError => problem
  p problem.class
end
begin
  Queue.new.freeze
rescue TypeError => problem
  p problem.message.start_with?("cannot freeze")
end
p(Thread::Queue.equal?(::Queue))
p(ConditionVariable.new.respond_to?(:marshal_dump))

p(Time.iso8601("1985-04-12T23:20:50.52Z").subsec)
p(Time.iso8601("1990-12-31T23:59:60Z").to_s)
p(Time.rfc2822("26 Aug 76 14:30 EDT").to_s)
p(Time.rfc2822("Fri, 21 Nov 1997 09 :   55  :  06 -0600").to_s)

p(Math.lgamma(-1))
p(Math.lgamma(-2))
p((-8.0) ** (1.0 / 3))
p(8.0 ** Rational(1, 2))

p(Matrix[[7, 8, 9], [14, 46, 51], [28, 82, 163]].lup.determinant == 15120)
p(Matrix[[7, 8, 9], [14, 46, 51], [28, 82, 163]].lup.solve(Vector[14, 55, 29]))

p(Thread.ignore_deadlock)
p(Process.ppid > 0)
