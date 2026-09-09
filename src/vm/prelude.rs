// Core library pieces defined in Ruby rather than in Rust.
//
// A method written here is a real user-defined method, so it can be aliased,
// redefined, mocked, and introspected the way MRI's own Ruby-level core
// methods can. Kernel#warn relies on that: the specs alias `Warning.warn`
// away and put their own back.

use crate::vm::core::VirtualMachine;

/// Ruby source evaluated into every fresh VM.
const PRELUDE_SOURCE: &str = r##"
module Warning
  def warn(message, category: nil)
    return nil unless category.nil? || Warning[category]
    $stderr.write message
    nil
  end

  extend self
end

# Math wraps the one native primitive in a method per function. Each is a
# module function, so `Math.sqrt` and a private `sqrt` inside a class that
# includes Math both reach it.
module Math
  def sqrt(x)
    __math_function__(:sqrt, x)
  end

  def cbrt(x)
    __math_function__(:cbrt, x)
  end

  def sin(x)
    __math_function__(:sin, x)
  end

  def cos(x)
    __math_function__(:cos, x)
  end

  def tan(x)
    __math_function__(:tan, x)
  end

  def asin(x)
    __math_function__(:asin, x)
  end

  def acos(x)
    __math_function__(:acos, x)
  end

  def atan(x)
    __math_function__(:atan, x)
  end

  def sinh(x)
    __math_function__(:sinh, x)
  end

  def cosh(x)
    __math_function__(:cosh, x)
  end

  def tanh(x)
    __math_function__(:tanh, x)
  end

  def asinh(x)
    __math_function__(:asinh, x)
  end

  def acosh(x)
    __math_function__(:acosh, x)
  end

  def atanh(x)
    __math_function__(:atanh, x)
  end

  def exp(x)
    __math_function__(:exp, x)
  end

  def log2(x)
    __math_function__(:log2, x)
  end

  def log10(x)
    __math_function__(:log10, x)
  end

  def log1p(x)
    __math_function__(:log1p, x)
  end

  def expm1(x)
    __math_function__(:expm1, x)
  end

  def erf(x)
    __math_function__(:erf, x)
  end

  def erfc(x)
    __math_function__(:erfc, x)
  end

  def gamma(x)
    __math_function__(:gamma, x)
  end

  def atan2(y, x)
    __math_function__(:atan2, y, x)
  end

  def hypot(x, y)
    __math_function__(:hypot, x, y)
  end

  def ldexp(fraction, exponent)
    __math_function__(:ldexp, fraction, exponent)
  end

  def frexp(x)
    __math_function__(:frexp, x)
  end

  def lgamma(x)
    __math_function__(:lgamma, x)
  end

  # A base of nil is an argument Ruby refuses, which is not the same as
  # leaving the base out.
  def log(x, *base)
    return __math_function__(:log, x) if base.empty?
    __math_function__(:log, x, base.first)
  end

  module_function :sqrt, :cbrt, :sin, :cos, :tan, :asin, :acos, :atan, :sinh, :cosh, :tanh, :asinh, :acosh, :atanh, :exp, :log2, :log10, :log1p, :expm1, :erf, :erfc, :gamma, :atan2, :hypot, :ldexp, :frexp, :lgamma, :log
end

# Numeric carries the protocol every number answers, written in terms of the
# methods a subclass supplies. Integer and Float reach their own native
# implementations first, so what is here serves the subclasses a program writes.
class Numeric
  def abs
    self < 0 ? -self : self
  end

  def magnitude
    abs
  end

  def ceil(digits = 0)
    to_f.ceil(digits)
  end

  def floor(digits = 0)
    to_f.floor(digits)
  end

  def round(digits = 0)
    to_f.round(digits)
  end

  def truncate(digits = 0)
    to_f.truncate(digits)
  end

  def to_int
    to_i
  end

  def zero?
    self == 0
  end

  def nonzero?
    zero? ? nil : self
  end

  def positive?
    self > 0
  end

  def negative?
    self < 0
  end

  def integer?
    false
  end

  def finite?
    true
  end

  def infinite?
    nil
  end

  def real?
    true
  end

  def real
    self
  end

  def imaginary
    0
  end

  def imag
    imaginary
  end

  def conjugate
    self
  end

  def conj
    conjugate
  end

  def div(other)
    raise ZeroDivisionError, "divided by 0" if other == 0
    (self / other).floor
  end

  def modulo(other)
    self - other * div(other)
  end

  def %(other)
    modulo(other)
  end

  def divmod(other)
    [div(other), modulo(other)]
  end

  # `remainder` truncates the division where `modulo` floors it, so the two
  # differ by one divisor whenever the signs disagree.
  def remainder(other)
    left = self % other
    return left if left == 0
    return left - other if (self < 0 && other > 0) || (self > 0 && other < 0)
    left
  end

  def fdiv(other)
    to_f / other.to_f
  end

  def eql?(other)
    return false unless other.instance_of? self.class
    (self == other) ? true : false
  end

  def coerce(other)
    return [other, self] if other.instance_of? self.class
    [Float(other), Float(self)]
  end

  def numerator
    to_r.numerator
  end

  def denominator
    to_r.denominator
  end

  # Ruby refuses a singleton method on a number, since two numbers of the same
  # value are the same object.
  def singleton_method_added(name)
    raise TypeError, "can't define singleton"
  end

  def dup
    self
  end

  # A number is frozen, so a clone cannot ask for an unfrozen one.
  def clone(freeze: true)
    raise ArgumentError, "can't unfreeze #{self.class}" if freeze == false
    self
  end

  def +@
    self
  end
end

# Enumerable, written over `each` the way Ruby writes it. A class that
# defines `each` and includes this answers the whole family. Array, Hash,
# Range, and Struct reach their own native implementations first.
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
    Enumerator.new(self, name, args, respond_to?(:size) ? size : nil)
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

  def map
    return sized_enum(:map) unless block_given?
    collected = []
    each { |*values| collected.push(yield(packed(values))) }
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

  def min(*count, &block)
    ordered = to_a.sort(&block)
    return ordered.first if count.empty? || count[0].nil?
    ordered.first(count[0])
  end

  def max(*count, &block)
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

  def to_h(&block)
    built = {}
    each do |*values|
      pair = block_given? ? yield(packed(values)) : (packed(values))
      built[pair[0]] = pair[1]
    end
    built
  end

  def to_set(*args, &block)
    to_a.to_set(*args, &block)
  end

  # Each argument is walked alongside the receiver. An Array is taken as it
  # is, anything else is asked for one through `to_ary`, and failing that
  # for an Enumerator through `to_enum`.
  def zip_partner(other)
    return other if other.is_a?(Array)
    return other.to_ary if other.respond_to?(:to_ary)
    if other.respond_to?(:to_enum) || other.respond_to?(:each)
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
      return Enumerator.new(self, :each_slice, [size], batches)
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
      return Enumerator.new(self, :each_cons, [size], windows)
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

  def cycle(*count)
    return to_enum(:cycle, *count) unless block_given?
    values = to_a
    return nil if values.empty?
    if count.empty?
      while true
        values.each { |element| yield element }
      end
    end
    rounds = count[0]
    return nil if rounds.nil? == false && rounds <= 0
    rounds.times { values.each { |element| yield element } }
    nil
  end

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
      unless run.empty? || found == key
        grouped.push([key, run])
        run = []
      end
      key = found
      run.push(element)
    end
    grouped.push([key, run]) unless run.empty?
    Enumerator.new(grouped, :each, [], grouped.size)
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
    Enumerator.new(grouped, :each, [], grouped.size)
  end
  private :grouped_runs

  # The runs that start at every element the pattern or block picks out.
  def slice_before(pattern = nil, &block)
    matcher = run_matcher(pattern, block)
    grouped = []
    run = []
    each do |*values|
      element = packed(values)
      unless run.empty? || !matcher.call(element)
        grouped.push(run)
        run = []
      end
      run.push(element)
    end
    grouped.push(run) unless run.empty?
    Enumerator.new(grouped, :each, [], grouped.size)
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
    Enumerator.new(grouped, :each, [], grouped.size)
  end

  # The walks run in order, which is what a chain is.
  def chain(*others)
    Enumerator::Chain.new(self, *others)
  end
end

