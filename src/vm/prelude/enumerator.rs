pub(super) const SOURCE: &str = r##"
class Enumerator
  include Enumerable

  # Every way of taking one element from each of the walks given, as a walk of
  # its own. With a block the tuples are handed over as they are made.
  def self.product(*walks, **keywords)
    unless keywords.empty?
      named = keywords.keys.map { |key| key.inspect }.join(", ")
      raise ArgumentError, "unknown keywords: #{named}"
    end
    made = Enumerator::Product.new(*walks)
    return made unless block_given?
    made.each { |held| yield held }
    nil
  end

  # What a generator block is handed, so `Enumerator.new { |y| y << 1 }`
  # reads the same way as one built over a method that yields. Everything
  # handed to it goes to the block it was made with.
  class Yielder
    def initialize(&block)
      @block = block
      self
    end
    private :initialize

    def <<(value)
      @block.call(value)
      self
    end

    def yield(*values)
      @block.call(*values)
    end

    def to_proc
      @block
    end
  end

  # `Enumerator.new { |y| ... }` names the count the walk will hand out, and
  # nothing else: the walk itself is the block.
  def initialize(size = nil, &generator)
    raise ArgumentError, "wrong number of arguments (given 0, expected 1+)" if generator.nil?
    @receiver = nil
    @method_name = nil
    @arguments = []
    @size = size
    @position = 0
    @generator = generator
    @stepping_walk = nil
    @walk_started = false
    @raw_values = nil
    @values = nil
    self
  end
  private :initialize

  # A walk over a method of another object, which is what `to_enum` and the
  # methods that answer an Enumerator without a block build. `new` takes a
  # block instead, so this one fills the same state without it.
  def self.over(receiver, method_name = nil, arguments = [], size = nil)
    made = allocate
    made.instance_variable_set(:@receiver, receiver)
    made.instance_variable_set(:@method_name, method_name)
    made.instance_variable_set(:@arguments, arguments)
    made.instance_variable_set(:@size, size)
    made.instance_variable_set(:@position, 0)
    made.instance_variable_set(:@generator, nil)
    made
  end

  # The count the walk will hand out, where that is known ahead of it. An
  # endpoint the walk cannot count to has no size to report, and asking is
  # refused rather than answered with a guess.
  def size
    raise ArgumentError, @size_error unless @size_error.nil?
    # A size named by something callable is asked each time, so a count that
    # depends on what the program has done since answers the new one.
    return @size.call if @size.respond_to?(:call)
    @size
  end

  def __refuse_size__(message)
    @size_error = message
    self
  end

  def to_a
    if @values.nil?
      collected = []
      # An enumerator given an `each` of its own is walked through that body
      # rather than through the object it was cut from.
      if singleton_methods.include?(:each)
        @result = each do |*yielded|
          collected.push(yielded.empty? ? nil : (yielded.size == 1 ? yielded[0] : yielded))
          nil
        end
      elsif @generator.nil?
        @result = @receiver.send(@method_name, *@arguments) do |*yielded|
          collected.push(yielded.empty? ? nil : (yielded.size == 1 ? yielded[0] : yielded))
          nil
        end
      else
        @result = @generator.call(Yielder.new { |*yielded|
          collected.push(yielded.size == 1 ? yielded[0] : yielded)
          nil
        })
      end
      @values = collected
    end
    @values
  end

  # `next_values` and `peek_values` answer what the walk yielded as an array,
  # where `next` and `peek` unwrap a lone value.
  # The walk a stepped read is part-way through. It runs on a fiber of its
  # own, so the values arrive one at a time rather than all at once, and a
  # walk that never ends can still be stepped through.
  def stepping_walk
    return @stepping_walk unless @stepping_walk.nil?
    @stepping_walk = Fiber.new do
      @result = if singleton_methods.include?(:each)
        each { |*yielded| Fiber.yield [:value, yielded] }
      elsif @generator.nil?
        @receiver.send(@method_name, *@arguments) { |*yielded| Fiber.yield [:value, yielded] }
      else
        @generator.call(Yielder.new { |*yielded| Fiber.yield [:value, yielded] })
      end
      [:done, @result]
    end
  end
  private :stepping_walk

  # The next run of values the walk hands over, or the end of it.
  def step_walk
    unless @peeked.nil?
      held = @peeked
      @peeked = nil
      return held
    end
    walker = stepping_walk
    refuse_ended unless walker.alive?
    # A value fed in reaches the `yield` the walk is parked at, so it is
    # handed over on the step that starts the walk again rather than the one
    # that opens it.
    if @walk_started
      fed = @fed
      @fed = nil
      @fed_given = false
    else
      fed = nil
      @walk_started = true
    end
    begin
      handed = walker.resume(fed)
    rescue Exception
      # A walk that ended in an exception starts again from the beginning,
      # which is what Ruby does with the one that raised.
      @stepping_walk = nil
      @walk_started = false
      raise
    end
    refuse_ended if handed.nil? || handed[0] == :done
    handed[1]
  end
  private :step_walk

  def refuse_ended
    ended = StopIteration.new("iteration reached an end")
    ended.result = @result
    raise ended
  end
  private :refuse_ended

  def next_values
    step_walk
  end

  def peek_values
    @peeked = step_walk if @peeked.nil?
    @peeked
  end

  def peek
    values = peek_values
    values.empty? ? nil : (values.size == 1 ? values[0] : values)
  end

  def next
    values = next_values
    values.empty? ? nil : (values.size == 1 ? values[0] : values)
  end

  # The value the next `yield` in the walk answers. Ruby holds one at a time,
  # and refuses a second before the walk has moved on.
  def feed(value)
    raise TypeError, "feed value already set" if @fed_given
    @fed = value
    @fed_given = true
    nil
  end

  def raw_to_a
    if @raw_values.nil?
      collected = []
      if singleton_methods.include?(:each)
        @result = each do |*yielded|
          collected.push(yielded)
          nil
        end
      elsif @generator.nil?
        @result = @receiver.send(@method_name, *@arguments) do |*yielded|
          collected.push(yielded)
          nil
        end
      else
        @result = @generator.call(Yielder.new { |*yielded|
          collected.push(yielded)
          nil
        })
      end
      @raw_values = collected
    end
    @raw_values
  end

  # Ruby hands the rewind on to the object the walk was cut from when that
  # object can be rewound, so a source with a place of its own goes back to
  # the start too. The stepped walk starts again from nothing.
  def rewind
    @position = 0
    @stepping_walk = nil
    @walk_started = false
    @peeked = nil
    @fed = nil
    @fed_given = false
    if !@receiver.nil? && !@receiver.equal?(self) && @receiver.respond_to?(:rewind)
      @receiver.rewind
    end
    self
  end

  # An endless walk built from a value and the block that answers the next
  # one. Without a starting value the block is handed nil the first time.
  def self.produce(*first, &step)
    raise ArgumentError, "no block given" if step.nil?
    if first.size > 1
      raise ArgumentError, "wrong number of arguments (given #{first.size}, expected 0..1)"
    end
    starts_empty = first.empty?
    opening = first[0]
    new do |yielder|
      held = starts_empty ? step.call(nil) : opening
      loop do
        yielder << held
        held = step.call(held)
      end
    end
  end

  # The enumerator `loop` answers when called without a block: it yields
  # forever and reports an endless size.
  def self.endless
    enumerator = over(nil, nil, [], Float::INFINITY)
    enumerator.mark_endless
  end

  def mark_endless
    @endless = true
    self
  end

  # With a block, the walk runs the method the Enumerator was cut from and
  # answers what that method answers, so `numbers.find(ifnone).each { }` is
  # the same call as `numbers.find(ifnone) { }`.
  def each(*args, &block)
    # `each(more)` with no block answers a walk carrying the extra values,
    # which the method behind it is handed when the walk runs.
    if block.nil?
      return self if args.empty?
      return Enumerator.over(@receiver, @method_name, @arguments + args, @size)
    end
    args = @arguments + args if !args.empty? && @generator.nil?
    if @endless
      while true
        block.call
      end
    end
    if @generator.nil?
      held = args.empty? ? @arguments : args
      return @receiver.send(@method_name, *held, &block)
    end
    # The generator runs with the block standing behind the yielder, so a
    # walk that has seen enough can stop it rather than waiting for a source
    # that never ends.
    @generator.call(Yielder.new { |*yielded|
      block.call(*yielded)
    })
    self
  end

  # Only as much of the walk as was asked for is run, so a walk with no end
  # still answers its first few.
  def first(count = nil)
    wanted = count.nil? ? 1 : count
    raise ArgumentError, "attempt to take negative size" if wanted < 0
    collected = []
    unless wanted == 0
      each do |*values|
        collected.push(packed(values))
        break if collected.size >= wanted
      end
    end
    count.nil? ? collected[0] : collected
  end

  def map(&block)
    return self if block.nil?
    # The walk runs as it goes rather than collecting first, so what the
    # method behind it sets, `$~` among them, is there for the block.
    collected = []
    # The values one step yielded reach the block spread out, so a block
    # taking one parameter takes the first of a pair.
    each { |*values| collected.push(block.call(*values)) }
    collected
  end

  def collect(&block)
    map(&block)
  end

  # Walks with a second value handed to the block each time, answering that
  # value once the walk is done.
  def with_object(memo)
    return Enumerator.over(self, :with_object, [memo], @size) unless block_given?
    each { |*values| yield packed(values), memo }
    memo
  end

  # The same walk, with a count alongside each element. The count starts at
  # the offset given, and what the block answers reaches the method behind
  # the walk, which is what `chunk.with_index { }` reads.
  def with_index(offset = 0, &block)
    counted = __index_offset__ offset
    return Enumerator.over(self, :with_index, [counted], @size) if block.nil?
    each do |*values|
      outcome = block.call(packed(values), counted)
      counted += 1
      outcome
    end
  end

  # Where a walk starts counting. Nothing at all starts at zero, and
  # anything that is not already an Integer is asked for one.
  def __index_offset__(offset)
    return 0 if offset.nil?
    return offset if offset.is_a? Integer
    unless offset.respond_to? :to_int
      raise TypeError, "no implicit conversion of #{offset.class} into Integer"
    end
    named = offset.to_int
    unless named.is_a? Integer
      raise TypeError, "can't convert #{offset.class} to Integer (#{offset.class}#to_int gives #{named.class})"
    end
    named
  end
  private :__index_offset__

  # A walk's `each_with_index` answers whatever the method behind the walk
  # answers, and the block's own value reaches that method.
  def each_with_index(*arguments, &block)
    unless arguments.empty?
      raise ArgumentError, "wrong number of arguments (given #{arguments.size}, expected 0)"
    end
    return Enumerator.over(self, :each_with_index, [], @size) if block.nil?
    index = 0
    each do |*values|
      outcome = block.call(packed(values), index)
      index += 1
      outcome
    end
  end

  def inspect
    return "#<#{self.class}: uninitialized>" if @position.nil?
    written = "#<#{self.class}: #{@receiver.inspect}:#{@method_name}"
    listed = @arguments.nil? ? [] : @arguments
    written += "(#{listed.map { |one| one.inspect }.join(', ')})" unless listed.empty?
    written + ">"
  end

end
# A walk that applies its operations one element at a time, so a source with
# no end can still answer `first` or `take`. Each step holds the walk it
# reads from, so the chain runs outside in."##;
