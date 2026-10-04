# A thread's scheduler takes the waits of its non-blocking fibers: a lock
# another fiber holds, an empty queue, a thread still running, a pipe with
# nothing to read, a sleep, a child process and a time limit. The thread
# closes its scheduler as it ends, which runs what the scheduler still holds.

class Loop
  attr_reader :calls

  def initialize
    @calls = []
    @ready = []
    @readers = {}
    @blocked = 0
  end

  def fiber &block
    @calls << :fiber
    made = Fiber.new blocking: false, &block
    made.resume
    made
  end

  def block blocker, timeout = nil
    @calls << [:block, blocker.class]
    @blocked += 1
    Fiber.yield
    true
  end

  def unblock blocker, fiber
    @calls << [:unblock, blocker.class]
    @blocked -= 1
    @ready << fiber
  end

  def kernel_sleep duration = nil
    @calls << [:kernel_sleep, duration]
    @ready << Fiber.current
    Fiber.yield
  end

  def io_wait io, events, timeout
    @calls << [:io_wait, events]
    @readers[io] = Fiber.current
    Fiber.yield
    events
  end

  def process_wait pid, flags
    @calls << :process_wait
    Thread.new { Process::Status.wait(pid, flags) }.value
  end

  def timeout_after duration, klass, message, &block
    @calls << [:timeout_after, duration]
    block.call duration
  end

  def fiber_interrupt fiber, exception
    fiber.raise exception
  end

  def close
    @calls << :close
    run
  end

  def run
    until @ready.empty? && @readers.empty? && @blocked.zero?
      @ready.shift.resume until @ready.empty?
      unless @readers.empty?
        readable, = IO.select @readers.keys, nil, nil, 0
        (readable || []).each { |io| @readers.delete(io).resume }
      end
      Thread.pass if @ready.empty?
    end
  end
end

log = []
scheduler = Loop.new
thread = Thread.new do
  Fiber.set_scheduler scheduler
  mutex = Mutex.new
  queue = Queue.new
  reader, writer = IO.pipe
  other = Thread.new { :other_value }

  Fiber.schedule do
    mutex.synchronize do
      log << :first_holds
      sleep 0.01
      log << :first_releases
    end
  end
  Fiber.schedule { mutex.synchronize { log << :second_holds } }
  Fiber.schedule { log << [:popped, queue.pop] }
  Fiber.schedule { log << [:read, reader.read(5)] }
  Fiber.schedule do
    queue << :item
    writer.write "hello"
    writer.close
  end
  Fiber.schedule do
    pid = Process.spawn "true"
    Process.wait pid
    log << [:waited, $?.success?]
  end
  Fiber.schedule { log << [:joined, other.value] }
  Fiber.schedule do
    require "timeout"
    log << [:timed, Timeout.timeout(1) { :inside }]
  end
end
thread.join
p log.sort_by(&:to_s)
p scheduler.calls.uniq.sort_by(&:to_s)
p scheduler.calls.count([:block, Thread::Mutex])
p Fiber.scheduler

# A scheduler that is replaced is closed first.
class Quiet
  def block(*) = nil
  def unblock(*) = nil
  def kernel_sleep(*) = nil
  def io_wait(*) = nil
  def fiber_interrupt(*) = nil

  def close
    puts "closed"
  end
end

Thread.new do
  Fiber.set_scheduler Quiet.new
  p Fiber.current_scheduler
  p Fiber.new(blocking: false) { Fiber.current_scheduler.class }.resume
  Fiber.set_scheduler nil
  p Fiber.scheduler
end.join

begin
  Fiber.schedule { :never }
rescue RuntimeError => refused
  p refused.message
end
