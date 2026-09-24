pub(super) const SOURCE: &str = r##"
class Numeric
  # `1.step(10, 3)` walks 1, 4, 7, 10. Without a block it answers the
  # sequence itself, which is what carries the walk about.
  def step(limit = nil, by = nil, **options, &block)
    unknown = options.keys - [:to, :by]
    raise ArgumentError, "unknown keyword: #{unknown[0].inspect}" unless unknown.empty?
    raise ArgumentError, "to is given twice" if !limit.nil? && options.key?(:to)
    raise ArgumentError, "step is given twice" if !by.nil? && options.key?(:by)
    named_by = options[:by]
    step_given = !by.nil? || !named_by.nil?
    walked_by = by.nil? ? (named_by.nil? ? 1 : named_by) : by
    raise ArgumentError, "step can\'t be 0" if walked_by == 0
    walked_to = limit.nil? ? options[:to] : limit
    unless walked_by.is_a?(Numeric)
      # Ruby reads the step's direction by comparing it against zero, so a
      # step that cannot be compared is refused in those words, and refused
      # at the point the walk is asked for rather than when it is built.
      refusal = "comparison of #{walked_by.class} with 0 failed"
      raise ArgumentError, refusal unless block.nil?
      return Enumerator.over(self, :step, [walked_to, walked_by]).__refuse_size__(refusal)
    end
    sequence = Enumerator::ArithmeticSequence.send(
      :new, self, walked_to, walked_by, false, nil, "step", step_given
    )
    return sequence if block.nil?
    sequence.each { |value| block.call(value) }
    self
  end
end

class Exception
  # Whether a report written to stderr would reach a terminal, which is what
  # decides if it is painted.
  def self.to_tty?
    $stderr.tty?
  end
end

