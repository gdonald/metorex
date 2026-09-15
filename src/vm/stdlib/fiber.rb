# The `fiber` library adds the scheduler a program hands to Fiber, which
# stands in for the operating system wherever a fiber would otherwise wait.
# Metorex runs fibers cooperatively already, so what a scheduler is given is
# remembered and reported rather than used.

class Fiber
  # The methods a scheduler has to answer before Fiber will take it.
  SCHEDULER_METHODS = [:block, :unblock, :kernel_sleep, :io_wait]

  def self.scheduler
    @scheduler
  end

  def self.set_scheduler(wanted)
    if wanted.nil?
      @scheduler = nil
      return nil
    end
    SCHEDULER_METHODS.each do |named|
      unless wanted.respond_to? named
        raise ArgumentError, "Scheduler must implement ##{named}"
      end
    end
    @scheduler = wanted
  end

  # The scheduler a fiber of this thread would use, which is the one the
  # thread was given.
  def self.current_scheduler
    @scheduler
  end
end
