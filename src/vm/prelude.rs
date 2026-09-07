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

  def max_by(*count)
    return sized_enum(:max_by, *count) unless block_given?
    ordered = sort_by { |*values| yield(packed(values)) }
    return ordered.last if count.empty? || count[0].nil?
    ordered.last(count[0]).reverse
  end

  def minmax_by(&block)
    return sized_enum(:minmax_by) unless block_given?
    ordered = sort_by(&block)
    [ordered.first, ordered.last]
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

  # Collects what a generator block hands it, so `Enumerator.new { |y| y << 1 }`
  # reads the same way as one built over a method that yields.
  class Yielder
    def initialize(collected)
      @collected = collected
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

  def initialize(receiver = nil, method_name = nil, arguments = [], size = nil, &generator)
    @receiver = receiver
    @method_name = method_name
    @arguments = arguments
    @size = size
    @position = 0
    @generator = generator
  end

  def size
    @size
  end

  def to_a
    if @values.nil?
      collected = []
      if @generator.nil?
        @result = @receiver.send(@method_name, *@arguments) do |*yielded|
          collected.push(yielded.empty? ? nil : (yielded.size == 1 ? yielded[0] : yielded))
        end
      else
        @result = @generator.call(Yielder.new(collected))
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
        end
      else
        @result = @generator.call(Yielder.new(collected))
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

  def rewind
    @position = 0
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

  def to_s
    "#{strftime("%Y-%m-%d %H:%M:%S")} #{zone_label}"
  end

  def inspect
    "#{strftime("%Y-%m-%d %H:%M:%S")}#{fraction_label} #{zone_label}"
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
            Object::Symbol(std::rc::Rc::new(method_name.to_string())),
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
        let tokens = crate::lexer::Lexer::new(PRELUDE_SOURCE).tokenize();
        let statements = crate::parser::Parser::new(tokens)
            .parse()
            .unwrap_or_else(|errors| panic!("prelude failed to parse: {:?}", errors));
        self.execute_program(&statements)
            .unwrap_or_else(|error| panic!("prelude failed to run: {}", error));
    }
}
