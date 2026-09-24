pub(super) const SOURCE: &str = r##"
class Enumerator::Lazy < Enumerator
  def initialize(source = nil, size = nil, &transform)
    raise ArgumentError, "tried to call lazy new without a block" if transform.nil?
    @source = source
    @lazy_size = size.is_a?(Proc) ? size.call : size
    @transform = transform
    @kind = nil
    @callable = nil
    @count = nil
    self
  end
  private :initialize

  # A lazy walk built by another lazy method rather than by `new`, which
  # takes no block of its own.
  def self.build(source, size = nil)
    made = allocate
    made.instance_variable_set(:@source, source)
    made.instance_variable_set(:@lazy_size, size)
    made.instance_variable_set(:@transform, nil)
    made.instance_variable_set(:@kind, nil)
    made.instance_variable_set(:@callable, nil)
    made.instance_variable_set(:@count, nil)
    made
  end

  # How many values a step leaves behind. A step that decides what to keep
  # cannot say, so it reports nil.
  def step_size(kind, count)
    return @lazy_size if kind == :map || kind == :with_index || kind == :zip
    return nil if @lazy_size.nil?
    if kind == :take
      return @lazy_size < count ? @lazy_size : count
    end
    if kind == :drop
      left = @lazy_size - count
      return left < 0 ? 0 : left
    end
    nil
  end
  private :step_size

  # Add one step to the walk, which is what every lazy method answers.
  def with_step(kind, callable = nil, count = nil)
    built = Enumerator::Lazy.build(self, step_size(kind, count))
    built.instance_variable_set(:@kind, kind)
    built.instance_variable_set(:@callable, callable)
    built.instance_variable_set(:@count, count)
    built
  end
  private :with_step

  def size
    @lazy_size
  end

  def lazy
    self
  end

  def eager
    Enumerator.over(self, :each, [], @lazy_size)
  end

  # A block names the count the walk will hand out.
  def to_enum(method_name = :each, *args, &size)
    Enumerator::Lazy.build(Enumerator::MethodWalk.new(self, method_name, args), size.nil? ? nil : size.call)
  end

  def enum_for(method_name = :each, *args, &size)
    to_enum(method_name, *args, &size)
  end

  # Enumerable's own methods answer a lazy walk here when given no block.
  [:each_with_index, :each_with_object, :with_object, :each_slice, :each_entry, :each_cons].each do |name|
    define_method(name) do |*args, &block|
      return to_enum(name, *args) if block.nil?
      super(*args, &block)
    end
  end

  # Hand each element the step it stands for, and stop the walk as soon as
  # nothing more can come out of it.
  def each(*args, &walker)
    return self if walker.nil?
    if @kind == :chunk && @callable.nil?
      @callable = walker
      return self
    end
    taken = 0
    seen = 0
    dropping = true
    kept = []
    grouped = []
    grouping = false
    group_key = nil
    previous = nil
    walk_source(*args) do |values|
      case @kind
      when nil
        yield(*values)
      when :map
        yield @callable.call(*values)
      when :select
        yield(*values) if @callable.call(packed(values))
      when :reject
        yield(*values) unless @callable.call(packed(values))
      when :filter_map
        answered = @callable.call(*values)
        yield answered if answered
      when :flat_map
        answered = @callable.call(*values)
        if answered.is_a?(Array)
          answered.each { |element| yield element }
        elsif answered.respond_to?(:each) && answered.respond_to?(:force)
          answered.each { |element| yield element }
        else
          yield answered
        end
      when :compact
        yield(*values) unless packed(values).nil?
      when :grep
        # The block is handed the element the source yielded, which is the
        # values gathered into one where it yielded several.
        subject = packed(values)
        yield(@callable.nil? ? subject : @callable.call(subject)) if @count === subject
      when :grep_v
        subject = packed(values)
        yield(@callable.nil? ? subject : @callable.call(subject)) unless @count === subject
      when :uniq
        key = @callable.nil? ? packed(values) : @callable.call(*values)
        unless kept.include?(key)
          kept.push(key)
          yield(*values)
        end
      when :with_index
        subject = packed(values)
        @callable.call(subject, seen + @count) unless @callable.nil?
        yield subject, seen + @count
        seen += 1
      when :zip
        subject = packed(values)
        alongside = @count.map do |other|
          begin
            other.next
          rescue StopIteration
            nil
          end
        end
        yield [subject] + alongside
        seen += 1
      when :chunk
        subject = packed(values)
        key = @callable.call(subject)
        if grouping && key != group_key
          yield [group_key, grouped]
          grouped = []
        end
        grouping = true
        group_key = key
        grouped.push(subject)
      when :chunk_while
        subject = packed(values)
        if grouping && !@callable.call(previous, subject)
          yield grouped
          grouped = []
        end
        grouping = true
        previous = subject
        grouped.push(subject)
      when :slice_when
        subject = packed(values)
        if grouping && @callable.call(previous, subject)
          yield grouped
          grouped = []
        end
        grouping = true
        previous = subject
        grouped.push(subject)
      when :slice_before
        subject = packed(values)
        if grouping && @callable.call(subject)
          yield grouped
          grouped = []
        end
        grouping = true
        grouped.push(subject)
      when :slice_after
        subject = packed(values)
        grouping = true
        grouped.push(subject)
        if @callable.call(subject)
          yield grouped
          grouped = []
          grouping = false
        end
      when :take
        break if taken >= @count
        taken += 1
        yield(*values)
        break if taken >= @count
      when :take_while
        break unless @callable.call(*values)
        yield(*values)
      when :drop
        if seen < @count
          seen += 1
        else
          yield(*values)
        end
      when :drop_while
        dropping = false if dropping && !@callable.call(*values)
        yield(*values) unless dropping
      end
    end
    if grouping && !grouped.empty?
      case @kind
      when :chunk
        yield [group_key, grouped]
      when :chunk_while, :slice_when, :slice_before, :slice_after
        yield grouped
      end
    end
    self
  end

  # The elements the step below hands up. A Lazy built with a block runs it
  # against a yielder, which is how `Enumerator::Lazy.new(obj) { |y, v| }` reads.
  def walk_source(*args)
    if @transform.nil?
      @source.each(*args) { |*values| yield values }
      return self
    end
    collector = Enumerator::LazyYielder.new
    @source.each(*args) do |*values|
      collector.clear
      @transform.call(collector, *values)
      collector.collected.each { |value| yield [value] }
    end
    self
  end
  private :walk_source

  def take(count)
    raise ArgumentError, "attempt to take negative size" if count < 0
    return Enumerator::Lazy.build(Enumerator::EmptyWalk.new, 0) if count == 0
    with_step(:take, nil, count)
  end

  def first(*count)
    wanted = count.empty? ? 1 : count[0]
    raise ArgumentError, "attempt to take negative size" if wanted < 0
    collected = []
    unless wanted == 0
      each do |*values|
        collected.push(packed(values))
        break if collected.size >= wanted
      end
    end
    return collected[0] if count.empty?
    collected
  end

  def force(*args)
    to_a(*args)
  end

  def to_a(*args)
    collected = []
    each(*args) { |*values| collected.push(packed(values)) }
    collected
  end

  def map(&block)
    raise ArgumentError, "tried to call lazy map without a block" if block.nil?
    with_step(:map, block)
  end

  def collect(&block)
    raise ArgumentError, "tried to call lazy collect without a block" if block.nil?
    with_step(:map, block)
  end

  def select(&block)
    raise ArgumentError, "tried to call lazy select without a block" if block.nil?
    with_step(:select, block)
  end

  def filter(&block)
    raise ArgumentError, "tried to call lazy filter without a block" if block.nil?
    with_step(:select, block)
  end

  def find_all(&block)
    raise ArgumentError, "tried to call lazy find_all without a block" if block.nil?
    with_step(:select, block)
  end

  def reject(&block)
    raise ArgumentError, "tried to call lazy reject without a block" if block.nil?
    with_step(:reject, block)
  end

  def filter_map(&block)
    raise ArgumentError, "tried to call lazy filter_map without a block" if block.nil?
    with_step(:filter_map, block)
  end

  def flat_map(&block)
    raise ArgumentError, "tried to call lazy flat_map without a block" if block.nil?
    with_step(:flat_map, block)
  end

  def collect_concat(&block)
    raise ArgumentError, "tried to call lazy collect_concat without a block" if block.nil?
    with_step(:flat_map, block)
  end

  def compact
    with_step(:compact)
  end

  def take_while(&block)
    raise ArgumentError, "tried to call lazy take_while without a block" if block.nil?
    with_step(:take_while, block)
  end

  def drop(count)
    raise ArgumentError, "attempt to drop negative size" if count < 0
    with_step(:drop, nil, count)
  end

  def drop_while(&block)
    raise ArgumentError, "tried to call lazy drop_while without a block" if block.nil?
    with_step(:drop_while, block)
  end

  def grep(pattern, &block)
    with_step(:grep, block, pattern)
  end

  def grep_v(pattern, &block)
    with_step(:grep_v, block, pattern)
  end

  def uniq(&block)
    with_step(:uniq, block)
  end

  def chunk(&block)
    with_step(:chunk, block)
  end

  def chunk_while(&block)
    raise ArgumentError, "tried to call lazy chunk_while without a block" if block.nil?
    with_step(:chunk_while, block)
  end

  def slice_when(&block)
    raise ArgumentError, "tried to call lazy slice_when without a block" if block.nil?
    with_step(:slice_when, block)
  end

  def slice_before(&block)
    raise ArgumentError, "tried to call lazy slice_before without a block" if block.nil?
    with_step(:slice_before, block)
  end

  def slice_after(&block)
    raise ArgumentError, "tried to call lazy slice_after without a block" if block.nil?
    with_step(:slice_after, block)
  end

  def with_index(offset = 0, &block)
    start = offset.nil? ? 0 : offset
    unless start.is_a?(Integer)
      raise TypeError, "no implicit conversion of #{start.class} into Integer"
    end
    with_step(:with_index, block, start)
  end

  # A lazy zip pulls one value at a time from the others, so a walk with no
  # end can still be zipped against.
  def zip(*others, &block)
    # Ruby treats a lazy zip given a block as the eager one, walking every
    # element and answering nil rather than a lazy enumerator.
    unless block.nil?
      each_with_index do |*values, index|
        subject = values.size == 1 ? values[0] : values
        alongside = others.map { |other| other.to_a[index] }
        block.call([subject] + alongside)
      end
      return nil
    end
    stepped = others.map do |other|
      unless other.respond_to?(:to_a)
        raise TypeError, "wrong argument type #{other.class} (must respond to :each)"
      end
      other.respond_to?(:next) ? other : other.to_enum(:each)
    end
    with_step(:zip, nil, stepped)
  end

  # The elements pulled out of the walk so far. `next` and `peek` read from
  # here so the walk runs no further than what was asked for.
  def pulled(wanted)
    @pulled = [] if @pulled.nil?
    @pulled = first(wanted) if @pulled.size < wanted
    @pulled
  end
  private :pulled

  def peek
    @walked = 0 if @walked.nil?
    values = pulled(@walked + 1)
    raise StopIteration, "iteration reached an end" if values.size <= @walked
    values[@walked]
  end

  def next
    value = peek
    @walked += 1
    value
  end

  def rewind
    @walked = 0
    @pulled = nil
    self
  end

  def inspect
    return "#<#{self.class}: uninitialized>" if @source.nil?
    "#<Enumerator::Lazy: #{@source.inspect}>"
  end
end

# Collects what a `Enumerator::Lazy.new(obj) { |yielder, *values| }` block hands over.
class Enumerator::LazyYielder
  def initialize
    @collected = []
  end

  def collected
    @collected
  end

  def clear
    @collected = []
  end

  def <<(value)
    @collected.push(value)
    self
  end

  def yield(*values)
    @collected.push(values.size == 1 ? values[0] : values)
    nil
  end
end

# A source with nothing in it, which `lazy.take(0)` walks."##;