# What a regexp match found: the whole match, the captures, and where each
# one sat in the subject. The native matcher builds one of these and hands it
# back from `Regexp#match`, `String#match`, and `=~`.
class MatchData
  def initialize(string, regexp, begins, ends, names)
    @string = string
    @regexp = regexp
    @begins = begins
    @ends = ends
    @names = names
  end

  def string
    @string
  end

  def regexp
    @regexp
  end

  def size
    @begins.size
  end

  def length
    @begins.size
  end

  # The index a subscript names: a number counts from the front, and a Symbol
  # or String names one of the pattern's groups.
  def group_index(key)
    if key.is_a?(Symbol) || key.is_a?(String)
      name = key.to_s
      index = @names[name]
      raise IndexError, "undefined group name reference: #{name}" if index.nil?
      return index
    end
    unless key.is_a?(Integer)
      unless key.respond_to?(:to_int)
        raise TypeError, "no implicit conversion of #{key.class} into Integer"
      end
      key = key.to_int
      unless key.is_a?(Integer)
        raise TypeError, "can't convert #{key.class} to Integer"
      end
    end
    index = key < 0 ? key + @begins.size : key
    if index < 0 || index >= @begins.size
      raise IndexError, "index #{key} out of matches"
    end
    index
  end
  private :group_index

  def group_text(index)
    index = index + @begins.size if index < 0
    return nil if index < 0 || index >= @begins.size
    return nil if @begins[index].nil?
    @string[@begins[index], @ends[index] - @begins[index]]
  end
  private :group_text

  def [](*keys)
    if keys.size == 2 && keys[0].is_a?(Integer) && keys[1].is_a?(Integer)
      return to_a[keys[0], keys[1]]
    end
    key = keys[0]
    return to_a[key] if key.is_a?(Range)
    if key.is_a?(Integer)
      index = key < 0 ? key + @begins.size : key
      return nil if index < 0 || index >= @begins.size
      return group_text(index)
    end
    group_text(group_index(key))
  end

  def to_a
    collected = []
    index = 0
    while index < @begins.size
      collected.push(group_text(index))
      index += 1
    end
    collected
  end

  def captures
    to_a[1, @begins.size - 1]
  end

  def named_captures(symbolize_names: false)
    collected = {}
    @names.each do |name, index|
      key = symbolize_names ? name.to_sym : name
      collected[key] = group_text(index)
    end
    collected
  end

  def names
    @names.keys
  end

  # A Range subscript picks a run of groups, the way it does on an Array.
  def values_at(*keys)
    collected = []
    keys.each do |key|
      if key.is_a?(Range)
        first = key.begin.nil? ? 0 : key.begin
        first += @begins.size if first < 0
        raise RangeError, "#{key} out of range" if first < 0 || first > @begins.size
        last = key.end.nil? ? @begins.size - 1 : key.end
        last += @begins.size if key.end && key.end < 0
        last -= 1 if key.exclude_end?
        index = first
        while index <= last
          collected.push(self[index])
          index += 1
        end
      elsif key.is_a?(Integer)
        collected.push(self[key])
      else
        collected.push(group_text(group_index(key)))
      end
    end
    collected
  end

  # `begin`, `end`, and `offset` count only forward, so a negative subscript
  # is out of bounds rather than a count from the end.
  def positive_group_index(key)
    if key.is_a?(Integer) && key < 0
      raise IndexError, "index #{key} out of matches"
    end
    group_index(key)
  end
  private :positive_group_index

  def begin(key)
    @begins[positive_group_index(key)]
  end

  def end(key)
    @ends[positive_group_index(key)]
  end

  def offset(key)
    index = positive_group_index(key)
    [@begins[index], @ends[index]]
  end

  # `match` and `match_length` answer one group's text and its length.
  def match(key)
    group_text(positive_group_index(key))
  end

  def match_length(key)
    found = match(key)
    found.nil? ? nil : found.size
  end

  def pre_match
    @string[0, @begins[0]]
  end

  def post_match
    @string[@ends[0], @string.size - @ends[0]]
  end

  def to_s
    group_text(0)
  end

  def deconstruct
    captures
  end

  def deconstruct_keys(keys)
    collected = {}
    wanted = keys.nil? ? @names.keys : keys.map { |key| key.to_s }
    wanted.each do |name|
      index = @names[name]
      collected[name.to_sym] = index.nil? ? nil : group_text(index)
    end
    collected
  end

  def ==(other)
    return false unless other.is_a?(MatchData)
    string == other.string && regexp == other.regexp && to_a == other.to_a
  end

  def eql?(other)
    self == other
  end

  def hash
    [string, to_a].hash
  end

  def inspect
    parts = ["#<MatchData"]
    parts.push(group_text(0).inspect)
    index = 1
    reversed = {}
    @names.each { |name, at| reversed[at] = name }
    while index < @begins.size
      label = reversed.has_key?(index) ? reversed[index] : index.to_s
      parts.push("#{label}:#{group_text(index).inspect}")
      index += 1
    end
    parts.join(" ") + ">"
  end
end

class Complex
  # The imaginary unit, which every other Complex is measured against.
  I = Complex(0, 1)

  # A Complex names no point on the number line, so it answers neither of the
  # questions a real number does.
  undef_method :positive?
  undef_method :negative?
end

class IO
  module WaitReadable
  end

  module WaitWritable
  end

  class EAGAINWaitReadable < Errno::EAGAIN
    include WaitReadable
  end

  class EAGAINWaitWritable < Errno::EAGAIN
    include WaitWritable
  end

  EWOULDBLOCKWaitReadable = EAGAINWaitReadable
  EWOULDBLOCKWaitWritable = EAGAINWaitWritable
end

class StopIteration
  attr_accessor :result
end

class Thread
  class Backtrace
    class Location
      attr_reader :path, :lineno, :label, :absolute_path

      def base_label
        return @label if @label.nil?
        return @label unless @label.start_with?("block ")
        @label.split(" in ", 2).last
      end

      def to_s
        return "#{@path}:#{@lineno}" if @label.nil? || @label.empty?
        "#{@path}:#{@lineno}:in '#{@label}'"
      end

      def inspect
        to_s.inspect
      end
    end
  end
end

