pub(super) const SOURCE: &str = r##"
class Rational
  # The simplest fraction standing no further away than the tolerance given.
  # Without one the number stands for itself.
  def rationalize(*limits)
    if limits.size > 1
      raise ArgumentError, "wrong number of arguments (given #{limits.size}, expected 0..1)"
    end
    return self if limits.empty?
    slack = limits[0].abs.to_r
    low = self - slack
    high = self + slack
    return Rational(0, 1) if low <= 0 && high >= 0
    return -Rational.__simplest_between__(-high, -low) if high < 0
    Rational.__simplest_between__(low, high)
  end

  # The fraction with the smallest denominator lying between two positive
  # bounds, found by the continued fraction the pair share.
  def self.__simplest_between__(low, high)
    whole = low.floor
    return Rational(whole + 1, 1) if whole + 1 <= high
    return Rational(whole, 1) if whole == low
    inner = __simplest_between__(Rational(1, 1) / (high - whole), Rational(1, 1) / (low - whole))
    Rational(whole * inner.numerator + inner.denominator, inner.numerator)
  end

  # The pair an arithmetic operator works on: an Integer becomes a Rational,
  # a Float makes this a Float, and a Complex with an exactly zero imaginary
  # part reads as its real part.
  def coerce(other)
    case other
    when Integer
      [Rational(other, 1), self]
    when Float
      [other, to_f]
    when Rational
      [other, self]
    when Complex
      if other.imaginary.is_a?(Integer) && other.imaginary.zero?
        [Rational(other.real), self]
      else
        [other, Complex(self)]
      end
    else
      raise TypeError, "#{other.class} can't be coerced into Rational"
    end
  end

  # What `Marshal` writes for a Rational: the two parts, in the order
  # `Rational(numerator, denominator)` takes them.
  def marshal_dump
    [numerator, denominator]
  end
  private :marshal_dump
end

class Set
  # `pp` calls this instead of `pretty_print` for a set that holds itself, so
  # printing one stops rather than descending forever.
  def pretty_print_cycle(printer)
    printer.text("Set[...]")
  end
end

class Fiber
  # Raise an exception in this fiber, built from the arguments the way
  # `Kernel#raise` builds one, where the fiber stands.
  def raise(*arguments, **options)
    __raise__(*arguments, **options)
  end
end

