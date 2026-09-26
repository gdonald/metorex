# `Thread#raise` builds the exception the way `raise` does, in the thread that
# asks, and the other thread raises it where it stands. On the thread running
# now it raises at once, and a thread that has ended takes nothing.
def refusal
  yield
rescue StandardError => problem
  [problem.class, problem.message]
end

p refusal { Thread.current.raise ArgumentError, "at once" }
p refusal { Thread.current.raise Object.new }

done = Thread.new { :over }
done.join
p done.raise("late")

# The cause is settled where the exception is built, from what that thread
# was handling, rather than from what the raising thread was handling.
caught = nil
sleeper = Thread.new do
  Thread.current.report_on_exception = false
  begin
    begin
      raise "inside"
    rescue
      sleep
    end
  rescue => caught
  end
end
Thread.pass until sleeper.stop?
p $!
sleeper.raise "handed over"
sleeper.join
p caught.message
p caught.cause

# An exception handed with a message is asked for a copy through its own
# `exception`.
logged = []
tracked = Class.new(StandardError) do
  define_method(:exception) do |*given|
    logged << given
    super(*given)
  end
end
waiting = Thread.new do
  Thread.current.report_on_exception = false
  sleep
end
Thread.pass until waiting.stop?
waiting.raise tracked.new, "with a message"
p refusal { waiting.join }[1]
p logged.first(2)

# `rescue => name` inside a block sets the variable the block can see.
seen = nil
[1].each do
  begin
    raise "in a block"
  rescue => seen
  end
end
p seen.message

# Each fiber keeps its own `$!`.
begin
  raise "outer"
rescue
  fiber = Fiber.new { $! }
  p fiber.resume
  p $!.message
end