# An in-memory IO. `StringIO.new` starts from the string it is given, and
# everything written is appended to it.
# A string read and written the way a file is: it holds a position, a line
# count, and the two sides of a stream that may be closed apart.
class StringIO
  include Enumerable

  VERSION = "3.1.2"

  def initialize(string = "", mode = nil)
    @string = string
    @position = 0
    @lineno = 0
    @closed_read = false
    @closed_write = false
    @ungotten = ""
    read_mode(mode)
    @string = "" if @truncates
    self
  end

  # What a mode string says about which sides of the stream are open. A
  # missing mode leaves both open.
  def read_mode(mode)
    @readable = true
    @writable = true
    @appends = false
    @truncates = false
    return if mode.nil?
    spelling = mode.to_s.gsub("b", "").gsub("t", "")
    case spelling
    when "r"
      @writable = false
    when "r+"
      nil
    when "w"
      @readable = false
      @truncates = true
    when "w+"
      @truncates = true
    when "a"
      @readable = false
      @appends = true
    when "a+"
      @appends = true
    else
      raise ArgumentError, "invalid access mode #{mode}"
    end
  end
  private :read_mode

  def self.new(string = "", mode = nil)
    if block_given?
      warn "warning: StringIO::new() does not take block; use StringIO::open() instead"
    end
    made = allocate
    made.send(:initialize, string, mode)
    made
  end

  def self.open(string = "", mode = nil)
    held = new(string, mode)
    return held unless block_given?
    begin
      yield held
    ensure
      held.close
    end
  end

  # ── What the stream holds ────────────────────────────────────────────────

  def string
    @string
  end

  def string=(text)
    @string = text.is_a?(String) ? text : text.to_str
    @position = 0
    @lineno = 0
    text
  end

  def size
    @string.length
  end

  def length
    @string.length
  end

  def pos
    @position
  end

  def tell
    @position
  end

  def pos=(offset)
    raise Errno::EINVAL, "Invalid argument" if offset < 0
    @position = offset
  end

  def lineno
    @lineno
  end

  def lineno=(count)
    @lineno = count
  end

  def rewind
    @position = 0
    @lineno = 0
    0
  end

  def seek(amount, whence = 0)
    unless amount.is_a?(Integer)
      raise TypeError, "no implicit conversion of #{amount.class} into Integer"
    end
    base = if whence == 0
      0
    elsif whence == 1
      @position
    elsif whence == 2
      @string.length
    else
      raise Errno::EINVAL, "Invalid argument"
    end
    landing = base + amount
    raise Errno::EINVAL, "Invalid argument" if landing < 0
    @position = landing
    0
  end

  def eof?
    @position >= @string.length
  end

  def eof
    eof?
  end

  def truncate(length)
    writing_allowed
    unless length.is_a?(Integer) || length.respond_to?(:to_int)
      raise TypeError, "no implicit conversion of #{length.class} into Integer"
    end
    wanted = length.is_a?(Integer) ? length : length.to_int
    raise Errno::EINVAL, "Invalid argument" if wanted < 0
    if wanted <= @string.length
      @string = @string[0, wanted]
    else
      @string = @string + "\0" * (wanted - @string.length)
    end
    0
  end

  def reopen(other = nil, mode = nil)
    if other.is_a?(StringIO)
      @string = other.string
      @position = 0
      @lineno = 0
      read_mode(nil)
      return self
    end
    @string = other.nil? ? "" : other
    @position = 0
    @lineno = 0
    read_mode(mode)
    @string = "" if @truncates
    self
  end

  # ── Which sides are open ─────────────────────────────────────────────────

  def close
    @closed_read = true
    @closed_write = true
    nil
  end

  def close_read
    raise IOError, "closing non-duplex IO for reading" unless @readable
    @closed_read = true
    nil
  end

  def close_write
    raise IOError, "closing non-duplex IO for writing" unless @writable
    @closed_write = true
    nil
  end

  def closed?
    (!@readable || @closed_read) && (!@writable || @closed_write)
  end

  def closed_read?
    !@readable || @closed_read
  end

  def closed_write?
    !@writable || @closed_write
  end

  # Whether this stream may still be read, which every reading method asks
  # before it does anything.
  def reading_allowed
    raise IOError, "not opened for reading" unless @readable
    raise IOError, "not opened for reading" if @closed_read
  end
  private :reading_allowed

  # Whether this stream may still be written.
  def writing_allowed
    raise IOError, "not opened for writing" unless @writable
    raise IOError, "not opened for writing" if @closed_write
  end
  private :writing_allowed

  # ── Reading ──────────────────────────────────────────────────────────────

  def read(length = nil, buffer = nil)
    reading_allowed
    remaining = @string[@position..-1] || ""
    if length.nil?
      @position = @string.length
      return remaining
    end
    wanted = length.is_a?(Integer) ? length : length.to_int
    raise ArgumentError, "negative length #{wanted} given" if wanted < 0
    return nil if remaining.empty? && wanted > 0
    taken = remaining[0, wanted]
    @position = @position + taken.length
    taken
  end

  def sysread(length = nil, buffer = nil)
    reading_allowed
    raise EOFError, "end of file reached" if eof? && !length.nil? && length > 0
    self.read(length, buffer)
  end

  def readpartial(length = nil, buffer = nil)
    self.sysread(length, buffer)
  end

  def read_nonblock(length = nil, buffer = nil, exception: true)
    reading_allowed
    if eof? && !length.nil? && length > 0
      return nil unless exception
      raise EOFError, "end of file reached"
    end
    self.read(length, buffer)
  end

  def getc
    reading_allowed
    return nil if @position >= @string.length
    letter = @string[@position]
    @position = @position + 1
    letter
  end

  def readchar
    letter = self.getc
    raise EOFError, "end of file reached" if letter.nil?
    letter
  end

  def getbyte
    reading_allowed
    bytes = @string.bytes
    return nil if @position >= bytes.length
    byte = bytes[@position]
    @position = @position + 1
    byte
  end

  def readbyte
    byte = self.getbyte
    raise EOFError, "end of file reached" if byte.nil?
    byte
  end

  def ungetc(letter)
    return nil if letter.nil?
    text = letter.is_a?(Integer) ? letter.chr : letter.to_s
    landing = @position - text.length
    landing = 0 if landing < 0
    @string = @string[0, landing] + text + (@string[landing + text.length..-1] || "")
    @position = landing
    nil
  end

  def ungetbyte(byte)
    return nil if byte.nil?
    self.ungetc(byte.is_a?(Integer) ? (byte % 256).chr : byte.to_s)
  end

  def gets(separator = "\n", limit = nil)
    reading_allowed
    remaining = @string[@position..-1] || ""
    return nil if remaining.empty?
    line = if separator.nil?
      remaining
    elsif separator == ""
      # A blank separator reads a paragraph: the newlines before it are
      # stepped over, and the run ends at the blank line that follows.
      skipped = 0
      skipped += 1 while skipped < remaining.length && remaining[skipped] == "\n"
      remaining = remaining[skipped..-1] || ""
      @position = @position + skipped
      return nil if remaining.empty?
      cut = remaining.index("\n\n")
      cut.nil? ? remaining : remaining[0, cut + 1]
    else
      cut = remaining.index(separator)
      cut.nil? ? remaining : remaining[0, cut + separator.length]
    end
    line = line[0, limit] unless limit.nil?
    return nil if line.empty?
    @position = @position + line.length
    @lineno = @lineno + 1
    line
  end

  def readline(separator = "\n", limit = nil)
    line = self.gets(separator, limit)
    raise EOFError, "end of file reached" if line.nil?
    line
  end

  def each_line(separator = "\n")
    reading_allowed
    return sized_enum(:each_line, separator) unless block_given?
    while (line = self.gets(separator))
      yield line
    end
    self
  end

  def each(separator = "\n", &block)
    each_line(separator, &block)
  end

  def readlines(separator = "\n", limit = nil)
    reading_allowed
    collected = []
    while (line = self.gets(separator, limit))
      collected.push(line)
    end
    collected
  end

  def each_byte
    reading_allowed
    return sized_enum(:each_byte) unless block_given?
    while (byte = self.getbyte)
      yield byte
    end
    self
  end

  def each_char
    reading_allowed
    return sized_enum(:each_char) unless block_given?
    while (letter = self.getc)
      yield letter
    end
    self
  end

  def each_codepoint
    reading_allowed
    return sized_enum(:each_codepoint) unless block_given?
    while (letter = self.getc)
      yield letter.ord
    end
    self
  end

  # ── Writing ──────────────────────────────────────────────────────────────

  def write(*values)
    writing_allowed
    written = 0
    values.each do |value|
      text = value.to_s
      @position = @string.length if @appends
      landing = @position
      if landing > @string.length
        @string = @string + "\0" * (landing - @string.length)
      end
      @string = @string[0, landing] + text + (@string[landing + text.length..-1] || "")
      @position = landing + text.length
      written = written + text.length
    end
    written
  end

  def syswrite(value)
    self.write(value)
  end

  def write_nonblock(value, exception: true)
    self.write(value)
  end

  def <<(value)
    self.write value
    self
  end

  def print(*values)
    writing_allowed
    values = [$_] if values.empty?
    values.each { |value| self.write(value.nil? ? "" : value.to_s) }
    self.write($\) unless $\.nil?
    nil
  end

  def printf(format, *values)
    self.write format % values
    nil
  end

  def putc(value)
    writing_allowed
    text = if value.is_a?(Integer)
      (value % 256).chr
    elsif value.is_a?(String)
      value[0, 1]
    elsif value.respond_to?(:to_int)
      (value.to_int % 256).chr
    else
      raise TypeError, "no implicit conversion of #{value.class} into Integer"
    end
    self.write text
    value
  end

  def puts(*values)
    writing_allowed
    write_lines(values, [])
    nil
  end

  # One line per value, where an array is written out element by element. An
  # array that reaches itself is written as `[...]` rather than followed.
  def write_lines(values, walking)
    if values.empty?
      self.write "\n"
      return
    end
    values.each do |value|
      if value.is_a?(Array)
        if walking.any? { |held| held.equal?(value) }
          self.write "[...]\n"
          next
        end
        walking.push(value)
        value.empty? ? self.write("\n") : write_lines(value, walking)
        walking.pop
        next
      end
      text = value.nil? ? "" : value.to_s
      self.write text
      self.write "\n" unless text.end_with? "\n"
    end
  end
  private :write_lines

  # ── What a stream reports about itself ───────────────────────────────────

  def fileno
    nil
  end

  def pid
    nil
  end

  def flush
    self
  end

  def fsync
    0
  end

  def sync
    true
  end

  def sync=(setting)
    setting
  end

  def isatty
    false
  end

  def tty?
    false
  end

  def binmode
    self
  end

  def external_encoding
    Encoding::UTF_8
  end

  def internal_encoding
    nil
  end

  def set_encoding(external, internal = nil)
    self
  end

  def set_encoding_by_bom
    nil
  end

  def fcntl(*args)
    raise NotImplementedError, "fcntl() function is unimplemented on this machine"
  end

  # A stream shows itself by class and address alone: what it holds is not
  # part of how it is written out.
  def inspect
    "#<StringIO:0x#{format("%016x", object_id * 2)}>"
  end

  def to_s
    inspect
  end

end

class Enumerator
  include Enumerable

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

  def initialize(receiver = nil, method_name = nil, arguments = [], size = nil, &generator)
    @receiver = receiver
    @method_name = method_name
    @arguments = arguments
    @size = size
    @position = 0
    @generator = generator
  end

  # The count the walk will hand out, where that is known ahead of it. An
  # endpoint the walk cannot count to has no size to report, and asking is
  # refused rather than answered with a guess.
  def size
    raise ArgumentError, @size_error unless @size_error.nil?
    @size
  end

  def __refuse_size__(message)
    @size_error = message
    self
  end

  def to_a
    if @values.nil?
      collected = []
      if @generator.nil?
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
  def next_values
    values = raw_to_a
    if @position >= values.size
      ended = StopIteration.new("iteration reached an end")
      ended.result = @result
      raise ended
    end
    value = values[@position]
    @position = @position + 1
    value
  end

  def peek_values
    values = raw_to_a
    if @position >= values.size
      ended = StopIteration.new("iteration reached an end")
      ended.result = @result
      raise ended
    end
    values[@position]
  end

  def raw_to_a
    if @raw_values.nil?
      collected = []
      if @generator.nil?
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

  def peek
    values = to_a
    if @position >= values.size
      ended = StopIteration.new("iteration reached an end")
      ended.result = @result
      raise ended
    end
    values[@position]
  end

  def next
    value = peek
    @position = @position + 1
    value
  end

  # Ruby hands the rewind on to the object the walk was cut from when that
  # object can be rewound, so a source with a place of its own goes back to
  # the start too.
  def rewind
    @position = 0
    if !@receiver.nil? && !@receiver.equal?(self) && @receiver.respond_to?(:rewind)
      @receiver.rewind
    end
    self
  end

  # The enumerator `loop` answers when called without a block: it yields
  # forever and reports an endless size.
  def self.endless
    enumerator = new(nil, nil, [], Float::INFINITY)
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
    return self if block.nil?
    if @endless
      while true
        block.call
      end
    end
    return @receiver.send(@method_name, *@arguments, *args, &block) if @generator.nil?
    to_a.each { |value| block.call(value) }
    @receiver
  end

  def first(count = nil)
    return to_a[0] if count.nil?
    to_a[0, count]
  end

  def map(&block)
    return self if block.nil?
    to_a.map { |value| block.call(value) }
  end

  def collect(&block)
    map(&block)
  end

  # Walks with a second value handed to the block each time, answering that
  # value once the walk is done.
  def with_object(memo)
    return Enumerator.new(self, :with_object, [memo], @size) unless block_given?
    each { |*values| yield packed(values), memo }
    memo
  end

  def inspect
    "#<Enumerator: #{@receiver.inspect}:#{@method_name}>"
  end

