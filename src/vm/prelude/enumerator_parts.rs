pub(super) const SOURCE: &str = r##"
class Enumerator::EmptyWalk
  include Enumerable

  def each
    self
  end
end

# The block behind `Enumerator.new { |y| ... }`, held as an object of its own
# so a walk can be built from one and asked to run it again.
class Enumerator::Generator
  include Enumerable

  def initialize(&block)
    raise ArgumentError, "tried to create a Generator object without a block" if block.nil?
    @block = block
    self
  end
  private :initialize

  def each(*arguments, &block)
    raise LocalJumpError, "no block given (yield)" if block.nil?
    @block.call(Enumerator::Yielder.new { |*values| block.call(*values) }, *arguments)
  end
end

# The walks a chain runs through in order, which `+` builds.
class Enumerator::Chain < Enumerator
  def initialize(*walks)
    @walks = walks
    self
  end
  private :initialize

  def each
    return to_enum(:each) unless block_given?
    @walked = []
    @walks.each do |walk|
      @walked.push(walk)
      walk.each { |*values| yield(*values) }
    end
    self
  end

  def to_a
    collected = []
    each { |*values| collected.push(values.size == 1 ? values[0] : values) }
    collected
  end

  def force
    to_a
  end

  def size
    total = 0
    @walks.each do |walk|
      walked = walk.size
      return nil if walked.nil?
      return walked if walked == Float::INFINITY
      total += walked
    end
    total
  end

  # Ruby rewinds the walks a chain has run, last one first, and leaves the
  # ones it never reached alone.
  def rewind
    listed = @walked.nil? ? [] : @walked
    listed.reverse.each { |walk| walk.rewind if walk.respond_to?(:rewind) }
    self
  end

  def +(other)
    Enumerator::Chain.new(self, other)
  end

  def inspect
    return "#<Enumerator::Chain: uninitialized>" if @walks.nil?
    "#<Enumerator::Chain: #{@walks.inspect}>"
  end
end

class Enumerator
  def +(other)
    Enumerator::Chain.new(self, other)
  end
end

# The Cartesian product of several walks, which yields one array per
# combination in the order the walks were given.
class Enumerator::Product < Enumerator
  def initialize(*enumerables)
    raise FrozenError, "can\'t modify frozen #{self.class}" if frozen?
    @enumerables = enumerables
    self
  end
  private :initialize

  def initialize_copy(other)
    return self if other.equal?(self)
    raise FrozenError, "can\'t modify frozen #{self.class}" if frozen?
    unless other.class == self.class
      raise TypeError, "initialize_copy should take same class object"
    end
    taken = other.instance_variable_get(:@enumerables)
    raise ArgumentError, "uninitialized product" if taken.nil?
    @enumerables = taken
    self
  end
  private :initialize_copy

  def each(&block)
    return Enumerator.over(self, :each, [], size) if block.nil?
    __combined__([], 0, &block)
    self
  end

  # One entry from the walk at `at`, then every combination of the walks
  # under it. Each walk is read only as far as the block asks for, so a
  # product over an endless walk still yields.
  def __combined__(prefix, at, &block)
    if at == @enumerables.size
      block.call(prefix.dup)
      return nil
    end
    @enumerables[at].each_entry do |entry|
      prefix.push(entry)
      __combined__(prefix, at + 1, &block)
      prefix.pop
    end
    nil
  end
  private :__combined__

  def to_a
    collected = []
    each { |combination| collected.push(combination) }
    collected
  end

  def rewind
    @enumerables.each do |enumerable|
      enumerable.rewind if enumerable.respond_to?(:rewind)
    end
    self
  end

  # The number of combinations, which is nil as soon as one walk cannot say
  # how long it is.
  def size
    total = 1
    @enumerables.each do |enumerable|
      counted = begin
        enumerable.size
      rescue NoMethodError
        return nil
      end
      return Float::INFINITY if counted == Float::INFINITY
      return nil unless counted.is_a?(Integer)
      total = total * counted
    end
    total
  end

  def inspect
    return "#<Enumerator::Product: uninitialized>" if @enumerables.nil?
    return "#<Enumerator::Product: ...>" if @rendering
    @rendering = true
    written = "#<Enumerator::Product: #{@enumerables.inspect}>"
    @rendering = false
    written
  end
end