class Range
  # A range built by hand out of `allocate`. Every range written as a literal
  # is frozen the moment it is made, so this is the only object whose ends
  # can still be written.
  def initialize(first, last, excludes = false)
    raise FrozenError, "can't modify frozen Range: #{inspect}" if frozen?
    if !first.nil? && !last.nil? && (first <=> last).nil?
      raise ArgumentError, "bad value for range"
    end
    @begin = first
    @end = last
    @excludes = excludes ? true : false
    self
  end
  private :initialize

  def begin
    @begin
  end

  def end
    @end
  end

  def exclude_end?
    @excludes.nil? ? false : @excludes
  end

  # Both ends at once. Without a block the ends name themselves, so a range
  # over anything with a `succ` is not walked to find them.
  def minmax(&block)
    return to_a.minmax(&block) unless block.nil?
    [min, max]
  end

  # `(1..10).step(3)` walks 1, 4, 7, 10, and `%` is written for the same
  # thing. Without a block either one answers the sequence itself.
  def step(by = nil, &block)
    stepped(by, "step", &block)
  end

  def %(by = nil, &block)
    stepped(by, "%", &block)
  end

  # Walks the range from its end down to its beginning. A range counting
  # integers can start past any beginning, while one walking with `succ`
  # needs both ends.
  def reverse_each(&block)
    last = self.end
    first = self.begin
    raise TypeError, "can't iterate from NilClass" if last.nil?
    counts = (first.is_a?(Integer) && last.is_a?(Numeric)) ||
      (first.nil? && last.is_a?(Integer))
    walks = !first.nil? && reverse_walkable(last) && reverse_walkable(first)
    unless counts || walks
      named = reverse_walkable(last) ? first.class : last.class
      raise TypeError, "can't iterate from #{named}"
    end
    return Enumerator.over(self, :reverse_each, [], reverse_walk_size) if block.nil?
    if counts
      value = last.is_a?(Integer) ? last : last.floor
      value -= 1 if exclude_end? && value == last
      while first.nil? || value >= first
        block.call(value)
        value -= 1
      end
      return self
    end
    to_a.reverse_each { |value| block.call(value) }
    self
  end

  # Whether a value walks with `succ` the way a String or a Symbol does. A
  # number counts instead, and a range of them is walked by counting.
  def reverse_walkable(value)
    return false if value.is_a?(Numeric)
    value.is_a?(String) || value.is_a?(Symbol) || value.respond_to?(:succ)
  end
  private :reverse_walkable

  # How many elements a reverse walk reports before it starts. A walk with no
  # beginning never ends, and one over anything but numbers reports nothing.
  def reverse_walk_size
    return Float::INFINITY if self.begin.nil?
    size
  end
  private :reverse_walk_size

  def stepped(by, written_as, &block)
    first = self.begin
    last = self.end
    return numeric_stepped(first, last, by, written_as, &block) if numeric_ends?(first, last)
    if first.nil?
      raise ArgumentError, "step is required for non-numeric ranges" if by.nil?
      raise ArgumentError, "#step for non-numeric beginless ranges is meaningless"
    end
    counts = first.respond_to?(:succ) && (by.nil? || by.is_a?(Integer))
    raise ArgumentError, "step is required for non-numeric ranges" if by.nil? && !counts
    return Enumerator.over(self, written_as.to_sym, [by]) if block.nil?
    if counts
      walk_by_succ(last, by.nil? ? 1 : by, &block)
    else
      walk_by_sum(first, last, by, &block)
    end
    self
  end
  private :stepped

  # Whether the walk counts its way along, which is what a range over numbers
  # does. A range with nothing at either end names no numbers to count.
  def numeric_ends?(first, last)
    return false if first.nil? && last.nil?
    (first.nil? || first.is_a?(Numeric)) && (last.nil? || last.is_a?(Numeric))
  end
  private :numeric_ends?

  def numeric_stepped(first, last, by, written_as, &block)
    unless by.nil? || by.is_a?(Numeric)
      # A step of some other kind is read through `coerce`, and only where the
      # walk is about to run, so asking for the walk alone refuses nothing.
      return Enumerator.over(self, written_as.to_sym, [by]) if block.nil?
      by = coerced_step(first, by)
    end
    raise ArgumentError, "step can\'t be 0" if by == 0
    sequence = Enumerator::ArithmeticSequence.send(
      :new, first, last, by.nil? ? 1 : by, exclude_end?, self, written_as, !by.nil?
    )
    return sequence if block.nil?
    sequence.each { |value| block.call(value) }
    self
  end
  private :numeric_stepped

  # The number a step of another kind stands for, which it names by answering
  # `coerce` with the walk's own beginning alongside it.
  def coerced_step(first, by)
    unless by.respond_to?(:coerce)
      raise TypeError, "#{by.class} can\'t be coerced into #{first.class}"
    end
    by.coerce(first)[1]
  end
  private :coerced_step

  # A walk over anything with a `succ`, handing out every step\'th value it
  # reaches. A step of none or less hands out the first and no more, since the
  # count it waits on never comes round again.
  def walk_by_succ(last, count, &block)
    value = self.begin
    remaining = 0
    loop do
      unless last.nil?
        comparison = value <=> last
        break if comparison.nil?
        break if comparison > 0
        break if comparison == 0 && exclude_end?
      end
      if remaining == 0
        block.call(value)
        remaining = count
      end
      remaining = remaining - 1
      value = value.succ
    end
  end
  private :walk_by_succ

  # A walk over anything answering `+`, which is what a step that is not a
  # count is added with. The range\'s direction is read first, and a step
  # running against it reaches nothing at all.
  def walk_by_sum(first, last, by, &block)
    if last.nil?
      value = first
      loop do
        block.call(value)
        value = value + by
      end
    end
    return if (first <=> last).nil?
    direction = first <=> last
    return unless direction == 0 || (first <=> first + by) == direction
    value = first
    loop do
      comparison = value <=> last
      break if comparison.nil?
      break if direction < 0 && comparison > 0
      break if direction > 0 && comparison < 0
      break if comparison == 0 && exclude_end?
      block.call(value)
      break if comparison == 0
      value = value + by
    end
  end
  private :walk_by_sum
end