end
# A walk that applies its operations one element at a time, so a source with
# no end can still answer `first` or `take`. Each step holds the walk it
# reads from, so the chain runs outside in.
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
    Enumerator.new(self, :each, [], @lazy_size)
  end

  def to_enum(method_name = :each, *args)
    Enumerator.new(self, method_name, args, nil)
  end

  def enum_for(method_name = :each, *args)
    to_enum(method_name, *args)
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
        subject = packed(values)
        yield(@callable.nil? ? subject : @callable.call(*values)) if @count === subject
      when :grep_v
        subject = packed(values)
        yield(@callable.nil? ? subject : @callable.call(*values)) unless @count === subject
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
        yield [subject] + @count.map { |other| other[seen] }
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

  def zip(*others)
    walked = others.map do |other|
      raise TypeError, "wrong argument type #{other.class} (must respond to :each)" unless other.respond_to?(:to_a)
      other.to_a
    end
    with_step(:zip, nil, walked)
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

# A source with nothing in it, which `lazy.take(0)` walks.
class Enumerator::EmptyWalk
  include Enumerable

  def each
    self
  end
end

# The walks a chain runs through in order, which `+` builds.
class Enumerator::Chain < Enumerator
  def initialize(*walks)
    @walks = walks
  end

  def each
    return to_enum(:each) unless block_given?
    @walks.each do |walk|
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

  def rewind
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

  def each
    return Enumerator.new(self, :each, [], size) unless block_given?
    combinations = [[]]
    @enumerables.each do |enumerable|
      entries = []
      enumerable.each_entry { |entry| entries.push(entry) }
      grown = []
      combinations.each do |prefix|
        entries.each { |entry| grown.push(prefix + [entry]) }
      end
      combinations = grown
    end
    combinations.each { |combination| yield combination }
    self
  end

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
    super(self, :each, [])
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
    value = @from
    while within?(value)
      block.call(value)
      value = value + @by
    end
    self
  end

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
    return Float::INFINITY if @to == Float::INFINITY || @to == -Float::INFINITY
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

class Numeric
  # `1.step(10, 3)` walks 1, 4, 7, 10. Without a block it answers the
  # sequence itself, which is what carries the walk about.
  def step(limit = nil, by = nil, to: nil, by_named: nil, &block)
    named_by = by_named
    step_given = !by.nil? || !named_by.nil?
    walked_by = by.nil? ? (named_by.nil? ? 1 : named_by) : by
    raise ArgumentError, "step can\'t be 0" if walked_by == 0
    walked_to = limit.nil? ? to : limit
    sequence = Enumerator::ArithmeticSequence.send(
      :new, self, walked_to, walked_by, false, nil, "step", step_given
    )
    return sequence if block.nil?
    sequence.each { |value| block.call(value) }
    self
  end
end

class Range
  # `(1..10).step(3)` walks 1, 4, 7, 10, and `%` is written for the same
  # thing. Without a block either one answers the sequence itself.
  def step(by = nil, &block)
    stepped(by, "step", &block)
  end

  def %(by = nil, &block)
    stepped(by, "%", &block)
  end

  def stepped(by, written_as, &block)
    raise ArgumentError, "step can\'t be 0" if by == 0
    sequence = Enumerator::ArithmeticSequence.send(
      :new, self.begin, self.end, by.nil? ? 1 : by, exclude_end?, self, written_as, !by.nil?
    )
    return sequence if block.nil?
    sequence.each { |value| block.call(value) }
    self
  end
  private :stepped
end

