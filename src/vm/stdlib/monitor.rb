# A monitor: a lock a thread may take more than once, and the condition
# variables that wait on it. Metorex runs one thread at a time, so the lock
# records who holds it and how deep rather than blocking anyone.

module MonitorMixin
  # Something to wait on while the monitor is held. Waiting hands the lock
  # back for the duration and takes it again afterwards.
  class ConditionVariable
    def initialize(monitor)
      @monitor = monitor
      @waiting = []
    end

    def wait(timeout = nil)
      @monitor.mon_check_owner
      count = @monitor.mon_exit_for_cond
      begin
        @waiting.push Thread.current
        timeout
      ensure
        @monitor.mon_enter_for_cond count
      end
    end

    def wait_while
      wait until !yield
    end

    def wait_until
      wait until yield
    end

    def signal
      @monitor.mon_check_owner
      @waiting.shift
      nil
    end

    def broadcast
      @monitor.mon_check_owner
      @waiting.clear
      nil
    end
  end

  def self.extend_object(object)
    super
    object.mon_initialize
  end

  def mon_initialize
    @mon_owner = nil
    @mon_count = 0
    self
  end

  def mon_enter
    mon_initialize unless defined? @mon_count
    if @mon_owner.equal? Thread.current
      @mon_count = @mon_count + 1
    else
      @mon_owner = Thread.current
      @mon_count = 1
    end
    nil
  end

  def mon_exit
    mon_check_owner
    @mon_count = @mon_count - 1
    @mon_owner = nil if @mon_count == 0
    nil
  end

  def mon_try_enter
    mon_initialize unless defined? @mon_count
    return false if !@mon_owner.nil? && !@mon_owner.equal?(Thread.current)
    mon_enter
    true
  end

  def mon_locked?
    mon_initialize unless defined? @mon_count
    @mon_count > 0
  end

  def mon_owned?
    mon_locked? && @mon_owner.equal?(Thread.current)
  end

  def mon_synchronize
    mon_enter
    begin
      yield
    ensure
      mon_exit
    end
  end

  def new_cond
    mon_initialize unless defined? @mon_count
    MonitorMixin::ConditionVariable.new self
  end

  # Raised on anyone who tries to leave a monitor they are not holding.
  def mon_check_owner
    mon_initialize unless defined? @mon_count
    unless @mon_owner.equal? Thread.current
      raise ThreadError, "current thread not owner"
    end
    nil
  end

  # Hand the lock back for the whole depth it is held to, answering that
  # depth so waiting can take it again.
  def mon_exit_for_cond
    held = @mon_count
    @mon_owner = nil
    @mon_count = 0
    held
  end

  def mon_enter_for_cond(count)
    @mon_owner = Thread.current
    @mon_count = count
    nil
  end

  alias_method :enter, :mon_enter
  alias_method :exit, :mon_exit
  alias_method :try_enter, :mon_try_enter
  alias_method :synchronize, :mon_synchronize
end

# A monitor of its own, for code that wants one rather than a class that is
# one.
class Monitor
  include MonitorMixin

  def initialize
    mon_initialize
  end
end
