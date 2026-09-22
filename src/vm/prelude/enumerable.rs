pub(super) const SOURCE: &str = r##"
module Enumerable
  # What `each` yielded, as one value: a lone value stays itself, and several
  # gather into an array the way Ruby packs a multi-value yield.
  def packed(values)
    return nil if values.empty?
    values.size == 1 ? values[0] : values
  end
  private :packed

  # The Enumerator a method hands back when it was called without its block,
  # carrying the receiver's own size so `enum.size` answers without walking
  # the values. A receiver that reports no size leaves the Enumerator's nil.
  def sized_enum(name, *args)
    Enumerator.over(self, name, args, respond_to?(:size) ? size : nil)
  end
  private :sized_enum

  def to_a(*args)
    collected = []
    each(*args) { |*values| collected.push(packed(values)) }
    collected
  end

  def entries(*args)
    to_a(*args)
  end

  def each_entry(*args)
    return sized_enum(:each_entry, *args) unless block_given?
    each(*args) { |*values| yield(packed(values)) }
    self
  end

  # The walk hands the block whatever `each` yields, and the block written
  # for the walk decides how many of those values it takes. The inner block
  # takes the same count, so an `each` that reads the block it was handed
  # sees the shape the caller wrote.
  def map(&block)
    return sized_enum(:map) if block.nil?
    collected = []
    case block.arity
    when 1
      each { |one| collected.push(block.call(one)) }
    when 2
      each { |one, two| collected.push(block.call(one, two)) }
    else
      each { |*values| collected.push(block.call(*values)) }
    end
    collected
  end

  def collect(&block)
    return sized_enum(:collect) unless block_given?
    map(&block)
  end

  def flat_map
    return sized_enum(:flat_map) unless block_given?
    collected = []
    each do |*values|
      answered = yield(packed(values))
      flattened = answered.is_a?(Array) ? answered : nil
      if flattened.nil? && answered.respond_to?(:to_ary)
        flattened = answered.to_ary
        unless flattened.nil? || flattened.is_a?(Array)
          raise TypeError, "can't convert #{answered.class} to Array"
        end
      end
      if flattened.nil?
        collected.push(answered)
      else
        flattened.each { |element| collected.push(element) }
      end
    end
    collected
  end

  def collect_concat(&block)
    return sized_enum(:collect_concat) unless block_given?
    flat_map(&block)
  end

  def select
    return sized_enum(:select) unless block_given?
    kept = []
    each { |*values| kept.push(packed(values)) if yield(packed(values)) }
    kept
  end

  def filter(&block)
    return sized_enum(:filter) unless block_given?
    select(&block)
  end

  def find_all(&block)
    return sized_enum(:find_all) unless block_given?
    select(&block)
  end

  def filter_map
    return to_enum(:filter_map) unless block_given?
    kept = []
    each do |*values|
      answered = yield(packed(values))
      kept.push(answered) if answered
    end
    kept
  end

  def reject
    return sized_enum(:reject) unless block_given?
    kept = []
    each { |*values| kept.push(packed(values)) unless yield(packed(values)) }
    kept
  end

  def find(ifnone = nil)
    return to_enum(:find, ifnone) unless block_given?
    each do |*values|
      element = packed(values)
      return element if yield(element)
    end
    ifnone.nil? ? nil : ifnone.call
  end

  def detect(ifnone = nil, &block)
    return to_enum(:detect, ifnone) unless block_given?
    find(ifnone, &block)
  end

  def find_index(*wanted)
    check_predicate_arguments(wanted, block_given?)
    return to_enum(:find_index) if wanted.empty? && !block_given?
    index = 0
    each do |*values|
      if wanted.empty?
        return index if yield(*values)
      elsif packed(values) == wanted[0]
        return index
      end
      index += 1
    end
    nil
  end

  # The elements a pattern matches with `===`, passed through the block when
  # one is given. `grep_v` keeps the ones it does not match.
  # Without a block the walk leaves the last match where it found it, so a
  # `$~` set before the call still reads the same afterwards.
  def grep(pattern)
    kept = []
    saved = $~
    each do |*values|
      element = packed(values)
      if pattern === element
        kept.push(block_given? ? yield(element) : element)
      end
    end
    $~ = saved unless block_given?
    kept
  end

  def grep_v(pattern)
    kept = []
    saved = $~
    each do |*values|
      element = packed(values)
      unless pattern === element
        kept.push(block_given? ? yield(element) : element)
      end
    end
    $~ = saved unless block_given?
    kept
  end

  def include?(wanted)
    each do |*values|
      element = packed(values)
      return true if element == wanted
    end
    false
  end

  def member?(wanted)
    include?(wanted)
  end

  def count(*wanted)
    check_predicate_arguments(wanted, block_given?)
    counted = 0
    each do |*values|
      if !wanted.empty?
        counted += 1 if packed(values) == wanted[0]
      elsif block_given?
        counted += 1 if yield(*values)
      else
        counted += 1
      end
    end
    counted
  end

  def first(*count)
    return take(count[0]) unless count.empty?
    answer = nil
    each do |*values|
      answer = packed(values)
      break
    end
    answer
  end

  def take(count)
    count = count.to_int if !count.is_a?(Integer) && count.respond_to?(:to_int)
    raise TypeError, "no implicit conversion into Integer" unless count.is_a?(Integer)
    raise ArgumentError, "attempt to take negative size" if count < 0
    if count > 9223372036854775807
      raise RangeError, "bignum too big to convert into 'long'"
    end
    collected = []
    return collected if count == 0
    each do |*values|
      collected.push(packed(values))
      break if collected.size >= count
    end
    collected
  end

  def take_while
    return to_enum(:take_while) unless block_given?
    collected = []
    each do |*values|
      break unless yield(*values)
      collected.push(packed(values))
    end
    collected
  end

  def drop(count)
    count = count.to_int if !count.is_a?(Integer) && count.respond_to?(:to_int)
    raise TypeError, "no implicit conversion into Integer" unless count.is_a?(Integer)
    raise ArgumentError, "attempt to drop negative size" if count < 0
    collected = []
    index = 0
    each do |*values|
      collected.push(packed(values)) if index >= count
      index += 1
    end
    collected
  end

  def drop_while
    return to_enum(:drop_while) unless block_given?
    collected = []
    dropping = true
    each do |*values|
      element = packed(values)
      dropping = false if dropping && !yield(element)
      collected.push(element) unless dropping
    end
    collected
  end

  def each_with_index(*args)
    return sized_enum(:each_with_index, *args) unless block_given?
    index = 0
    each(*args) do |*values|
      yield(packed(values), index)
      index += 1
    end
    self
  end

  def each_with_object(memo)
    return sized_enum(:each_with_object, memo) unless block_given?
    each { |*values| yield(packed(values), memo) }
    memo
  end

  # The last argument names an operator when there are two of them, or when
  # there is one and no block. The remaining argument is the starting value,
  # and without one the first element starts the walk.
  def inject(*args)
    raise ArgumentError, "wrong number of arguments (given #{args.size}, expected 0..2)" if args.size > 2
    names_operator = args.size == 2 || (args.size == 1 && !block_given?)
    operator = names_operator ? args.last : nil
    unless operator.nil?
      unless operator.is_a?(Symbol) || operator.is_a?(String)
        unless operator.respond_to?(:to_str)
          raise TypeError, "#{operator.inspect} is not a symbol nor a string"
        end
        operator = operator.to_str
      end
    end
    warn "given block not used", uplevel: 1 if args.size == 2 && block_given?
    raise ArgumentError, "wrong number of arguments (given 0, expected 1)" if operator.nil? && !block_given?
    seeded = args.size == 2 || (args.size == 1 && operator.nil?)
    memo = seeded ? args[0] : nil
    started = seeded
    each do |*values|
      element = packed(values)
      if !started
        memo = element
        started = true
      elsif operator.nil?
        memo = yield(memo, element)
      else
        memo = memo.send(operator, element)
      end
    end
    memo
  end

  alias reduce inject

  # Floats are added with Kahan-Babuska compensation, which is what keeps
  # `[2.78, 5.0, 2.5, ...].sum` on 50.0 where a running total drifts.
  def sum(*start)
    total = start.empty? ? 0 : start[0]
    compensation = 0.0
    each do |*values|
      element = block_given? ? yield(packed(values)) : packed(values)
      compensated = total.is_a?(Float) || element.is_a?(Float)
      element = element.to_f if compensated && element.is_a?(Integer)
      compensated = false unless element.is_a?(Float)
      if compensated
        total = total.to_f unless total.is_a?(Float)
        running = total + element
        # An infinity or a NaN has no rounding to make up for, and the
        # difference the compensation is built from would be a NaN.
        if running.finite?
          if total.abs >= element.abs
            compensation = compensation + ((total - running) + element)
          else
            compensation = compensation + ((element - running) + total)
          end
        else
          compensation = 0.0
        end
        total = running
      else
        total = total + element
      end
    end
    total.is_a?(Float) ? total + compensation : total
  end

  def sort(&block)
    to_a.sort(&block)
  end

  def sort_by
    return sized_enum(:sort_by) unless block_given?
    keyed = []
    each do |*values|
      element = packed(values)
      keyed.push([yield(packed(values)), element])
    end
    keyed.sort { |left, right| left[0] <=> right[0] }.map { |pair| pair[1] }
  end

  # What a comparison block answered, read as an ordering. Ruby asks a value
  # that is not already a number which side of zero it falls on.
  def compared_to_zero(held)
    return held if held.is_a?(Integer)
    if held.nil?
      raise ArgumentError, "comparison failed"
    end
    return 1 if held > 0
    return -1 if held < 0
    0
  end
  private :compared_to_zero

  # The smallest value the walk hands over, decided by the block where one is
  # given. The block is handed the value being looked at and the one held so
  # far.
  def pick_by_comparison(wanted, &block)
    held = nil
    seen = false
    each do |*values|
      value = packed(values)
      if !seen
        held = value
        seen = true
      elsif compared_to_zero(block.call(value, held)) == wanted
        held = value
      end
    end
    held
  end
  private :pick_by_comparison

  def min(*count, &block)
    if !block.nil? && (count.empty? || count[0].nil?)
      return pick_by_comparison(-1, &block)
    end
    ordered = to_a.sort(&block)
    return ordered.first if count.empty? || count[0].nil?
    ordered.first(count[0])
  end

  def max(*count, &block)
    if !block.nil? && (count.empty? || count[0].nil?)
      return pick_by_comparison(1, &block)
    end
    ordered = to_a.sort(&block)
    return ordered.last if count.empty? || count[0].nil?
    ordered.last(count[0]).reverse
  end

  def minmax(&block)
    to_a.minmax(&block)
  end

  def min_by(*count)
    return sized_enum(:min_by, *count) unless block_given?
    ordered = sort_by { |*values| yield(packed(values)) }
    return ordered.first if count.empty? || count[0].nil?
    ordered.first(count[0])
  end

  # On a tie the one that came first wins, so the walk keeps what it has
  # unless a later value is strictly greater.
  def max_by(*count)
    return sized_enum(:max_by, *count) unless block_given?
    unless count.empty? || count[0].nil?
      ordered = sort_by { |*values| yield(packed(values)) }
      return ordered.last(count[0]).reverse
    end
    held = nil
    best = nil
    each do |*values|
      value = packed(values)
      measured = yield(value)
      if best.nil? || (measured <=> best) > 0
        best = measured
        held = value
      end
    end
    held
  end

  # The first of a tie wins at either end, which sorting alone does not
  # promise, so each end is walked for itself.
  def minmax_by(&block)
    return sized_enum(:minmax_by) unless block_given?
    [min_by(&block), max_by(&block)]
  end

  def group_by
    return sized_enum(:group_by) unless block_given?
    grouped = {}
    each do |*values|
      element = packed(values)
      key = yield(packed(values))
      grouped[key] = [] unless grouped.has_key?(key)
      grouped[key].push(element)
    end
    grouped
  end

  def partition
    return sized_enum(:partition) unless block_given?
    held = []
    rest = []
    each do |*values|
      element = packed(values)
      if yield(packed(values))
        held.push(element)
      else
        rest.push(element)
      end
    end
    [held, rest]
  end

  # A Hash argument is counted into and answered, so tallies from several
  # walks add up. Its own default value plays no part in the counting.
  def tally(*counter)
    counts = counter.empty? ? {} : counter[0]
    unless counter.empty?
      counts = counts.to_hash if !counts.is_a?(Hash) && counts.respond_to?(:to_hash)
      raise TypeError, "no implicit conversion into Hash" unless counts.is_a?(Hash)
      raise FrozenError, "can't modify frozen Hash: #{counts.inspect}" if counts.frozen?
      counts.each do |key, value|
        raise TypeError, "wrong argument type #{value.class} (expected Integer)" unless value.is_a?(Integer)
      end
    end
    each do |*values|
      element = packed(values)
      counts[element] = counts.has_key?(element) ? counts[element] + 1 : 1
    end
    counts
  end

  def uniq(&block)
    to_a.uniq(&block)
  end

  def to_h(*arguments, &block)
    built = {}
    each(*arguments) do |*values|
      pair = block.nil? ? packed(values) : block.call(packed(values))
      pair = __pair_of__ pair
      built[pair[0]] = pair[1]
    end
    built
  end

  # One [key, value] pair, which is what every element of a walk read as a
  # Hash has to be. Anything that is not already an Array is asked for one.
  def __pair_of__(pair)
    unless pair.is_a? Array
      converted = pair.respond_to?(:to_ary) ? pair.to_ary : nil
      unless converted.is_a? Array
        raise TypeError, "wrong element type #{pair.class} (expected array)"
      end
      pair = converted
    end
    unless pair.size == 2
      raise ArgumentError, "element has wrong array length (expected 2, was #{pair.size})"
    end
    pair
  end
  private :__pair_of__

  def to_set(*args, &block)
    to_a.to_set(*args, &block)
  end

  # Each argument is walked alongside the receiver. An Array is taken as it
  # is, anything else is asked for one through `to_ary`, and failing that
  # for an Enumerator through `to_enum`.
  def zip_partner(other)
    return other if other.is_a?(Array)
    return other.to_ary if other.respond_to?(:to_ary)
    # Every object answers `to_enum`, so the walk is only taken where there
    # is an `each` for it to run.
    if other.respond_to?(:each)
      return other.to_enum(:each).to_a
    end
    raise TypeError, "wrong argument type #{other.class} (must respond to :each)"
  end
  private :zip_partner

  def zip(*others)
    walked = others.map { |other| zip_partner(other) }
    collected = []
    index = 0
    each do |*values|
      row = [packed(values)]
      walked.each { |other| row.push(other[index]) }
      collected.push(row)
      index += 1
    end
    return collected unless block_given?
    collected.each { |row| yield row }
    nil
  end

  # The batch width has to be a positive Integer, which an object that
  # answers `to_int` can stand in for.
  def slice_width(size)
    size = size.to_int if !size.is_a?(Integer) && size.respond_to?(:to_int)
    raise TypeError, "no implicit conversion into Integer" unless size.is_a?(Integer)
    raise ArgumentError, "invalid size" if size < 1
    size
  end
  private :slice_width

  def each_slice(size)
    size = slice_width(size)
    unless block_given?
      batches = respond_to?(:size) && !self.size.nil? ? (self.size + size - 1) / size : nil
      return Enumerator.over(self, :each_slice, [size], batches)
    end
    batch = []
    each do |*values|
      batch.push(packed(values))
      if batch.size == size
        yield batch
        batch = []
      end
    end
    yield batch unless batch.empty?
    self
  end

  def each_cons(size)
    size = slice_width(size)
    unless block_given?
      windows = nil
      if respond_to?(:size) && !self.size.nil?
        windows = self.size - size + 1
        windows = 0 if windows < 0
      end
      return Enumerator.over(self, :each_cons, [size], windows)
    end
    window = []
    each do |*values|
      window.push(packed(values))
      window.shift if window.size > size
      yield window.dup if window.size == size
    end
    self
  end

  def reverse_each(&block)
    return sized_enum(:reverse_each) unless block_given?
    to_a.reverse_each(&block)
    self
  end

  # A predicate takes at most a pattern, matched with `===`. The pattern wins
  # over a block, and Ruby warns that the block went unused.
  def check_predicate_arguments(pattern, block_was_given)
    if pattern.size > 1
      raise ArgumentError, "wrong number of arguments (given #{pattern.size}, expected 0..1)"
    end
    warn "warning: given block not used" if !pattern.empty? && block_was_given
  end
  private :check_predicate_arguments

  def all?(*pattern)
    check_predicate_arguments(pattern, block_given?)
    each do |*values|
      held = if !pattern.empty?
        pattern[0] === packed(values)
      elsif block_given?
        yield(*values)
      else
        packed(values)
      end
      return false unless held
    end
    true
  end

  def any?(*pattern)
    check_predicate_arguments(pattern, block_given?)
    each do |*values|
      held = if !pattern.empty?
        pattern[0] === packed(values)
      elsif block_given?
        yield(*values)
      else
        packed(values)
      end
      return true if held
    end
    false
  end

  def none?(*pattern)
    check_predicate_arguments(pattern, block_given?)
    each do |*values|
      held = if !pattern.empty?
        pattern[0] === packed(values)
      elsif block_given?
        yield(*values)
      else
        packed(values)
      end
      return false if held
    end
    true
  end

  def one?(*pattern)
    check_predicate_arguments(pattern, block_given?)
    counted = 0
    each do |*values|
      held = if !pattern.empty?
        pattern[0] === packed(values)
      elsif block_given?
        yield(*values)
      else
        packed(values)
      end
      if held
        counted += 1
        return false if counted > 1
      end
    end
    counted == 1
  end

  # Walk the elements over and over, a given number of rounds or without
  # end. The first round hands out the elements as they are read, so an
  # endpoint that stops early is asked for no more than it needs.
  def cycle(*count)
    if count.size > 1
      raise ArgumentError, "wrong number of arguments (given #{count.size}, expected 0..1)"
    end
    rounds = __cycle_rounds__ count.first
    unless block_given?
      # The enumerator is handed the count already read as an Integer, so a
      # count that reports how often it was asked is asked only once.
      forwarded = count.empty? ? [] : [rounds]
      return to_enum(:cycle, *forwarded) { __cycle_size__ rounds }
    end
    return nil if !rounds.nil? && rounds <= 0
    values = []
    each do |*yielded|
      held = yielded.size == 1 ? yielded[0] : yielded
      values.push held
      yield held
    end
    return nil if values.empty?
    if rounds.nil?
      while true
        values.each { |element| yield element }
      end
    else
      (rounds - 1).times { values.each { |element| yield element } }
    end
    nil
  end

  # The number of rounds a count names. Anything that reads as an Integer
  # names one, and anything else is refused.
  def __cycle_rounds__(count)
    return nil if count.nil?
    return count if count.is_a?(Integer)
    unless count.respond_to?(:to_int)
      raise TypeError, "no implicit conversion of #{count.class} into Integer"
    end
    held = count.to_int
    unless held.is_a?(Integer)
      raise TypeError,
            "can't convert #{count.class} to Integer (#{count.class}#to_int gives #{held.class})"
    end
    held
  end
  private :__cycle_rounds__

  # The count a cycling walk hands out, where the source can say how many it
  # holds. A walk with no end counts to infinity.
  def __cycle_size__(rounds)
    held = begin
      size
    rescue StandardError
      nil
    end
    return nil unless held.is_a?(Integer)
    return Float::INFINITY if rounds.nil?
    return 0 if rounds <= 0
    held * rounds
  end
  private :__cycle_size__

  def compact
    reject { |element| element.nil? }
  end

  # A lazy walk applies its operations one element at a time, so a source
  # with no end can still answer `first` or `take`.
  def lazy
    Enumerator::Lazy.build(self, lazy_source_size)
  end

  # The size a lazy walk starts from. A source that cannot report one leaves
  # the walk's size nil.
  def lazy_source_size
    size
  rescue NoMethodError, NameError
    nil
  end
  private :lazy_source_size

  # The runs a block groups together, each answered with the key it grouped
  # under. A run ends where the key changes.
  def chunk(&block)
    return to_enum(:chunk) if block.nil?
    grouped = []
    run = []
    key = nil
    each do |*values|
      element = packed(values)
      found = block.call(element)
      # A name beginning with an underscore is the library's own: `:_alone`
      # keeps the element on its own, `:_separator` and nil drop it, and
      # every other such name is a mistake.
      if found.is_a?(Symbol) && found.to_s.start_with?("_") &&
         found != :_alone && found != :_separator
        raise RuntimeError, "symbols beginning with an underscore are reserved"
      end
      if found.nil? || found == :_separator
        grouped.push([key, run]) unless run.empty?
        run = []
        key = nil
        next
      end
      if found == :_alone
        grouped.push([key, run]) unless run.empty?
        grouped.push([:_alone, [element]])
        run = []
        key = nil
        next
      end
      unless run.empty? || found == key
        grouped.push([key, run])
        run = []
      end
      key = found
      run.push(element)
    end
    grouped.push([key, run]) unless run.empty?
    Enumerator.over(grouped, :each, [], nil)
  end

  # The runs where each neighboring pair answers true, so a false answer
  # starts a new run.
  def chunk_while(&block)
    raise ArgumentError, "tried to create Proc object without a block" if block.nil?
    grouped_runs(block, false)
  end

  # The runs where each neighboring pair answers false, which is chunk_while
  # read the other way around.
  def slice_when(&block)
    raise ArgumentError, "tried to create Proc object without a block" if block.nil?
    grouped_runs(block, true)
  end

  # What decides where a run is cut. Exactly one of a pattern and a block
  # says so, and giving both or neither is an error.
  def run_matcher(pattern, block)
    if block.nil?
      raise ArgumentError, "both pattern and block are given" if false
      raise ArgumentError, "wrong number of arguments (given 0, expected 1)" if pattern.nil?
      return lambda { |element| pattern === element }
    end
    raise ArgumentError, "both pattern and block are given" unless pattern.nil?
    block
  end
  private :run_matcher

  def grouped_runs(block, cut_on)
    grouped = []
    run = []
    previous = nil
    each do |*values|
      element = packed(values)
      unless run.empty? || block.call(previous, element) != cut_on
        grouped.push(run)
        run = []
      end
      previous = element
      run.push(element)
    end
    grouped.push(run) unless run.empty?
    Enumerator.over(grouped, :each, [], grouped.size)
  end
  private :grouped_runs

  # The runs that start at every element the pattern or block picks out.
  def slice_before(pattern = nil, &block)
    matcher = run_matcher(pattern, block)
    grouped = []
    run = []
    each do |*values|
      element = packed(values)
      # Every element is put to the test, the first one included, even
      # though a run that has not started yet cannot be closed.
      starts = matcher.call element
      if starts && !run.empty?
        grouped.push(run)
        run = []
      end
      run.push(element)
    end
    grouped.push(run) unless run.empty?
    Enumerator.over(grouped, :each, [], nil)
  end

  # The runs that end at every element the pattern or block picks out.
  def slice_after(pattern = nil, &block)
    matcher = run_matcher(pattern, block)
    grouped = []
    run = []
    each do |*values|
      element = packed(values)
      run.push(element)
      if matcher.call(element)
        grouped.push(run)
        run = []
      end
    end
    grouped.push(run) unless run.empty?
    Enumerator.over(grouped, :each, [], grouped.size)
  end

  # The walks run in order, which is what a chain is.
  def chain(*others)
    Enumerator::Chain.new(self, *others)
  end
end

# What a regexp match found: the whole match, the captures, and where each
# one sat in the subject. The native matcher builds one of these and hands it
# back from `Regexp#match`, `String#match`, and `=~`."##;
