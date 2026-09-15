# The same as `fibers_that_schedule.rb`, written without parentheses.
# `require "fiber"` adds the scheduler a program hands to Fiber, which has to
# answer the four methods a scheduler is asked for.
require 'fiber'

p Fiber.scheduler

incomplete = Object.new
def incomplete.block; end
def incomplete.unblock; end
def incomplete.kernel_sleep; end

begin
  Fiber.set_scheduler incomplete
rescue ArgumentError => trouble
  p trouble.message
end

complete = Object.new
def complete.block; end
def complete.unblock; end
def complete.kernel_sleep; end
def complete.io_wait; end

p Fiber.set_scheduler(complete).equal?(complete)
p Fiber.scheduler.equal?(complete)
p Fiber.set_scheduler(nil)
p Fiber.scheduler

# An exception handed to a thread is raised where it waits, and
# `Thread.handle_interrupt` says when a thread is willing to take one.
holding = []
begin
  Thread.handle_interrupt(RuntimeError => :never) do
    main = Thread.current
    Thread.new { main.raise 'later' }.join
    holding << Thread.pending_interrupt?
  end
rescue RuntimeError => waiting
  holding << waiting.message
end
p holding
