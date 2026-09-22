# A program that ends takes its threads with it. Each thread unwinds where it
# waits, so the `ensure` clauses the fiber it is on sits inside run. A fiber
# the thread left suspended stays that way, and its own do not.

ready = false

Thread.new do
  suspended = Fiber.new do
    begin
      Fiber.yield
    ensure
      puts("the suspended fiber would say so here")
    end
  end
  suspended.resume

  waiting = Fiber.new do
    begin
      ready = true
      sleep
    ensure
      puts("the waiting fiber unwinds")
    end
  end
  waiting.resume
end

Thread.pass until ready