class Array
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
    Enumerator.new(self, name, args, reach)
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
    return Enumerator.new(self, :bsearch, [], nil) unless block_given?
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
    return Enumerator.new(self, :bsearch_index, [], nil) unless block_given?
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
  # The elements grouped into sets under whatever the block answers for each
  # of them.
  def classify
    return Enumerator.new(self, :classify, [], size) unless block_given?
    grouped = Hash.new { |held, key| held[key] = Set.new }
    each { |element| grouped[yield element].add(element) }
    grouped
  end

  # The set of subsets a block cuts self into. A block of one parameter groups
  # by what it answers, and a block of two groups the elements it relates.
  def divide(&block)
    return Enumerator.new(self, :divide, [], size) if block.nil?
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
# a local time follows the zone rules the operating system holds.
class Time
  include Comparable

  MICROSECONDS_IN_SECOND = 1000000
  NANOSECONDS_IN_SECOND = 1000000000
  MONTH_NAMES = %w[jan feb mar apr may jun jul aug sep oct nov dec]

  def self.now
    counted = Time.__now__
    from_exact(counted[0].to_r + Rational(counted[1], NANOSECONDS_IN_SECOND), false, nil)
  end

  def self.new(*args)
    return now if args.empty?
    zone = args.size > 6 ? args[6] : nil
    fields = args[0, 6]
    return from_exact(calendar_seconds(fields, false), false, nil) if zone.nil?
    # A zone may be an object that converts between local and UTC readings,
    # which is what a timezone library hands over.
    if zone.respond_to?(:local_to_utc)
      reading = calendar_seconds(fields, true)
      counted = zone.local_to_utc(from_exact(reading, true, nil)).to_r
      made = from_exact(counted, false, (reading - counted).to_i)
      made.instance_variable_set(:@zone_object, zone)
      return made
    end
    offset = offset_seconds(zone)
    from_exact(calendar_seconds(fields, true) - offset, false, offset)
  end

  def self.at(seconds, extra = nil)
    return from_exact(seconds.to_r, seconds.utc?, nil) if seconds.is_a?(Time)
    return from_exact(seconds.to_r, false, nil) if extra.nil?
    from_exact(seconds.to_r + extra.to_r / MICROSECONDS_IN_SECOND, false, nil)
  end

  def self.utc(*args)
    from_exact(calendar_seconds(args, true), true, nil)
  end

  def self.gm(*args)
    utc(*args)
  end

  def self.local(*args)
    from_exact(calendar_seconds(args, false), false, nil)
  end

  def self.mktime(*args)
    local(*args)
  end

  # A time built from an exact count of seconds since the epoch.
  def self.from_exact(total, utc, offset)
    exact = total.to_r
    whole = exact.floor
    made = allocate
    made.instance_variable_set(:@seconds, whole)
    made.instance_variable_set(:@fraction, exact - whole)
    made.instance_variable_set(:@utc, utc)
    made.instance_variable_set(:@offset, offset)
    made
  end

  # The seconds a calendar argument list stands for, where the last argument
  # is a count of microseconds.
  def self.calendar_seconds(args, utc)
    year = args[0].to_i
    month = month_number(args[1])
    day = args[2].nil? ? 1 : args[2].to_i
    hour = args[3].nil? ? 0 : args[3].to_i
    minute = args[4].nil? ? 0 : args[4].to_i
    second = args[5].nil? ? 0 : args[5]
    micro = args.size > 6 && args[6].is_a?(Numeric) ? args[6] : 0
    whole = Time.__assemble__(year, month, day, hour, minute, second.to_i, utc)
    whole.to_r + (second.to_r - second.to_i) + micro.to_r / MICROSECONDS_IN_SECOND
  end

  def self.month_number(named)
    return 1 if named.nil?
    return named.to_i if named.is_a?(Integer)
    text = named.to_s
    return text.to_i if text.to_i > 0
    found = MONTH_NAMES.index(text.downcase[0, 3])
    raise ArgumentError, "mon out of range" if found.nil?
    found + 1
  end

  # The seconds an offset stands for, given either as a count of seconds or
  # as the `"+05:00"` spelling.
  def self.offset_seconds(offset)
    return offset if offset.is_a?(Integer)
    return offset.to_r if offset.is_a?(Numeric)
    text = offset.to_s
    return 0 if text == "UTC" || text == "Z"
    unless text =~ /\A([+-])(\d\d):?(\d\d)(?::?(\d\d))?\z/
      raise ArgumentError, "\"+HH:MM\", \"-HH:MM\", \"UTC\" or \"A\"..\"I\",\"K\"..\"Z\" expected for utc_offset: #{offset}"
    end
    counted = $2.to_i * 3600 + $3.to_i * 60 + ($4.nil? ? 0 : $4.to_i)
    $1 == "-" ? -counted : counted
  end

  # The calendar fields this time stands for, read once and kept.
  def calendar
    return @calendar unless @calendar.nil?
    @calendar = if @offset.nil?
      Time.__breakdown__(@seconds, @utc)
    else
      Time.__breakdown__((@seconds + @offset).floor, true)
    end
  end
  private :calendar

  def year
    calendar[0]
  end

  def mon
    calendar[1]
  end

  def month
    calendar[1]
  end

  def day
    calendar[2]
  end

  def mday
    calendar[2]
  end

  def hour
    calendar[3]
  end

  def min
    calendar[4]
  end

  def sec
    calendar[5]
  end

  def wday
    calendar[6]
  end

  def yday
    calendar[7]
  end

  def isdst
    @offset.nil? && !@utc && calendar[8]
  end

  def dst?
    isdst
  end

  def utc_offset
    return @offset unless @offset.nil?
    return 0 if @utc
    calendar[9]
  end

  def gmt_offset
    utc_offset
  end

  def gmtoff
    utc_offset
  end

  def zone
    return @zone_object unless @zone_object.nil?
    return nil unless @offset.nil?
    return "UTC" if @utc
    calendar[10]
  end

  def sunday?
    wday == 0
  end

  def monday?
    wday == 1
  end

  def tuesday?
    wday == 2
  end

  def wednesday?
    wday == 3
  end

  def thursday?
    wday == 4
  end

  def friday?
    wday == 5
  end

  def saturday?
    wday == 6
  end

  def to_i
    @seconds
  end

  def tv_sec
    @seconds
  end

  def to_f
    to_r.to_f
  end

  def to_r
    @seconds.to_r + @fraction
  end

  def subsec
    @fraction == 0 ? 0 : @fraction
  end

  def usec
    (@fraction * MICROSECONDS_IN_SECOND).to_i
  end

  def tv_usec
    usec
  end

  def nsec
    (@fraction * NANOSECONDS_IN_SECOND).to_i
  end

  def tv_nsec
    nsec
  end

  def utc?
    @utc
  end

  def gmt?
    @utc
  end

  def utc
    return self if @utc && @offset.nil?
    @utc = true
    @offset = nil
    @calendar = nil
    self
  end

  def gmtime
    utc
  end

  def getutc
    Time.from_exact(to_r, true, nil)
  end

  def getgm
    getutc
  end

  def localtime(offset = nil)
    @utc = false
    @offset = offset.nil? ? nil : Time.offset_seconds(offset)
    @calendar = nil
    self
  end

  def getlocal(offset = nil)
    Time.from_exact(to_r, false, offset.nil? ? nil : Time.offset_seconds(offset))
  end

  def +(other)
    raise TypeError, "time + time?" if other.is_a?(Time)
    shifted(to_r + exact_seconds(other))
  end

  def -(other)
    return (to_r - other.to_r).to_f if other.is_a?(Time)
    shifted(to_r - exact_seconds(other))
  end

  # A time at another point, reading in the same zone this one does.
  def shifted(total)
    made = Time.from_exact(total, @utc, @offset)
    made.instance_variable_set(:@zone_object, @zone_object) unless @zone_object.nil?
    made
  end
  private :shifted

  # The seconds an argument to `+` or `-` stands for. A String names no
  # number of seconds however much it looks like one.
  def exact_seconds(other)
    raise TypeError, "can't convert String into an exact number" if other.is_a?(String)
    return other.to_r if other.is_a?(Numeric)
    unless other.respond_to?(:to_r)
      raise TypeError, "can't convert #{other.class} into an exact number"
    end
    other.to_r
  end
  private :exact_seconds

  def <=>(other)
    return nil unless other.is_a?(Time)
    to_r <=> other.to_r
  end

  def ==(other)
    other.is_a?(Time) && to_r == other.to_r
  end

  def eql?(other)
    other.is_a?(Time) && to_r == other.to_r
  end

  def hash
    to_r.hash
  end

  def floor(digits = 0)
    scale = 10 ** digits
    Time.from_exact(Rational((to_r * scale).floor, scale), @utc, @offset)
  end

  def ceil(digits = 0)
    scale = 10 ** digits
    Time.from_exact(Rational((to_r * scale).ceil, scale), @utc, @offset)
  end

  def round(digits = 0)
    scale = 10 ** digits
    Time.from_exact(Rational((to_r * scale).round, scale), @utc, @offset)
  end

  def to_a
    [sec, min, hour, mday, mon, year, wday, yday, isdst, zone]
  end

  def deconstruct_keys(keys)
    all = {
      year: year, month: month, day: day, yday: yday, wday: wday,
      hour: hour, min: min, sec: sec, subsec: subsec, dst: dst?, zone: zone
    }
    return all if keys.nil?
    unless keys.is_a?(Array)
      raise TypeError, "wrong argument type #{keys.class} (expected Array or nil)"
    end
    picked = {}
    keys.each { |key| picked[key] = all[key] if all.key?(key) }
    picked
  end

  def strftime(template)
    reading = @offset.nil? ? @seconds : @seconds + @offset
    Time.__format__(ruby_directives(template), reading, @offset.nil? ? @utc : true)
  end

  # The directives Ruby adds on top of the C library's, filled in before the
  # template reaches it.
  def ruby_directives(template)
    filled = template.gsub("%N", fraction_digits(9))
    filled = filled.gsub("%L", fraction_digits(3))
    filled = filled.gsub("%:z", offset_label(true))
    filled.gsub("%z", offset_label(false))
  end
  private :ruby_directives

  # The fraction of a second, written to the given number of digits.
  def fraction_digits(places)
    scaled = (@fraction * 10 ** places).round.to_i
    scaled.to_s.rjust(places, "0")
  end
  private :fraction_digits

  # The offset from UTC as `+0900`, or as `+09:00` when asked for the spelling
  # that carries a colon.
  def offset_label(with_colon)
    counted = utc_offset
    sign = counted < 0 ? "-" : "+"
    counted = counted.abs
    hours = (counted / 3600).to_s.rjust(2, "0")
    minutes = (counted % 3600 / 60).to_s.rjust(2, "0")
    "#{sign}#{hours}#{with_colon ? ":" : ""}#{minutes}"
  end
  private :offset_label

  def asctime
    strftime("%a %b %e %H:%M:%S %Y")
  end

  def ctime
    asctime
  end

  # A time writes itself with nothing but ASCII in it, which is the encoding
  # Ruby tags the result with.
  def to_s
    "#{strftime("%Y-%m-%d %H:%M:%S")} #{zone_label}".force_encoding(Encoding::US_ASCII)
  end

  def inspect
    "#{strftime("%Y-%m-%d %H:%M:%S")}#{fraction_label} #{zone_label}".force_encoding(Encoding::US_ASCII)
  end

  # What follows the seconds in a rendering: `UTC` for a time read in UTC,
  # and the offset otherwise.
  def zone_label
    @utc && @offset.nil? ? "UTC" : offset_label(false)
  end
  private :zone_label

  # The fraction of a second a rendering shows, with the trailing zeros left
  # off and nothing at all for a whole second.
  def fraction_label
    return "" if @fraction == 0
    digits = fraction_digits(9).sub(/0+\z/, "")
    ".#{digits}"
  end
  private :fraction_label

  def iso8601(fraction = 0)
    xmlschema(fraction)
  end

  def xmlschema(fraction = 0)
    written = "#{year_label}-#{padded(mon)}-#{padded(day)}"
    written += "T#{padded(hour)}:#{padded(min)}:#{padded(sec)}"
    written += ".#{fraction_digits(fraction)}" if fraction > 0
    "#{written}#{@utc && @offset.nil? ? "Z" : offset_label(true)}"
  end

  # The year an ISO-8601 rendering shows, which runs to four digits at least
  # and carries a sign when it falls before the common era.
  def year_label
    counted = year
    return "-#{counted.abs.to_s.rjust(4, "0")}" if counted < 0
    counted.to_s.rjust(4, "0")
  end
  private :year_label

  def padded(field)
    field.to_s.rjust(2, "0")
  end
  private :padded

  def succ
    self + 1
  end
end


class IO
  SEEK_SET = 0
  SEEK_CUR = 1
  SEEK_END = 2
end