class Array
  # The array an object stands for, or nil where it stands for none. Only an
  # object answering `to_ary` is asked.
  def self.try_convert(held)
    return held if held.is_a?(Array)
    return nil unless held.respond_to?(:to_ary)
    converted = held.to_ary
    return converted if converted.nil? || converted.is_a?(Array)
    raise TypeError,
      "can't convert #{held.class} into Array (#{held.class}#to_ary gives #{converted.class})"
  end

  # The number an index or a length arrives as, which Ruby reads through
  # `to_int` and refuses when the object names none.
  def fill_count(given)
    return fill_machine_word(given) if given.is_a?(Integer)
    unless given.respond_to?(:to_int)
      raise TypeError, "no implicit conversion of #{given.class} into Integer"
    end
    read = given.to_int
    unless read.is_a?(Integer)
      raise TypeError, "can\'t convert #{given.class} to Integer (#{given.class}#to_int gives #{read.class})"
    end
    fill_machine_word(read)
  end
  private :fill_count

  # A count Ruby reads into a machine word, which a number too large for one
  # is refused as.
  def fill_machine_word(read)
    if read > 9223372036854775807 || read < -9223372036854775808
      raise RangeError, "bignum too big to convert into 'long'"
    end
    read
  end
  private :fill_machine_word

  # The most elements an array can hold, which is what a machine word counts
  # divided by the room one element takes.
  ARRAY_LIMIT = 1152921504606846975
  private_constant :ARRAY_LIMIT

  # `fill` writes over a stretch of the array: everything, from an index on,
  # a run of a given length, or the span a Range names. A block is handed each
  # index and writes what it answers.
  def fill(*given, &block)
    limit = block.nil? ? 3 : 2
    if given.size > limit
      raise ArgumentError, "wrong number of arguments (given #{given.size}, expected #{block.nil? ? "1..3" : "0..2"})"
    end
    if block.nil?
      raise ArgumentError, "wrong number of arguments (given 0, expected 1..3)" if given.empty?
      value = given.shift
    else
      value = nil
    end
    if given.empty?
      from = 0
      count = size
    elsif given[0].is_a?(Range)
      raise TypeError, "wrong number of arguments (given 3, expected 1..2)" if given.size > 1
      span = given[0]
      from = span.begin.nil? ? 0 : fill_count(span.begin)
      from += size if from < 0
      raise RangeError, "#{span} out of range" if from < 0
      endless = span.end.nil?
      last = endless ? size - 1 : fill_count(span.end)
      last += size if last < 0
      last -= 1 if span.exclude_end? && !endless
      count = last - from + 1
      count = 0 if count < 0
    else
      from = given[0].nil? ? 0 : fill_count(given[0])
      from += size if from < 0
      from = 0 if from < 0
      if given.size > 1 && !given[1].nil?
        count = fill_count(given[1])
      else
        count = size - from
      end
    end
    return self if count <= 0
    raise ArgumentError, "argument too big" if from > ARRAY_LIMIT - count
    index = from
    stop = from + count
    while index < stop
      self[index] = block.nil? ? value : block.call(index)
      index += 1
    end
    self
  end

  # Array#to_h names the index of the element it refused, which the shared
  # Enumerable version has no position to report.
  def to_h(&block)
    built = {}
    index = 0
    while index < size
      element = self[index]
      pair = block.nil? ? element : block.call(element)
      unless pair.is_a? Array
        converted = pair.respond_to?(:to_ary) ? pair.to_ary : nil
        unless converted.is_a? Array
          raise TypeError, "wrong element type #{pair.class} at #{index} (expected array)"
        end
        pair = converted
      end
      unless pair.size == 2
        raise ArgumentError, "wrong array length at #{index} (expected 2, was #{pair.size})"
      end
      built[pair[0]] = pair[1]
      index += 1
    end
    built
  end

  # The length a walk was asked for, which may arrive as a Float or as an
  # object that converts to an Integer.
  def walk_length(count)
    return count if count.is_a?(Integer)
    return count.to_i if count.is_a?(Float)
    count.to_int
  end
  private :walk_length

  # How many ways `wanted` elements can be picked out of `total` when the
  # order they are picked in does not matter.
  def binomial(total, wanted)
    return 0 if wanted < 0 || wanted > total
    answered = 1
    step = 0
    while step < wanted
      answered = answered * (total - step) / (step + 1)
      step += 1
    end
    answered
  end
  private :binomial

  # How many ways `wanted` elements can be picked out of `total` when the
  # order they are picked in matters.
  def descending_factorial(total, wanted)
    return 0 if wanted < 0 || wanted > total
    answered = 1
    step = 0
    while step < wanted
      answered *= total - step
      step += 1
    end
    answered
  end
  private :descending_factorial

  # The Enumerator a walk hands back when it was called without its block,
  # carrying the count of what the walk would have yielded.
  def sized_walk(name, args, reach)
    Enumerator.over(self, name, args, reach)
  end
  private :sized_walk

  def combination(count)
    wanted = walk_length(count)
    return sized_walk(:combination, [wanted], binomial(size, wanted)) unless block_given?
    walked = dup
    if wanted >= 0 && wanted <= walked.size
      pick_combination(walked, wanted, 0, [], false) { |picked| yield picked }
    end
    self
  end

  def repeated_combination(count)
    wanted = walk_length(count)
    reach = binomial(size + wanted - 1, wanted)
    reach = 1 if wanted == 0
    reach = 0 if wanted < 0
    return sized_walk(:repeated_combination, [wanted], reach) unless block_given?
    walked = dup
    if wanted == 0
      yield []
    elsif wanted > 0 && walked.size > 0
      pick_combination(walked, wanted, 0, [], true) { |picked| yield picked }
    end
    self
  end

  def pick_combination(source, wanted, start, picked, repeating, &block)
    if picked.size == wanted
      block.call(picked.dup)
      return
    end
    index = start
    while index < source.size
      picked.push(source[index])
      pick_combination(source, wanted, repeating ? index : index + 1, picked, repeating, &block)
      picked.pop
      index += 1
    end
  end
  private :pick_combination

  def permutation(count = nil)
    wanted = count.nil? ? size : walk_length(count)
    unless block_given?
      return sized_walk(:permutation, count.nil? ? [] : [count], descending_factorial(size, wanted))
    end
    walked = dup
    if wanted >= 0 && wanted <= walked.size
      pick_permutation(walked, wanted, [], []) { |picked| yield picked }
    end
    self
  end

  def pick_permutation(source, wanted, used, picked, &block)
    if picked.size == wanted
      block.call(picked.dup)
      return
    end
    index = 0
    while index < source.size
      unless used[index]
        used[index] = true
        picked.push(source[index])
        pick_permutation(source, wanted, used, picked, &block)
        picked.pop
        used[index] = false
      end
      index += 1
    end
  end
  private :pick_permutation

  def repeated_permutation(count)
    wanted = walk_length(count)
    reach = wanted < 0 ? 0 : size ** wanted
    return sized_walk(:repeated_permutation, [wanted], reach) unless block_given?
    walked = dup
    pick_repeated_permutation(walked, wanted, []) { |picked| yield picked } if wanted >= 0
    self
  end

  def pick_repeated_permutation(source, wanted, picked, &block)
    if picked.size == wanted
      block.call(picked.dup)
      return
    end
    source.each do |element|
      picked.push(element)
      pick_repeated_permutation(source, wanted, picked, &block)
      picked.pop
    end
  end
  private :pick_repeated_permutation

  def product(*lists)
    walked = [self]
    lists.each { |other| walked.push(array_argument(other)) }
    total = 1
    walked.each { |other| total *= other.size }
    raise RangeError, "too big to product" if total > 1073741823
    unless block_given?
      collected = []
      pick_product(walked, 0, []) { |row| collected.push(row) }
      return collected
    end
    pick_product(walked, 0, []) { |row| yield row }
    self
  end

  def pick_product(lists, index, picked, &block)
    if index == lists.size
      block.call(picked.dup)
      return
    end
    lists[index].each do |element|
      picked.push(element)
      pick_product(lists, index + 1, picked, &block)
      picked.pop
    end
  end
  private :pick_product

  # An argument a walk reads as a list, reached through `to_ary` when it is
  # not one already.
  def array_argument(other)
    return other if other.is_a?(Array)
    converted = begin
      other.to_ary
    rescue NoMethodError
      nil
    end
    unless converted.is_a?(Array)
      raise TypeError, "no implicit conversion of #{other.class} into Array"
    end
    converted
  end
  private :array_argument

  # The element a block picks out of a sorted array, halving the search each
  # time rather than walking it.
  def bsearch
    return Enumerator.over(self, :bsearch, [], nil) unless block_given?
    found = nil
    low = 0
    high = size
    while low < high
      middle = (low + high) / 2
      answered = yield self[middle]
      if answered == true
        found = middle
        high = middle
      elsif answered == false || answered.nil?
        low = middle + 1
      elsif answered.is_a?(Integer) || answered.is_a?(Float)
        return self[middle] if answered == 0
        if answered < 0
          high = middle
        else
          low = middle + 1
        end
      else
        raise TypeError, "wrong argument type #{answered.class} (must be numeric, true, false or nil)"
      end
    end
    found.nil? ? nil : self[found]
  end

  def bsearch_index
    return Enumerator.over(self, :bsearch_index, [], nil) unless block_given?
    found = nil
    low = 0
    high = size
    while low < high
      middle = (low + high) / 2
      answered = yield self[middle]
      if answered == true
        found = middle
        high = middle
      elsif answered == false || answered.nil?
        low = middle + 1
      elsif answered.is_a?(Integer) || answered.is_a?(Float)
        return middle if answered == 0
        if answered < 0
          high = middle
        else
          low = middle + 1
        end
      else
        raise TypeError, "wrong argument type #{answered.class} (must be numeric, true, false or nil)"
      end
    end
    found
  end
