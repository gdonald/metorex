# A thread waiting on a lock, a sleep, a queue or a condition variable
# answers a backtrace that starts at the method it waits in.
def short(lines)
  lines.map { |line| line.sub(/\A.*\//, "") }
end

held = Mutex.new
held.lock
locker = Thread.new { held.lock }
Thread.pass until locker.stop?
p(short(locker.backtrace))

sleeper = Thread.new { sleep }
Thread.pass until sleeper.stop?
p(short(sleeper.backtrace))

queue = Queue.new
popper = Thread.new { queue.pop }
Thread.pass until popper.stop?
p(short(popper.backtrace))

signal = ConditionVariable.new
guard = Mutex.new
waiter = Thread.new { guard.synchronize { signal.wait(guard) } }
Thread.pass until waiter.stop?
p(short(waiter.backtrace))

begin
  guard.synchronize { raise("inside") }
rescue RuntimeError => raised
  p(short(raised.backtrace))
end

[sleeper, popper, waiter].each(&:kill)
held.unlock
locker.join