# The numbers the operating system keeps about a file, presented the way Ruby
# presents them.
class File
  class Stat
    include Comparable

    def initialize(path, follow = true)
      unless path.is_a?(String)
        unless path.respond_to?(:to_path)
          raise TypeError, "no implicit conversion of #{path.class} into String"
        end
        path = path.to_path
      end
      @path = path.to_s
      @fields = File.__stat_fields__(@path, follow)
    end

    def dev
      @fields[:dev]
    end

    def dev_major
      (@fields[:dev] >> 24) & 0xff
    end

    def dev_minor
      @fields[:dev] & 0xffffff
    end

    def ino
      @fields[:ino]
    end

    def mode
      @fields[:mode]
    end

    def nlink
      @fields[:nlink]
    end

    def uid
      @fields[:uid]
    end

    def gid
      @fields[:gid]
    end

    def rdev
      @fields[:rdev]
    end

    def rdev_major
      (@fields[:rdev] >> 24) & 0xff
    end

    def rdev_minor
      @fields[:rdev] & 0xffffff
    end

    def size
      @fields[:size]
    end

    def size?
      @fields[:size] == 0 ? nil : @fields[:size]
    end

    def blksize
      @fields[:blksize]
    end

    def blocks
      @fields[:blocks]
    end

    def atime
      Time.at(@fields[:atime])
    end

    def mtime
      Time.at(@fields[:mtime])
    end

    def ctime
      Time.at(@fields[:ctime])
    end

    def birthtime
      raise NotImplementedError, "birthtime() function is unimplemented" if @fields[:birthtime].nil?
      Time.at(@fields[:birthtime])
    end

    def ftype
      @fields[:ftype]
    end

    def file?
      @fields[:ftype] == "file"
    end

    def directory?
      @fields[:ftype] == "directory"
    end

    def symlink?
      @fields[:ftype] == "link"
    end

    def chardev?
      @fields[:ftype] == "characterSpecial"
    end

    def blockdev?
      @fields[:ftype] == "blockSpecial"
    end

    def pipe?
      @fields[:ftype] == "fifo"
    end

    def socket?
      @fields[:ftype] == "socket"
    end

    def zero?
      @fields[:size] == 0
    end

    # The permission bits, which say who may read, write, and run the file.
    def setuid?
      (@fields[:mode] & 0o4000) != 0
    end

    def setgid?
      (@fields[:mode] & 0o2000) != 0
    end

    def sticky?
      (@fields[:mode] & 0o1000) != 0
    end

    def world_readable?
      (@fields[:mode] & 0o004) == 0 ? nil : @fields[:mode] & 0o7777
    end

    def world_writable?
      (@fields[:mode] & 0o002) == 0 ? nil : @fields[:mode] & 0o7777
    end

    def owned?
      @fields[:uid] == Process.uid
    end

    # A file belongs to this process's group when its group is any of the
    # ones the process is in, not only the one it runs as.
    def grpowned?
      return true if @fields[:gid] == Process.gid
      Process.groups.include?(@fields[:gid])
    end

    def readable?
      self.permitted?(0o400, 0o040, 0o004)
    end

    def writable?
      self.permitted?(0o200, 0o020, 0o002)
    end

    def executable?
      self.permitted?(0o100, 0o010, 0o001)
    end

    def readable_real?
      self.readable?
    end

    def writable_real?
      self.writable?
    end

    def executable_real?
      self.executable?
    end

    # Whether this process may do the thing the three bits stand for, read
    # against whichever of owner, group, and other it counts as.
    def permitted?(owner, group, other)
      return true if Process.uid == 0
      return (@fields[:mode] & owner) != 0 if @fields[:uid] == Process.uid
      return (@fields[:mode] & group) != 0 if @fields[:gid] == Process.gid
      (@fields[:mode] & other) != 0
    end
    private :permitted?

    def <=>(other)
      return nil unless other.is_a?(File::Stat)
      @fields[:mtime] <=> other.mtime.to_i
    end

    def inspect
      written = "#<File::Stat dev=0x#{self.dev.to_s(16)}, ino=#{self.ino}"
      written = written + ", mode=#{"%07o" % self.mode}, nlink=#{self.nlink}"
      written = written + ", uid=#{self.uid}, gid=#{self.gid}"
      written = written + ", rdev=0x#{self.rdev.to_s(16)}, size=#{self.size}"
      written = written + ", blksize=#{self.blksize.inspect}, blocks=#{self.blocks.inspect}"
      written = written + ", atime=#{self.atime.inspect}, mtime=#{self.mtime.inspect}"
      written = written + ", ctime=#{self.ctime.inspect}"
      written = written + ", birthtime=#{self.birthtime.inspect}" unless @fields[:birthtime].nil?
      written + ">"
    end

    def to_s
      self.inspect
    end
  end

  def self.stat(path)
    File::Stat.new(path, true)
  end

  def self.lstat(path)
    File::Stat.new(path, false)
  end

  def self.birthtime(path)
    File::Stat.new(path, true).birthtime
  end

  # The questions about a file that read what the operating system keeps
  # about it. A name with nothing behind it answers the way Ruby's does
  # rather than raising.
  def self.__stat_answer__(path, follow, missing, &block)
    begin
      held = File::Stat.new(path, follow)
    rescue SystemCallError, Errno::ENOENT
      return missing
    end
    block.call(held)
  end

  def self.ftype(path)
    File::Stat.new(path, false).ftype
  end

  def self.zero?(path)
    self.__stat_answer__(path, true, false) { |held| held.zero? }
  end

  def self.empty?(path)
    self.zero?(path)
  end

  def self.world_readable?(path)
    self.__stat_answer__(path, true, nil) { |held| held.world_readable? }
  end

  def self.world_writable?(path)
    self.__stat_answer__(path, true, nil) { |held| held.world_writable? }
  end

  def self.readable?(path)
    self.__stat_answer__(path, true, false) { |held| held.readable? }
  end

  def self.readable_real?(path)
    self.readable?(path)
  end

  def self.writable?(path)
    self.__stat_answer__(path, true, false) { |held| held.writable? }
  end

  def self.writable_real?(path)
    self.writable?(path)
  end

  def self.executable_real?(path)
    self.__stat_answer__(path, true, false) { |held| held.executable? }
  end

  def self.owned?(path)
    self.__stat_answer__(path, true, false) { |held| held.owned? }
  end

  def self.grpowned?(path)
    self.__stat_answer__(path, true, false) { |held| held.grpowned? }
  end

  def self.setuid?(path)
    self.__stat_answer__(path, true, false) { |held| held.setuid? }
  end

  def self.setgid?(path)
    self.__stat_answer__(path, true, false) { |held| held.setgid? }
  end

  def self.sticky?(path)
    self.__stat_answer__(path, true, false) { |held| held.sticky? }
  end

  def self.blockdev?(path)
    self.__stat_answer__(path, false, false) { |held| held.blockdev? }
  end

  def self.chardev?(path)
    self.__stat_answer__(path, false, false) { |held| held.chardev? }
  end

  def self.pipe?(path)
    self.__stat_answer__(path, false, false) { |held| held.pipe? }
  end

  def self.socket?(path)
    self.__stat_answer__(path, false, false) { |held| held.socket? }
  end

  # Two names stand for the same file when the device and the number the
  # filesystem keeps it under both match.
  def self.identical?(one, other)
    first = self.__stat_answer__(one, true, nil) { |held| held }
    second = self.__stat_answer__(other, true, nil) { |held| held }
    return false if first.nil? || second.nil?
    first.dev == second.dev && first.ino == second.ino
  end

  # `File.stat` and `File.lstat` read a name, and the instance forms read the
  # name the handle was opened under. The two differ over a symlink: `stat`
  # follows it to what it points at, `lstat` reports the link itself.
  def self.stat(path)
    File::Stat.new(path)
  end

  def self.lstat(path)
    File::Stat.new(path, false)
  end

  def stat
    File::Stat.new(self.path)
  end

  def lstat
    File::Stat.new(self.path, false)
  end

  # The name the handle was opened under. An IO that never came from a name
  # has none, which is what `to_path` answers for.
  def path
    @__file_path
  end

  def to_path
    @__file_path
  end

  # An IO stands for itself where one is asked for.
  def to_io
    self
  end

  # The time the file was created, which the filesystem records separately
  # from the last write.
  # A name nothing stands for is refused rather than answered with nil, the
  # way every other reading of a missing file is.
  def self.birthtime(path)
    File::Stat.new(path).birthtime
  end

  def birthtime
    File.birthtime(self.path)
  end
end

# An open directory, walked one name at a time.
# The maps that hold their entries only as long as something else does. Both
# are written here as ordinary maps, since metorex frees an object when the
# last reference to it goes and never before.
# A hook the interpreter calls as it runs. A TracePoint is built over the
# events it cares about, switched on around a block, and handed itself when
# one of those events happens. The readings it answers describe the event
# being handled, so asking for one outside a handler is refused.
class TracePoint
  KNOWN_EVENTS = [:line, :call, :return, :c_call, :c_return, :class, :end,
                  :b_call, :b_return, :raise, :rescue, :thread_begin,
                  :thread_end, :fiber_switch, :script_compiled, :a_call,
                  :a_return]

  def self.new(*events, &block)
    raise ArgumentError, "must be called with a block" if block.nil?
    events.each do |event|
      unless KNOWN_EVENTS.include?(event)
        raise ArgumentError, "unknown event: #{event}"
      end
    end
    made = allocate
    made.send(:__set_up__, events, block)
    made
  end

  def self.trace(*events, &block)
    made = new(*events, &block)
    made.enable
    made
  end

  def __set_up__(events, block)
    @events = events.empty? ? KNOWN_EVENTS : events
    @block = block
    @enabled = false
    @handling = nil
    self
  end
  private :__set_up__

  def enabled?
    @enabled == true
  end

  # With a block, the trace is on for the length of it and the block's value
  # is the answer. Without one, the answer is what the switch was before.
  def enable(target: nil, target_line: nil, target_thread: nil, &block)
    was = enabled?
    @enabled = true
    __register__
    return was if block.nil?
    begin
      block.call
    ensure
      @enabled = was
      __register__
    end
  end

  def disable(&block)
    was = enabled?
    @enabled = false
    __register__
    return was if block.nil?
    begin
      block.call
    ensure
      @enabled = was
      __register__
    end
  end

  # Ruby switches a trace off while its own handler runs, so an event the
  # handler causes does not call it again. `allow_reentry` lifts that for the
  # length of a block.
  # Ruby switches a trace off while its own handler runs, so an event the
  # handler causes does not call it again. `allow_reentry` lifts that for the
  # length of a block, and is refused outside a handler.
  def self.allow_reentry
    raise RuntimeError, "allow_reentry is not allowed outside of a trace" unless __tracing__
    yield
  end

  def event
    __reading__(:event)
  end

  def lineno
    __reading__(:lineno)
  end

  def path
    __reading__(:path)
  end

  def self
    __reading__(:self)
  end

  def method_id
    __reading__(:method_id)
  end

  def return_value
    __reading__(:return_value)
  end

  def callee_id
    __reading__(:callee_id)
  end

  def defined_class
    __reading__(:defined_class)
  end

  def inspect
    return "#<TracePoint:disabled>" if @handling.nil?
    "#<TracePoint:#{@handling["event"]}@#{@handling["path"]}:#{@handling["lineno"]}>"
  end

  # What the event being handled says about itself. Nothing is being handled
  # outside a handler, which is what Ruby reports.
  def __reading__(name)
    raise RuntimeError, "access from outside" if @handling.nil?
    @handling[name.to_s]
  end
  private :__reading__

  def __handle__(details)
    held = @handling
    @handling = details
    begin
      @block.call(self)
    ensure
      @handling = held
    end
  end

  def __wants__(event)
    @events.include?(event)
  end