end

class Set
  # `Set.new` builds the set itself, and the hook behind it is private the
  # way every other `initialize` is.
  def initialize(enumerable = nil, &block)
    return self if enumerable.nil?
    unless enumerable.respond_to?(:each_entry) || enumerable.respond_to?(:each)
      raise ArgumentError, "value must be enumerable"
    end
    walked = enumerable.respond_to?(:each_entry) ? :each_entry : :each
    enumerable.send(walked) do |entry|
      add(block.nil? ? entry : block.call(entry))
    end
    self
  end
  private :initialize

  # The elements grouped into sets under whatever the block answers for each
  # of them.
  def classify
    return Enumerator.over(self, :classify, [], size) unless block_given?
    grouped = Hash.new { |held, key| held[key] = Set.new }
    each { |element| grouped[yield element].add(element) }
    grouped
  end

  # The set of subsets a block cuts self into. A block of one parameter groups
  # by what it answers, and a block of two groups the elements it relates.
  def divide(&block)
    return Enumerator.over(self, :divide, [], size) if block.nil?
    return Set.new(classify { |element| block.call(element) }.values) unless block.arity == 2
    walked = to_a
    group_of = {}
    walked.each_with_index { |element, index| group_of[index] = index }
    walked.each_with_index do |left, left_index|
      walked.each_with_index do |right, right_index|
        next if left_index == right_index
        merge_groups(group_of, left_index, right_index) if block.call(left, right)
      end
    end
    grouped = {}
    walked.each_with_index do |element, index|
      root = group_root(group_of, index)
      grouped[root] = Set.new unless grouped.key?(root)
      grouped[root].add(element)
    end
    Set.new(grouped.values)
  end

  def group_root(group_of, index)
    walked = index
    walked = group_of[walked] while group_of[walked] != walked
    walked
  end
  private :group_root

  def merge_groups(group_of, left, right)
    left_root = group_root(group_of, left)
    right_root = group_root(group_of, right)
    return if left_root == right_root
    group_of[right_root] = left_root
  end
  private :merge_groups

  # A copy of self with every set it holds opened up into its own elements.
  def flatten
    Set.new(flattened_elements(self, []))
  end

  # Opens up every set self holds, answering self when that changed anything
  # and nil when it did not.
  def flatten!
    flattened = flattened_elements(self, [])
    return nil if flattened.size == size && flattened.all? { |element| include?(element) }
    replace(Set.new(flattened))
    self
  end

  def flattened_elements(source, walking)
    if walking.any? { |held| held.equal?(source) }
      raise ArgumentError, "tried to flatten recursive Set"
    end
    walking.push(source)
    collected = []
    source.each do |element|
      if element.is_a?(Set)
        flattened_elements(element, walking).each { |held| collected.push(held) }
      else
        collected.push(element)
      end
    end
    walking.pop
    collected
  end
  private :flattened_elements