# A walk over evenly spaced numbers, which is what `1.step(10)` and
# `(1..10).step(2)` answer when they are handed no block. It is built only
# through those methods, so `new` and `allocate` are not among its own.
class Enumerator::ArithmeticSequence < Enumerator
  def self.allocate
    raise TypeError, "allocator undefined for Enumerator::ArithmeticSequence"
  end

  def initialize(from, to, by, exclude_end, source, written_as, step_given)
    @from = from
    @to = to
    @by = by
    @exclude_end = exclude_end
    @source = source
    @written_as = written_as
    @step_given = step_given
    # The walk is over this sequence's own `each`, which `Enumerator.over`
    # fills in for a walk that has no block behind it.
    @receiver = self
    @method_name = :each
    @arguments = []
    @size = nil
    @position = 0
    @generator = nil
  end
  private_class_method :new

  def begin
    @from
  end

  def end
    @to
  end

  def step
    @by
  end

  def exclude_end?
    @exclude_end
  end

  def each(&block)
    return self if block.nil?
    if walks_as_floats?
      counted = float_step_count
      # A step of no width at all reaches nowhere, so the walk stands where
      # it started rather than counting from it.
      if @by.to_f.infinite?
        block.call(@from.to_f) if counted > 0
        return self
      end
      endless = counted == Float::INFINITY
      place = 0
      while endless || place < counted
        value = place * @by + @from
        # The last value a counted walk reaches can land past its end by the
        # rounding the multiplication carries, so it is pulled back to the end
        # it was counted to.
        value = @to if !endless && (@by >= 0 ? @to < value : value < @to)
        block.call(value.to_f)
        place = place + 1
      end
      return self
    end
    value = @from
    while within?(value)
      block.call(value)
      value = value + @by
    end
    self
  end

  # Whether the walk counts in floats, where the steps are found by counting
  # rather than by adding one to the last, so rounding does not build up.
  def walks_as_floats?
    return false if @from.nil?
    @from.is_a?(Float) || @to.is_a?(Float) || @by.is_a?(Float)
  end
  private :walks_as_floats?

  # How many steps a float walk takes, read the way Ruby reads it: from the
  # span divided by the step, widened by the rounding the division carries.
  def float_step_count
    first = @from.to_f
    last = @to.nil? ? Float::INFINITY : @to.to_f
    by = @by.to_f
    if by.infinite?
      return (by > 0 ? first <= last : first >= last) ? 1 : 0
    end
    return Float::INFINITY if by == 0
    steps = (last - first) / by
    # A walk from one infinity to the same one has a span that is no number,
    # so the count is none either. It is handed back as it stands: a walk
    # counted that way yields nothing, and asking its size is refused.
    return steps if steps.nan?
    # A span no number of steps can cross is not rounded, since rounding an
    # infinity is refused. Counting up from it never ends, and counting down
    # from it reaches nowhere.
    return steps if steps == Float::INFINITY
    return 0 if steps == -Float::INFINITY
    slack = (first.abs + last.abs + (last - first).abs) / by.abs * Float::EPSILON
    slack = 0.5 if slack > 0.5
    if @exclude_end
      return 0 if steps <= 0
      counted = steps < 1 ? 0 : (steps - slack).floor
    else
      return 0 if steps < 0
      counted = (steps + slack).floor
    end
    # One more step can still land inside the end once the rounding above has
    # been taken off, so where it does, it is counted.
    reach = (counted + 1) * by + first
    inside = by > 0 ? (@exclude_end ? reach < last : reach <= last) :
      (@exclude_end ? reach > last : reach >= last)
    counted = counted + 1 if inside
    counted + 1
  end
  private :float_step_count

  # Whether a value is still inside the sequence, which for a walk with no
  # end is always.
  def within?(value)
    return true if @to.nil?
    if @by < 0
      @exclude_end ? value > @to : value >= @to
    else
      @exclude_end ? value < @to : value <= @to
    end
  end
  private :within?

  def size
    return Float::INFINITY if @to.nil? || @from.nil?
    if walks_as_floats?
      counted = float_step_count
      raise FloatDomainError, "NaN" if counted.is_a?(Float) && counted.nan?
      return counted
    end
    span = @to - @from
    steps = (span / @by).floor
    counted = steps + 1
    counted = counted - 1 if @exclude_end && steps * @by == span
    counted < 0 ? 0 : counted
  end

  def first(count = nil)
    return @from if count.nil?
    collected = []
    each do |value|
      break if collected.size >= count
      collected.push(value)
    end
    collected
  end

  def last(count = nil)
    counted = size
    return nil if counted == Float::INFINITY
    return nil if counted == 0
    ending = @from + (counted - 1) * @by
    return ending if count.nil?
    kept = count > counted ? counted : count
    (0...kept).map { |place| @from + (counted - kept + place) * @by }
  end

  def ==(other)
    return false unless other.is_a?(Enumerator::ArithmeticSequence)
    self.begin == other.begin && self.end == other.end &&
      step == other.step && exclude_end? == other.exclude_end?
  end

  def eql?(other)
    self == other
  end

  def hash
    [@from, @to, @by, @exclude_end].hash
  end

  def inspect
    "(" + written_form + ")"
  end

  def to_s
    inspect
  end

  # How the sequence was asked for, which is what Ruby writes it back as.
  def written_form
    if @source.nil?
      return "#{@from.inspect}.step" if @to.nil? && !@step_given
      written = "#{@from.inspect}.step(#{@to.inspect}"
      written = written + ", #{@by.inspect}" if @step_given
      return written + ")"
    end
    written = "(#{@source.inspect}).#{@written_as}"
    return written + "(#{@by.inspect})" if @step_given
    written
  end
  private :written_form
end
"##;
