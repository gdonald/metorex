# `sleep` waits for as long as it is told. A length may be named by anything
# that says how to divide itself, waiting backwards is refused, and a fiber
# that is not blocking hands its waiting to the scheduler in place.

p(sleep(0).is_a?(Integer))
p(sleep(Rational(1, 999)) >= 0)

class WaitsItsOwnWay
  def divmod(other)
    [0, 0.001]
  end
end

p(sleep(WaitsItsOwnWay.new) >= 0)

begin
  sleep(-1)
rescue ArgumentError => refused
  p(refused.message)
end

class NotesTheWait
  attr_reader :noted

  def initialize
    @noted = []
  end

  def kernel_sleep(*given)
    @noted << given
    Fiber.yield
  end

  def block(*)
    Fiber.yield
  end

  def unblock(*)
    nil
  end

  def io_wait(*)
    Fiber.yield
  end

  def fiber_interrupt(*)
    nil
  end
end

noting = NotesTheWait.new
Fiber.set_scheduler(noting)

Fiber.new(blocking: false) { sleep(0.01) }.resume
Fiber.new(blocking: false) { sleep }.resume
Fiber.new(blocking: true) { sleep(0) }.resume

p(noting.noted)
Fiber.set_scheduler(nil)
p(Fiber.scheduler)