end

class Hash
  # The hash an object stands for, or nil where it stands for none. Only an
  # object answering `to_hash` is asked.
  def self.try_convert(held)
    return held if held.is_a?(Hash)
    return nil unless held.respond_to?(:to_hash)
    converted = held.to_hash
    return converted if converted.nil? || converted.is_a?(Hash)
    raise TypeError,
      "can't convert #{held.class} into Hash (#{held.class}#to_hash gives #{converted.class})"
  end

  # The hook behind `Hash.new`, which a subclass reaches through `super` and
  # a program may call again to set a new default. The pairs already stored
  # are left alone.
  def initialize(*arguments, &block)
    raise FrozenError, "can't modify frozen Hash: #{inspect}" if frozen?
    if arguments.size > 1
      raise ArgumentError, "wrong number of arguments (given #{arguments.size}, expected 0..1)"
    end
    unless block.nil?
      unless arguments.empty?
        raise ArgumentError, "wrong number of arguments (given #{arguments.size}, expected 0)"
      end
      self.default = nil
      self.default_proc = block
      return self
    end
    self.default_proc = nil
    self.default = arguments.empty? ? nil : arguments[0]
    self
  end
  private :initialize

  # A lambda that reads one key out of the hash, which is what `&hash` passes
  # to a method expecting a block.
  def to_proc
    stored = self
    lambda { |key| stored[key] }
  end
