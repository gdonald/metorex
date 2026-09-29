# A fiber that has not started, or has run out, takes no exception.
unborn = Fiber.new { true }
begin
  unborn.raise()
rescue FiberError => error
  p(error.message)
end
dead = Fiber.new { true }
dead.resume()
begin
  dead.raise()
rescue FiberError => error
  p(error.message)
end

# One that handed control on by a transfer raises where it stands.
root = Fiber.current
parked = Fiber.new do
  begin
    root.transfer()
  rescue RuntimeError => error
    [:rescued, error.message]
  end
end
parked.transfer()
p(parked.raise("into the transfer"))

# One part-way through resuming another passes it on to that one, and the
# exception comes back out through both.
low = Fiber.new { root.transfer() }
high = Fiber.new { low.resume() }
high.transfer()
begin
  high.raise(RuntimeError, "through the resumer")
rescue RuntimeError => error
  p(error.message)
end
p([low.alive?(), high.alive?()])

# Raised on the fiber that resumed the one running, it is raised at once.
outer = nil
inner = Fiber.new { outer.raise("back up") }
outer = Fiber.new { inner.resume() }
begin
  outer.resume()
rescue RuntimeError => error
  p(error.message)
end

# Another thread cannot raise in a fiber, which stays usable.
waiting = Fiber.new { Fiber.yield(); :resumed }
waiting.resume()
Thread.new do
  begin
    waiting.raise()
  rescue FiberError => error
    p(error.message)
  end
end.join()
p(waiting.resume())