class Thread
  # Raise an exception in this thread, built from the arguments the way
  # `Kernel#raise` builds one. On the thread running now it is raised at once.
  # Another thread raises it where it stands, with a backtrace of its own
  # unless the exception carries one already, and a thread that has ended
  # takes nothing.
  def raise(*arguments, **options)
    return nil unless alive?
    return ::Kernel.raise(*arguments, **options) if equal? Thread.current
    __raise_later__ __build_raised__(*arguments, **options)
    nil
  end

  # Run the block with the interrupts named in `mapping` handled the way it
  # says: :immediate raises one where the thread stands, :on_blocking waits
  # for the next place the thread waits on something, and :never holds it
  # until the block is over.
  def self.handle_interrupt(mapping)
    unless mapping.is_a?(Hash)
      raise ArgumentError, "unknown mask signature"
    end
    raise LocalJumpError, "no block given" unless block_given?
    thread = Thread.current
    held = thread.instance_variable_get(:@__interrupt_masks) || []
    thread.instance_variable_set(:@__interrupt_masks, held + [mapping])
    begin
      Thread.__run_pending_interrupt__ false
      yield
    ensure
      thread.instance_variable_set(:@__interrupt_masks, held)
      Thread.__run_pending_interrupt__ false
    end
  end

  # Whether an exception handed to a thread is waiting to be raised.
  def self.pending_interrupt?(_error = nil)
    Thread.current.pending_interrupt?
  end

  def pending_interrupt?(_error = nil)
    held = instance_variable_get(:@__thread_raise)
    !held.nil? && !held.empty?
  end

  # Raise what the thread was handed, when the masks it set allow it now.
  # `blocking` says whether the thread is waiting on something here.
  def self.__run_pending_interrupt__(blocking)
    thread = Thread.current
    pending = thread.instance_variable_get(:@__thread_raise)
    return nil if pending.nil? || pending.empty?
    how = __interrupt_handling__ thread, pending
    return nil if how == :never
    return nil if how == :on_blocking && !blocking
    thread.instance_variable_set(:@__thread_raise, nil)
    # The exception is raised where the thread was interrupted, so the
    # backtrace starts at the call that was waiting rather than here.
    handed = pending.first
    handed.set_backtrace(caller(1)) if handed.is_a?(Exception) && handed.backtrace.nil?
    raise(*pending)
  end

  # How the innermost mask naming the pending exception's class says to
  # handle it. Nothing naming it means it is raised right away.
  def self.__interrupt_handling__(thread, pending)
    masks = thread.instance_variable_get(:@__interrupt_masks)
    return :immediate if masks.nil? || masks.empty?
    first = pending.first
    kind = if first.is_a?(Class)
             first
           elsif first.is_a?(Exception)
             first.class
           else
             RuntimeError
           end
    masks.reverse_each do |mask|
      mask.each_pair do |named, how|
        return how if kind <= named
      end
    end
    :immediate
  end

  # Hand each place the running code was called from to the block, innermost
  # first, the way `caller_locations` reads them.
  def self.each_caller_location(*args, **keywords)
    unless args.empty? && keywords.empty?
      raise ArgumentError, "wrong number of arguments (given #{args.size}, expected 0)"
    end
    raise LocalJumpError, "no block given" unless block_given?
    caller_locations(2).each { |place| yield place }
    nil
  end

  # Whether an exception a thread dies of is reported, and whether it takes
  # the program down with it. Both are settings a program may read back, and
  # a thread of its own overrides what the class says.
  def self.abort_on_exception
    @abort_on_exception == true
  end

  def self.abort_on_exception=(wanted)
    @abort_on_exception = wanted
  end

  def report_on_exception
    @__report_on_exception__.nil? ? Thread.report_on_exception : @__report_on_exception__
  end

  def report_on_exception=(wanted)
    @__report_on_exception__ = wanted
  end

  def abort_on_exception
    @__abort_on_exception__.nil? ? Thread.abort_on_exception : @__abort_on_exception__
  end

  def abort_on_exception=(wanted)
    @__abort_on_exception__ = wanted
  end

  # A thread that dies of an exception writes what it died of, unless it was
  # told to keep quiet. It is written while the thread is still running, so
  # the report names it the way it stood when it failed.
  def __report_terminated__(error)
    return nil unless report_on_exception
    return nil unless error.is_a?(Exception)
    # A thread ending the program carries the word to the main thread rather
    # than dying of it, so there is nothing to report.
    return nil if error.is_a?(SystemExit)
    written = "#{inspect} terminated with exception (report_on_exception is true):\n"
    written += error.full_message(highlight: false, order: :top)
    $stderr.write written
    nil
  end
end

class Thread
  # A thread that stops waits to be woken. Metorex runs a thread's block on
  # the thread that made it, so there is nothing to wait for and nothing to
  # wake it from.
  def self.stop
    nil
  end
end

class Thread
  # Ruby reports a deadlock among its threads unless this is switched off.
  # Metorex runs a thread's block on the thread that made it, so nothing can
  # deadlock, and the reading is kept because a program may set it.
  def self.ignore_deadlock
    @ignore_deadlock == true
  end

  def self.ignore_deadlock=(ignored)
    @ignore_deadlock = ignored
  end
end

