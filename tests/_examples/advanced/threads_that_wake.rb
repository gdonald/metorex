# A thread reports what it is doing, and a thread that waits on a condition
# variable is asleep until something signals it.
queue = Queue.new
waiting = Thread.new do
  queue.pop
end

Thread.pass until waiting.stop?
p(waiting.status)
queue.push(:ready)
p(waiting.value)
p(waiting.status)

# A lock a thread never let go of is released when the thread ends, so
# whoever waits for it next is not held up.
lock = Mutex.new
holder = Thread.new do
  lock.lock
end
holder.join
p(lock.locked?)

# A condition variable lets a thread sleep inside a lock it holds, and wakes
# it once something signals.
guard = Mutex.new
signal = ConditionVariable.new
heard = []
listener = Thread.new do
  guard.synchronize do
    signal.wait(guard)
    heard << :woken
  end
end

Thread.pass until listener.stop?
guard.synchronize do
  signal.signal
end
listener.join
p(heard)

# Stopping a thread runs the `ensure` blocks it is inside, and the thread
# holds the lock it was taking while they run.
notes = []
dying = Thread.new do
  guard.synchronize do
    begin
      signal.wait(guard)
    ensure
      notes << guard.owned?
    end
  end
end

Thread.pass until dying.stop?
dying.kill
dying.join
p(notes)
p(dying.status)