end

# The format version `Marshal.dump` writes and `Marshal.load` reads. Metorex
# writes no marshalled data yet, and these name the format it would be.
module Marshal
  MAJOR_VERSION = 4
  MINOR_VERSION = 8
end

module ObjectSpace
  # Keyed by identity: two objects that are equal but not the same are two
  # keys.
  class WeakMap
    include Enumerable

    def initialize
      @entries = []
    end

    def []=(key, value)
      place = place_of(key)
      if place.nil?
        @entries.push([key, value])
      else
        @entries[place] = [key, value]
      end
      value
    end

    def [](key)
      place = place_of(key)
      place.nil? ? nil : @entries[place][1]
    end

    def delete(key)
      place = place_of(key)
      if place.nil?
        return yield(key) if block_given?
        return nil
      end
      @entries.delete_at(place)[1]
    end

    def key?(key)
      !place_of(key).nil?
    end

    def member?(key)
      key?(key)
    end

    def include?(key)
      key?(key)
    end

    def has_key?(key)
      key?(key)
    end

    def key(value)
      @entries.each do |entry|
        return entry[0] if entry[1].equal?(value)
      end
      nil
    end

    def size
      @entries.size
    end

    def length
      size
    end

    def keys
      @entries.map { |entry| entry[0] }
    end

    def values
      @entries.map { |entry| entry[1] }
    end

    # A walk with no block is refused once there is anything to walk, which
    # is what Ruby does with a map that cannot answer an enumerator.
    def each
      unless block_given?
        return self if @entries.empty?
        raise LocalJumpError, "no block given (yield)"
      end
      @entries.each { |entry| yield entry[0], entry[1] }
      self
    end

    def each_pair(&block)
      each(&block)
    end

    def each_key
      unless block_given?
        return self if @entries.empty?
        raise LocalJumpError, "no block given (yield)"
      end
      @entries.each { |entry| yield entry[0] }
      self
    end

    def each_value
      unless block_given?
        return self if @entries.empty?
        raise LocalJumpError, "no block given (yield)"
      end
      @entries.each { |entry| yield entry[1] }
      self
    end

    # Where a key sits, found by identity rather than by value.
    def place_of(key)
      @entries.each_with_index do |entry, place|
        return place if entry[0].equal?(key)
      end
      nil
    end
    private :place_of
  end

  # Keyed by value, and only by something the collector could free, so a
  # number or a symbol is refused as a key.
  class WeakKeyMap
    def initialize
      @entries = []
    end

    def []=(key, value)
      unless collectable?(key)
        raise ArgumentError, "WeakKeyMap must be garbage collectable"
      end
      place = place_of(key)
      if place.nil?
        @entries.push([key, value])
      else
        @entries[place][1] = value
      end
      value
    end

    def [](key)
      return nil unless collectable?(key)
      place = place_of(key)
      place.nil? ? nil : @entries[place][1]
    end

    def delete(key)
      place = collectable?(key) ? place_of(key) : nil
      if place.nil?
        return yield(key) if block_given?
        return nil
      end
      @entries.delete_at(place)[1]
    end

    def getkey(key)
      return nil unless collectable?(key)
      place = place_of(key)
      place.nil? ? nil : @entries[place][0]
    end

    def key?(key)
      return false unless collectable?(key)
      !place_of(key).nil?
    end

    def clear
      @entries = []
      self
    end

    def size
      @entries.size
    end

    def length
      size
    end

    def inspect
      "#<ObjectSpace::WeakKeyMap:0x#{format("%016x", object_id * 2)} size=#{size}>"
    end

    def to_s
      inspect
    end

    # Whether a key is something the collector could free. A number, a
    # symbol, and the three singletons live for the whole run.
    def collectable?(key)
      return false if key.nil? || key == true || key == false
      return false if key.is_a?(Numeric) || key.is_a?(Symbol)
      true
    end
    private :collectable?

    # Where a key sits. The hash decides which entries are worth comparing,
    # and the same object is its own match however its `eql?` answers.
    def place_of(key)
      wanted = key.__send__(:hash)
      @entries.each_with_index do |entry, place|
        held = entry[0]
        next unless held.__send__(:hash) == wanted
        return place if held.equal?(key) || key.__send__(:eql?, held)
      end
      nil
    end
    private :place_of
  end
end

# The file questions File answers, gathered as a module so they can be asked
# without naming File and mixed into anything that wants them.
module FileTest
  module_function

  def blockdev?(path)
    File.blockdev?(path)
  end

  def chardev?(path)
    File.chardev?(path)
  end

  def directory?(path)
    File.directory?(path)
  end

  def empty?(path)
    File.empty?(path)
  end

  def executable?(path)
    File.executable?(path)
  end

  def executable_real?(path)
    File.executable_real?(path)
  end

  def exist?(path)
    File.exist?(path)
  end

  def file?(path)
    File.file?(path)
  end

  def grpowned?(path)
    File.grpowned?(path)
  end

  def identical?(path, other)
    File.identical?(path, other)
  end

  def owned?(path)
    File.owned?(path)
  end

  def pipe?(path)
    File.pipe?(path)
  end

  def readable?(path)
    File.readable?(path)
  end

  def readable_real?(path)
    File.readable_real?(path)
  end

  def setgid?(path)
    File.setgid?(path)
  end

  def setuid?(path)
    File.setuid?(path)
  end

  def size(path)
    File.size(path)
  end

  def size?(path)
    File.size?(path)
  end

  def socket?(path)
    File.socket?(path)
  end

  def sticky?(path)
    File.sticky?(path)
  end

  def symlink?(path)
    File.symlink?(path)
  end

  def world_readable?(path)
    File.world_readable?(path)
  end

  def world_writable?(path)
    File.world_writable?(path)
  end

  def writable?(path)
    File.writable?(path)
  end

  def writable_real?(path)
    File.writable_real?(path)
  end

  def zero?(path)
    File.zero?(path)
  end
end

class Dir
  include Enumerable

  def initialize(path, **options)
    unless path.is_a?(String)
      unless path.respond_to?(:to_path)
        raise TypeError, "no implicit conversion of #{path.class} into String"
      end
      path = path.to_path
    end
    @path = path.to_s
    unless File.directory?(@path)
      raise Errno::ENOENT, "No such file or directory @ dir_initialize - #{@path}"
    end
    @names = Dir.entries(@path)
    @position = 0
    @closed = false
  end

  def self.open(path, **options, &block)
    made = Dir.new(path, **options)
    return made if block.nil?
    begin
      block.call(made)
    ensure
      made.close
    end
  end

  # The path stands whether the directory is still open or not, which is what
  # Ruby answers for a closed one.
  def path
    @path
  end

  def to_path
    @path
  end

  def inspect
    "#<Dir:#{@path}>"
  end

  def closed?
    @closed
  end

  def close
    @closed = true
    nil
  end

  def fileno
    self.refuse_closed
    raise NotImplementedError, "fileno() function is unimplemented on this machine"
  end

  # Every operation that walks the names needs the directory still open.
  def refuse_closed
    raise IOError, "closed directory" if @closed
  end
  private :refuse_closed

  def read
    self.refuse_closed
    return nil if @position >= @names.length
    found = @names[@position]
    @position += 1
    found
  end

  def pos
    self.refuse_closed
    @position
  end

  def tell
    self.pos
  end

  def seek(position)
    self.refuse_closed
    @position = position
    self
  end

  def pos=(position)
    self.seek(position)
    position
  end

  def rewind
    self.refuse_closed
    @position = 0
    self
  end

  # Walking the whole directory starts at the beginning and leaves the
  # position at the end, which is where a read after it finds nothing.
  def each(&block)
    self.refuse_closed
    return self.to_enum(:each) if block.nil?
    @position = 0
    @names.each { |name| block.call(name) }
    @position = @names.length
    self
  end

  def each_child(&block)
    self.refuse_closed
    walked = @names.reject { |name| name == "." || name == ".." }
    return walked.each if block.nil?
    @position = 0
    walked.each { |name| block.call(name) }
    @position = @names.length
    self
  end

  def children
    self.refuse_closed
    @names.reject { |name| name == "." || name == ".." }
  end

  def entries
    self.refuse_closed
    @names.dup
  end
end

module Kernel
  # `pretty_inspect` is what `pp` writes for an object, which is its own
  # `inspect` on a line of its own. `require "pp"` is what defines it in
  # Ruby, and metorex reports pp as already loaded.
  def pretty_inspect
    inspect.to_s + "\n"
  end

  # Ruby calls into the operating system by number here. Metorex does not
  # reach the system call layer at all, which is what Ruby itself reports on
  # a platform that cannot.
  def syscall(*args)
    raise NotImplementedError, "syscall() function is unimplemented on this machine"
  end
  private :syscall

  # Ruby hands each interpreter event to the block set here. Metorex has no
  # tracing hook for the evaluator to call, so there is nothing to set.
  def set_trace_func(callable)
    raise NotImplementedError, "set_trace_func() function is unimplemented on this machine"
  end
  private :set_trace_func