end

# A value object built from a fixed set of members. Once made, it cannot be
# changed, and `with` answers a fresh one carrying the changes.
class Data
  def self.define(*names, &body)
    members = []
    names.each do |name|
      unless name.is_a?(Symbol) || name.is_a?(String)
        raise TypeError, "#{name.inspect} is not a symbol nor a string"
      end
      named = name.to_sym
      raise ArgumentError, "duplicate member: #{named}" if members.include?(named)
      members.push(named)
    end
    built = Class.new(self)
    built.define_singleton_method(:members) { members }
    built.class_eval do
      def self.new(*args, **kwargs)
        made = allocate
        made.send(:initialize, **Data.paired_members(members, args, kwargs))
        made
      end

      def self.[](*args, **kwargs)
        # `Measure[amount: 42, unit: "km"]` names its members the way `new`
        # does, so a lone hash whose keys are all members reads as those.
        if kwargs.empty? && args.size == 1 && args[0].is_a?(Hash) &&
           !members.empty? && args[0].keys.all? { |key| members.include?(key.to_sym) }
          return new(**args[0])
        end
        new(*args, **kwargs)
      end
    end
    members.each do |named|
      built.define_method(named) { instance_variable_get("@#{named}") }
    end
    built.class_eval(&body) unless body.nil?
    built
  end

  # Positional arguments read as the members they line up with, so
  # `new(42, "km")` reaches `initialize` the same way `new(amount: 42)` does.
  def self.paired_members(members, args, kwargs)
    return kwargs if args.empty?
    unless kwargs.empty?
      raise ArgumentError, "wrong number of arguments (given #{args.size}, expected 0)"
    end
    if args.size > members.size
      raise ArgumentError,
            "wrong number of arguments (given #{args.size}, expected 0..#{members.size})"
    end
    paired = {}
    args.each_with_index { |value, index| paired[members[index]] = value }
    paired
  end

  def initialize(**kwargs)
    members = self.class.members
    given = {}
    kwargs.each do |key, value|
      named = key.to_sym
      raise ArgumentError, "unknown keyword: #{named.inspect}" unless members.include?(named)
      given[named] = value
    end
    missing = members.reject { |named| given.key?(named) }
    unless missing.empty?
      word = missing.size == 1 ? "keyword" : "keywords"
      raise ArgumentError, "missing #{word}: #{missing.map { |named| named.inspect }.join(", ")}"
    end
    members.each { |named| instance_variable_set("@#{named}", given[named]) }
    self.freeze
    self
  end
  private :initialize

  def members
    self.class.members
  end

  def deconstruct
    self.class.members.map { |named| instance_variable_get("@#{named}") }
  end

  def to_h
    collected = {}
    self.class.members.each do |named|
      value = instance_variable_get("@#{named}")
      unless block_given?
        collected[named] = value
        next
      end
      pair = yield named, value
      unless pair.is_a?(Array)
        converted = pair.respond_to?(:to_ary) ? pair.to_ary : nil
        unless converted.is_a?(Array)
          raise TypeError, "wrong element type #{pair.class} (expected array)"
        end
        pair = converted
      end
      unless pair.size == 2
        raise ArgumentError, "element has wrong array length (expected 2, was #{pair.size})"
      end
      collected[pair[0]] = pair[1]
    end
    collected
  end

  def deconstruct_keys(keys)
    return to_h if keys.nil?
    unless keys.is_a?(Array)
      raise TypeError, "wrong argument type #{keys.class} (expected Array or nil)"
    end
    members = self.class.members
    return {} if keys.size > members.size
    collected = {}
    keys.each do |key|
      if key.is_a?(Symbol) || key.is_a?(String)
        named = key.to_sym
        return collected unless members.include?(named)
        collected[key] = instance_variable_get("@#{named}")
      else
        index = member_position(key)
        return collected if index >= members.size || index < -members.size
        collected[key] = instance_variable_get("@#{members[index]}")
      end
    end
    collected
  end

  # A member named by its position rather than by name, which pattern
  # matching uses when it deconstructs into an array.
  def member_position(key)
    return key if key.is_a?(Integer)
    unless key.respond_to?(:to_int)
      raise TypeError, "no implicit conversion of #{key.class} into Integer"
    end
    converted = key.to_int
    unless converted.is_a?(Integer)
      raise TypeError, "can't convert #{key.class} into Integer"
    end
    converted
  end
  private :member_position

  def with(*args, **kwargs)
    unless args.empty?
      raise ArgumentError, "wrong number of arguments (given #{args.size}, expected 0)"
    end
    return self if kwargs.empty?
    made = self.class.allocate
    made.send(:initialize, **to_h.merge(kwargs))
    made
  end

  def ==(other)
    same_members?(other) { |mine, theirs| mine == theirs }
  end

  def eql?(other)
    same_members?(other) { |mine, theirs| mine.eql?(theirs) }
  end

  # Whether two data objects hold the same class and the same values. A
  # member that is the object itself on both sides matches, which is how a
  # structure that reaches itself is compared without running forever.
  def same_members?(other)
    return true if equal?(other)
    return false unless other.is_a?(Data)
    return false unless other.class == self.class
    self.class.members.each do |named|
      mine = instance_variable_get("@#{named}")
      theirs = other.instance_variable_get("@#{named}")
      next if mine.equal?(self) && theirs.equal?(other)
      return false unless yield(mine, theirs)
    end
    true
  end
  private :same_members?

  def hash
    parts = [self.class.object_id]
    self.class.members.each do |named|
      value = instance_variable_get("@#{named}")
      parts.push(value.equal?(self) ? 0 : value.hash)
    end
    parts.hash
  end

  def inspect
    data_inspect([])
  end

  def to_s
    data_inspect([])
  end

  # The rendering, with the walk so far so a member that reaches back to the
  # object it is held by is shown as the class alone.
  def data_inspect(walking)
    label = self.class.inspect
    anonymous = label.start_with?("#<Class:")
    return "#<data #{label}:...>" if walking.any? { |held| held.equal?(self) }
    walking.push(self)
    parts = self.class.members.map do |named|
      value = instance_variable_get("@#{named}")
      shown = value.is_a?(Data) ? value.data_inspect(walking) : value.inspect
      "#{named}=#{shown}"
    end
    walking.pop
    body = parts.join(", ")
    if anonymous
      parts.empty? ? "#<data>" : "#<data #{body}>"
    else
      parts.empty? ? "#<data #{label}>" : "#<data #{label} #{body}>"
    end
  end
end

# A point in time, held as a whole number of seconds since the epoch plus an
# exact fraction of a second. The calendar fields come from the C library, so
# a local time follows the zone rules the operating system holds."##;
