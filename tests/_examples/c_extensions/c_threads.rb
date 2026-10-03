# A C extension working with threads: the thread running now and its
# locals, a thread made around a C function, waking and waiting, and a C
# function run without the interpreter while the other threads take turns.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require(build_extension("c_threads.c", "c_threads", directory))
threads = CThreads.new

p([threads.alone, threads.current.equal?(Thread.current)])
threads.local_set(Thread.current, :written_from_c, 1)
Thread.current[:written_from_ruby] = 2
p([Thread.current[:written_from_c], threads.local_get(Thread.current, :written_from_ruby), threads.local_get(Thread.current, :unset)])

made = threads.create(->(value) { value * 2 }, 21)
p([made.class, made.value])
failing = threads.create(->(_) { Thread.current.report_on_exception = false; raise "failed in the thread" }, nil)
report { failing.join }

ran = false
other = Thread.new { ran = true }
threads.wait_for(20_000)
other.join
p(ran)

sleeper = Thread.new do
  sleep
  :woken
end
Thread.pass until sleeper.stop?
p([threads.wakeup(sleeper).equal?(sleeper), sleeper.value])
report { threads.wakeup(sleeper) }

p([threads.native_here, threads.native_elsewhere])
p(threads.sum_without_lock(40, 2))

reader = Thread.new { threads.blocking_read }
Thread.pass until reader.stop?
p(reader.status)
reader.wakeup
p(reader.value)

interrupted = Thread.new { threads.interrupted_read }
Thread.pass until interrupted.stop?
interrupted.wakeup
p(interrupted.value)

main = Thread.current
signalled = false
interrupter = Thread.new do
  Thread.pass until main.stop?
  previous = Signal.trap(:HUP) { signalled = true }
  begin
    Process.kill(:HUP, Process.pid)
    sleep(0.001) until signalled
  ensure
    Signal.trap(:HUP, previous)
  end
end
p(threads.blocking_read)
interrupter.join
p(signalled)

FileUtils.rm_rf(directory)