class Regexp
  # A match that runs longer than a pattern was given is given up on, which
  # this is what it reports.
  class TimeoutError < RegexpError
  end

  # How long a match may take before it is given up on. Nothing named here
  # lets a match take as long as it takes.
  def self.timeout
    $__regexp_timeout__
  end

  def self.timeout=(seconds)
    if seconds.nil?
      $__regexp_timeout__ = nil
      return seconds
    end
    held = Float(seconds)
    raise ArgumentError, "invalid timeout: #{seconds}" unless held > 0
    $__regexp_timeout__ = held
    seconds
  end

  IGNORECASE = 1
  EXTENDED = 2
  MULTILINE = 4
  FIXEDENCODING = 16
  NOENCODING = 32

  # A Regexp that was made without a pattern has nothing to answer about,
  # which Ruby reports rather than treating it as an empty pattern.
  def options
    raise TypeError, "uninitialized Regexp"
  end

  def match(*_arguments)
    raise TypeError, "uninitialized Regexp"
  end

  def match?(*_arguments)
    raise TypeError, "uninitialized Regexp"
  end

  # The pattern an object stands for, or nil where it stands for none. Only
  # an object answering `to_regexp` is asked.
  def self.try_convert(held)
    return held if held.is_a?(Regexp)
    return nil unless held.respond_to?(:to_regexp)
    converted = held.to_regexp
    return converted if converted.is_a?(Regexp)
    raise TypeError,
      "can't convert #{held.class} into Regexp (#{held.class}#to_regexp gives #{converted.class})"
  end

  # Whether a pattern is matched in time proportional to the subject's length.
  # Metorex matches with a linear automaton, which reads every pattern but a
  # back reference that way.
  def self.linear_time?(pattern, options = nil)
    if pattern.is_a?(Regexp)
      warn "warning: flags ignored" unless options.nil?
      return !__reaches_back__(pattern.source)
    end
    unless pattern.is_a?(String)
      raise TypeError, "wrong argument type #{pattern.class} (expected Regexp)"
    end
    !__reaches_back__(pattern)
  end

  # A Regexp is built once. Handing `initialize` to one that already holds a
  # pattern refuses, which is what Ruby does for a literal and for one built
  # by `new` alike.
  def initialize(source = nil, options = nil, timeout: nil)
    # A subclass writing its own `initialize` calls this one through `super`
    # while the pattern behind it is being built, which is the one time it
    # has nothing to refuse.
    return nil if instance_variable_defined?(:@__building_regexp__)
    raise FrozenError, "can't modify frozen Regexp: #{inspect}" if frozen?
    raise TypeError, "already initialized regexp"
  end
  private :initialize

  # Whether a pattern names one of its own earlier groups, which no automaton
  # reads in one pass.
  def self.__reaches_back__(source)
    !(source =~ /\\(?:[1-9]|k<[^>]+>)/).nil?
  end
  private_class_method :__reaches_back__
end

class Time
  # A Time already stands for the moment `to_time` asks for.
  def to_time
    self
  end
end

# A synchronization object stands for something the running program holds, so
# Marshal refuses to write one out rather than hand back a copy that locks
# nothing.
class ConditionVariable
  def marshal_dump
    raise TypeError, "can't dump #{self.class}"
  end
end

class Mutex
  def marshal_dump
    raise TypeError, "can't dump #{self.class}"
  end
end

class Queue
  def marshal_dump
    raise TypeError, "can't dump #{self.class}"
  end
end

class ThreadGroup
  def initialize
    @threads = []
    @enclosed = false
  end

  def list
    @threads.dup
  end

  def add(thread)
    held = thread.group
    if held && held.enclosed? && !held.equal?(self)
      raise ThreadError, "can't move from the enclosed thread group"
    end
    held.__remove__(thread) if held
    @threads.push thread unless @threads.include?(thread)
    thread.__set_group__(self)
    self
  end

  def enclose
    @enclosed = true
    self
  end

  def enclosed?
    @enclosed
  end

  def __remove__(thread)
    @threads = @threads.reject { |held| held.equal?(thread) }
    self
  end

  Default = new
end

class Thread
  # One fiber-local by name, with a default or a block standing in when the
  # thread never stored it. A default and a block together are an ambiguity
  # Ruby warns about and settles in the block's favor.
  def fetch(name, *default, &block)
    if default.size > 1
      raise ArgumentError, "wrong number of arguments (given #{1 + default.size}, expected 1..2)"
    end
    return self[name] if key?(name)
    unless block.nil?
      warn "warning: block supersedes default value argument" unless default.empty?
      return block.call(name)
    end
    return default[0] unless default.empty?
    raise KeyError, "key not found: #{name.inspect}"
  end
end

class Thread
  # A thread is set up when it is made. Calling `initialize` on one again is
  # refused, since the thread it would start is already running.
  def initialize(*_arguments)
    raise ThreadError, "already initialized thread"
  end

  # The group this thread belongs to, which is the default one until another
  # group takes it.
  def group
    @__thread_group__ = ThreadGroup::Default if @__thread_group__.nil?
    @__thread_group__
  end

  def __set_group__(group)
    @__thread_group__ = group
    self
  end
end
"##;