end

# The stream `gets` reads from when a script is handed filenames: each named
# file in turn, read as though the whole list were one file. `ARGF` is the one
# the interpreter set up over ARGV, and `ARGF.class.new` builds another over a
# list of names, which is how the specs read a pair of fixtures.
ArgfStream = ARGF.class

class ArgfStream
  def initialize(*names)
    @names = names.flatten
    @current = nil
    @lineno = 0
    @binmode = false
    @drained = false
  end

  def to_s
    "ARGF"
  end

  def inspect
    "ARGF"
  end

  # The names still to be read. The one being read has already been taken off,
  # which is what makes `argv` shrink as the walk goes on.
  def argv
    self.__names__
  end

  # The handle now being read. The first name opens on the first ask, so the
  # file is current before a line has been taken from it.
  def file
    self.__open_current__
    @current
  end

  def to_io
    self.file
  end

  def path
    self.file.path
  end

  def filename
    self.path
  end

  def fileno
    raise ArgumentError, "closed stream" if self.__names__.empty? && @current.nil?
    self.file.fileno
  end

  def to_i
    self.fileno
  end

  def lineno
    self.__lineno__
  end

  def lineno=(counted)
    @lineno = counted
  end

  def binmode
    @binmode = true
    self
  end

  def binmode?
    @binmode == true
  end

  def closed?
    self.file.closed?
  end

  def close
    self.file.close
    self
  end

  # Move past whatever is left of the file being read, so the next line comes
  # from the one after it.
  def skip
    @current = nil unless self.__names__.empty?
    self
  end

  def gets
    loop do
      self.__open_current__
      return nil if @current.nil?
      line = @current.gets
      if line.nil?
        if self.__names__.empty?
          @drained = true
          break
        end
        @current = nil
        next
      end
      @lineno = self.__lineno__ + 1
      return line
    end
    nil
  end

  def readline
    line = self.gets
    raise EOFError, "end of file reached" if line.nil?
    line
  end

  def each_line(&block)
    return self if block.nil?
    while (line = self.gets)
      block.call(line)
    end
    self
  end

  def each(&block)
    self.each_line(&block)
  end

  def readlines(*args)
    collected = []
    while (line = self.gets)
      collected.push line
    end
    collected
  end

  def to_a(*args)
    self.readlines
  end

  # `read(length)` stops at the count it was asked for, crossing into the next
  # file only when the one being read runs out first. With no count it drains
  # every remaining file.
  def read(length = nil)
    collected = ""
    loop do
      self.__open_current__
      break if @current.nil?
      wanted = length.nil? ? nil : length - collected.length
      break if !wanted.nil? && wanted <= 0
      taken = wanted.nil? ? @current.read : @current.read(wanted)
      collected = collected + taken.to_s
      break if !wanted.nil? && collected.length >= length
      if self.__names__.empty?
        @drained = true
        break
      end
      @current = nil
    end
    return nil if !length.nil? && length > 0 && collected.empty?
    collected
  end

  def getc
    loop do
      self.__open_current__
      return nil if @current.nil?
      character = @current.getc
      return character unless character.nil?
      return nil if self.__names__.empty?
      @current = nil
    end
  end

  def readchar
    character = self.getc
    raise EOFError, "end of file reached" if character.nil?
    character
  end

  # Whether the file being read has run out, which is asked per file rather
  # than of the whole list. A stream whose last file was drained is closed,
  # and refuses the question.
  def eof?
    raise IOError, "closed stream" if @drained
    self.__open_current__
    return true if @current.nil?
    @current.eof?
  end

  def eof
    self.eof?
  end

  # Where the file being read stands. A stream whose last file has been read
  # to the end is closed, and refuses to report a position at all.
  def pos
    raise ArgumentError, "closed stream" if @drained
    self.file.pos
  end

  def tell
    self.pos
  end

  def pos=(offset)
    self.file.pos = offset
    offset
  end

  def rewind
    self.file.rewind
    @lineno = 0
    @drained = false
    0
  end

  # Open the next name when there is no file being read. The name comes off
  # the list as it opens, which is what `argv` reports on.
  def __open_current__
    return if @current
    return if self.__names__.empty?
    @current = File.open(self.__names__.shift, "r")
  end
  private :__open_current__

  # The names still to be read. The interpreter builds the global ARGF without
  # running `initialize`, and that one reads ARGV itself, so a name taken off
  # here is taken off ARGV too.
  def __names__
    @names = ARGV if @names.nil?
    @names
  end
  private :__names__

  # How many lines have been read, zero before any have been.
  def __lineno__
    @lineno = 0 if @lineno.nil?
    @lineno
  end
  private :__lineno__
end

# A named set of threads. Ruby starts every thread in the default group and
# moves it when another group takes it, and an enclosed group refuses to give
# its threads up.
class Complex
  # What `Marshal` writes for a Complex: the two parts, in the order
  # `Complex(real, imaginary)` takes them.
  def marshal_dump
    [real, imaginary]
  end
  private :marshal_dump
end

class Rational
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
  # Whether a pattern is matched in time proportional to the subject's length.
  # Metorex matches with a linear automaton, so every pattern is.
  def self.linear_time?(pattern, options = nil)
    true
  end
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

module ObjectSpace
  # Ruby 4.0 deprecated reading an object back from its id. Metorex keeps no
  # table of every live object, so only the values whose id is derived from
  # the value itself can be answered at all.
  def self._id2ref(id)
    warn "warning: ObjectSpace._id2ref is deprecated"
    return nil if id == 4
    return true if id == 2
    return false if id == 0
    return (id - 1) / 2 if id.odd? || id < 0
    raise RangeError, "#{id} is not an id value"
  end
end

module GC
  # An object that extends GC answers `garbage_collect` the way the module
  # itself does, and Ruby documents the answer as always nil.
  def garbage_collect(full_mark: true, immediate_sweep: true)
    GC.start
    nil
  end

  # Metorex frees an object when its last reference goes, so switching the
  # collector off leaves nothing running differently. The switch is still read
  # back the way it was written, which is what a program that saves and
  # restores it depends on.
  def self.disable
    was = @disabled == true
    @disabled = true
    was
  end

  def self.enable
    was = @disabled == true
    @disabled = false
    was
  end

  def self.auto_compact
    @auto_compact == true
  end

  def self.auto_compact=(wanted)
    @auto_compact = wanted
  end

  def self.measure_total_time
    @measure_total_time == true
  end

  def self.measure_total_time=(wanted)
    @measure_total_time = wanted
  end

  # Ruby collects on every allocation while this is set, which is a way to
  # shake out collector bugs. There is no collector here to drive that hard,
  # so the switch is read back the way it was written and nothing else.
  def self.stress
    @stress == true
  end

  def self.stress=(wanted)
    @stress = wanted
  end

  # Ruby's collector reports what each run cost when the profiler is enabled.
  # There are no runs to report here, so the report stays empty however the
  # switch is set.
  module Profiler
    def self.enabled?
      @enabled == true
    end

    def self.enable
      @enabled = true
      nil
    end

    def self.disable
      @enabled = false
      nil
    end

    def self.clear
      nil
    end

    def self.result
      ""
    end

    def self.report(target = nil)
      nil
    end

    def self.total_time
      0.0
    end
  end
end

module Process
  # The four processor-time readings `Process.times` reports: this process's
  # own user and system time, and the totals for the children it waited for.
  Tms = Struct.new(:utime, :stime, :cutime, :cstime)
end

"##;

impl VirtualMachine {
    /// Build an `Enumerator` over `method_name` sent to `receiver`, which is
    /// what a method that yields answers when called without a block.
    pub(crate) fn build_enumerator(
        &mut self,
        receiver: crate::object::Object,
        method_name: &str,
        arguments: Vec<crate::object::Object>,
        size: Option<i64>,
        position: crate::lexer::Position,
    ) -> Result<crate::object::Object, crate::error::MetorexError> {
        use crate::object::Object;
        let Some(enumerator_class) = self.globals().get("Enumerator") else {
            let message = "uninitialized constant Enumerator".to_string();
            return Err(crate::error::MetorexError::UncaughtException {
                exception: Object::exception("NameError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        };
        let arguments = vec![
            receiver,
            Object::symbol(method_name.to_string()),
            Object::Array(std::rc::Rc::new(std::cell::RefCell::new(arguments))),
            match size {
                Some(size) => Object::Int(size),
                None => Object::Nil,
            },
        ];
        self.send_to_object(enumerator_class, "new", arguments, position)
    }

    /// Evaluate the Ruby-level core library. A parse or runtime failure here
    /// is a defect in `PRELUDE_SOURCE` itself, so it panics rather than
    /// leaving a half-built VM behind.
    pub(crate) fn load_prelude(&mut self) {
        let tokens = crate::lexer::Lexer::for_prelude(PRELUDE_SOURCE).tokenize();
        let statements = crate::parser::Parser::new(tokens)
            .parse()
            .unwrap_or_else(|errors| panic!("prelude failed to parse: {:?}", errors));
        self.execute_program(&statements)
            .unwrap_or_else(|error| panic!("prelude failed to run: {}", error));
    }
}
