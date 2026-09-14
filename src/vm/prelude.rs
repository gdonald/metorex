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
  # Exact division. A number metorex has no arithmetic of its own for is
  # asked for the rational it stands for, and that is divided instead.
  def quo(other)
    converted = to_r
    unless converted.is_a?(Rational)
      raise TypeError, "can't convert #{self.class} into Rational"
    end
    converted / other
  end

  # A real number stands at angle zero when it is positive and at Pi when it
  # is negative, which is where it sits on the complex plane.
  def arg
    return self if respond_to?(:nan?) && nan?
    self < 0 ? Math::PI : 0
  end

  alias_method :angle, :arg
  alias_method :phase, :arg

  # The same number written as a distance and an angle.
  def polar
    [abs, arg]
  end

  # A real number has no imaginary part, so its rectangular form is itself
  # and zero.
  def rect
    [self, 0]
  end

  alias_method :rectangular, :rect

  # The square of the distance from zero, which a number reaches by
  # multiplying itself.
  def abs2
    self * self
  end

  # `2.i` is the complex number whose imaginary part is the number itself.
  def i
    Complex(0, self)
  end

  # A number stands as the real part of a complex number with no imaginary
  # part of its own.
  def to_c
    Complex(self, 0)
  end

  # Ruby negates a number it has no arithmetic of its own for by asking it to
  # coerce zero, then subtracting it from what came back.
  def -@
    first, second = coerce(0)
    first - second
  end

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
  # differ by one divisor whenever the signs disagree. Anything that is not
  # a number is asked to coerce the pair first.
  def remainder(other)
    mine = self
    mine, other = other.coerce(self) unless other.is_a?(Numeric)
    left = mine % other
    return left if left == 0
    return left - other if (mine < 0 && other > 0) || (mine > 0 && other < 0)
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
# back from `Regexp#match`, `String#match`, and `=~`.
class MatchData
  def initialize(string, regexp, begins, ends, names)
    # The subject is kept as a frozen copy, so a later change to the string
    # that was matched leaves what the match reports alone.
    @string = string.dup.freeze
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
      # A negative index counts back over the groups the pattern named, so it
      # never reaches the whole match at index 0.
      if key < 0
        index = key + @begins.size
        return nil if index < 1
      else
        index = key
      end
      return nil if index >= @begins.size
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

  # Where a group sat counted in bytes rather than in characters, which is
  # what a program reading the subject byte by byte needs.
  def byteoffset(key)
    index = positive_group_index(key)
    [__bytes_before__(@begins[index]), __bytes_before__(@ends[index])]
  end

  def bytebegin(key)
    __bytes_before__ @begins[positive_group_index(key)]
  end

  def byteend(key)
    __bytes_before__ @ends[positive_group_index(key)]
  end

  # How many bytes of the subject sit before a place counted in characters.
  def __bytes_before__(counted)
    return nil if counted.nil?
    @string[0, counted].bytesize
  end
  private :__bytes_before__

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

  # Only an Array names which groups to read, and each name in it must be a
  # Symbol. A key the pattern does not name ends the reading, and more keys
  # than there are named groups reads none at all.
  def deconstruct_keys(keys)
    return named_captures.transform_keys { |name| name.to_sym } if keys.nil?
    unless keys.is_a?(Array)
      raise TypeError, "wrong argument type #{keys.class} (expected Array)"
    end
    return {} if keys.size > @names.size
    collected = {}
    keys.each do |key|
      unless key.is_a?(Symbol)
        raise TypeError, "wrong argument type #{key.class} (expected Symbol)"
      end
      index = @names[key.to_s]
      return collected if index.nil?
      collected[key] = group_text(index)
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
    # How many frames under the top one a report writes out, which
    # `--backtrace-limit` settles and which is -1 when it was not written.
    def self.limit
      $__backtrace_limit__.nil? ? -1 : $__backtrace_limit__
    end

    class Location
      attr_reader :path, :lineno, :label, :absolute_path

      # The label without what it was reached through: a block's label names
      # the method holding it, and a method's label names the method alone
      # rather than the class or module it was found on.
      def base_label
        return @label if @label.nil?
        held = @label.start_with?("block ") ? @label.split(" in ", 2).last : @label
        return held if held.start_with?("<")
        held.split(/[.#]/).last
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
    # Truncating empties the buffer without changing what it is written in.
    @string = "".dup.force_encoding(@string.encoding) if @truncates
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

  # The cursor is reported in bytes, so a character made of several bytes
  # moves it by that many.
  def pos
    beyond = @position - @string.length
    beyond = 0 if beyond < 0
    @string[0, @position].bytesize + beyond
  end

  def tell
    pos
  end

  def pos=(offset)
    raise Errno::EINVAL, "Invalid argument" if offset < 0
    @position = characters_before offset
    # An offset that lands inside a character leaves the cursor between
    # two of them, which only a reader of code points minds.
    @misaligned = @string[0, @position].bytesize != offset
    offset
  end

  # The number of characters standing before a byte offset.
  def characters_before(counted)
    at = 0
    seen = 0
    while seen < counted && at < @string.length
      seen += @string[at].bytesize
      at += 1
    end
    # An offset past the end counts what lies beyond it, and one that lands
    # inside a character stops at the character it landed in.
    return at + (counted - seen) if seen < counted
    at
  end
  private :characters_before

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
    raise IOError, "closed stream" if closed?
    amount = amount.to_int if !amount.is_a?(Integer) && amount.respond_to?(:to_int)
    unless amount.is_a?(Integer)
      raise TypeError, "no implicit conversion of #{amount.class} into Integer"
    end
    base = if whence == 0
      0
    elsif whence == 1
      pos
    elsif whence == 2
      @string.bytesize
    else
      raise Errno::EINVAL, "Invalid argument"
    end
    landing = base + amount
    raise Errno::EINVAL, "Invalid argument" if landing < 0
    self.pos = landing
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
    # The buffer itself is cut or padded, so whoever handed it over sees the
    # change.
    if wanted <= @string.length
      @string.replace @string[0, wanted]
    else
      @string.replace(@string + "\0" * (wanted - @string.length))
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
    # Truncating empties the buffer without changing what it is written in.
    @string = "".dup.force_encoding(@string.encoding) if @truncates
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

  # A buffer handed in is filled with what was read and answered in place of
  # a string of its own.
  def read(length = nil, buffer = nil)
    reading_allowed
    remaining = @string[@position..-1] || ""
    if length.nil?
      @position = @string.length
      return filled(buffer, remaining)
    end
    wanted = StringIO.whole_number length
    raise ArgumentError, "negative length #{wanted} given" if wanted < 0
    if remaining.empty? && wanted > 0
      filled buffer, "" unless buffer.nil?
      return nil
    end
    # A count names bytes rather than characters, which is what a stream of
    # text holding multibyte characters reads out one piece at a time.
    taken = StringIO.__first_bytes__ remaining, wanted
    @position = @position + StringIO.__characters_for__(remaining, taken.bytesize)
    filled buffer, taken
  end

  # The first so many bytes of some text, as bytes rather than as text.
  def self.__first_bytes__(text, wanted)
    listed = text.bytes
    return text.b if listed.size <= wanted
    listed[0, wanted].pack("C*")
  end

  # How many characters the first so many bytes of some text spell. A count
  # landing inside a character counts that character as read.
  def self.__characters_for__(text, counted)
    used = 0
    walked = 0
    text.each_char do |held|
      break if used >= counted
      used += held.bytesize
      walked += 1
    end
    walked
  end

  # The number an argument stands for, refusing anything that names none.
  def self.whole_number(held)
    return held if held.is_a? Integer
    unless held.respond_to? :to_int
      raise TypeError, "no implicit conversion of #{held.class} into Integer"
    end
    held.to_int
  end

  # A buffer keeps the encoding it was tagged with, whatever the text put
  # into it was tagged with.
  def filled(buffer, text)
    return text if buffer.nil?
    unless buffer.is_a? String
      unless buffer.respond_to? :to_str
        raise TypeError, "no implicit conversion of #{buffer.class} into String"
      end
      buffer = buffer.to_str
    end
    kept = buffer.encoding
    buffer.replace text
    buffer.force_encoding kept
    buffer
  end
  private :filled

  def sysread(length = nil, buffer = nil)
    reading_allowed
    if eof? && !length.nil? && length > 0
      # The buffer holds what was read, so nothing read leaves it empty.
      buffer.replace "" unless buffer.nil?
      raise EOFError, "end of file reached"
    end
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
    reading_allowed
    return nil if letter.nil?
    text = if letter.is_a? Integer
      letter.chr
    elsif letter.is_a? String
      letter
    elsif letter.respond_to? :to_str
      letter.to_str
    else
      raise TypeError, "no implicit conversion of #{letter.class} into String"
    end
    landing = @position - text.length
    landing = 0 if landing < 0
    # A position past the end leaves a gap, which Ruby fills with zero bytes
    # so the character lands where the position said.
    if landing > @string.length
      @string = @string + "\000" * (landing - @string.length)
    end
    @string = @string[0, landing] + text + (@string[landing + text.length..-1] || "")
    @position = landing
    nil
  end

  # Bytes put back land where the cursor stands, so a byte in the middle of a
  # character replaces that byte alone. What follows the cursor stays where
  # it was, which is why putting back more bytes than were read grows the
  # string.
  def ungetbyte(byte)
    return nil if byte.nil?
    listed = byte.is_a?(Integer) ? [byte & 0xff] : byte.to_s.bytes
    held = @string.bytes
    landing = @position - listed.size
    landing = 0 if landing < 0
    tail = held[@position..-1] || []
    named = @string.encoding
    @string = (held[0, landing] + listed + tail).pack("C*").force_encoding(named)
    @position = landing
    nil
  end

  def gets(separator = $/, limit = nil, chomp: false)
    separator, limit = StringIO.line_arguments separator, limit
    line = read_line separator, limit, chomp
    $_ = line
    line
  end

  # What a line reader's first two arguments stand for. A lone number in the
  # separator's place is a limit, and anything that reads as a String is a
  # separator.
  def self.line_arguments(separator, limit)
    if !separator.nil? && !separator.is_a?(String)
      if separator.respond_to? :to_str
        separator = separator.to_str
      else
        limit = separator
        separator = $/
      end
    end
    unless limit.nil?
      limit = whole_number limit
      # A negative limit is no limit at all.
      limit = nil if limit < 0
    end
    [separator, limit]
  end

  # One line, without touching `$_`, which is what every reader but `gets`
  # and `readline` does. The arguments arrive already read.
  def read_line(separator, limit, chomp)
    reading_allowed
    return "" if limit == 0
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
      if cut.nil?
        remaining
      else
        # A paragraph keeps every blank line that closes it.
        ending = cut + 1
        ending += 1 while ending < remaining.length && remaining[ending] == "\n"
        remaining[0, ending]
      end
    else
      cut = remaining.index(separator)
      cut.nil? ? remaining : remaining[0, cut + separator.length]
    end
    line = line[0, limit] unless limit.nil?
    return nil if line.empty?
    @position = @position + line.length
    @lineno = @lineno + 1
    if chomp
      return separator == "\n" || separator.nil? ? line.chomp : line.chomp(separator)
    end
    line
  end
  private :read_line

  def readline(separator = $/, limit = nil, chomp: false)
    line = self.gets(separator, limit, chomp: chomp)
    raise EOFError, "end of file reached" if line.nil?
    line
  end

  def each_line(separator = $/, limit = nil, chomp: false)
    reading_allowed
    separator, limit = StringIO.line_arguments separator, limit
    return each_line_enumerator(separator, limit, chomp) unless block_given?
    while (line = read_line(separator, limit, chomp))
      # A limit of zero reads nothing, so there is no next line to reach.
      break if line.empty?
      yield line
    end
    self
  end

  # Without a block the lines are handed over one at a time, already read the
  # way the arguments ask for.
  def each_line_enumerator(separator, limit, chomp)
    collected = []
    while (line = read_line(separator, limit, chomp))
      # A limit of zero reads nothing, so there is no next line to reach.
      break if line.empty?
      collected.push line
    end
    collected.each
  end
  private :each_line_enumerator

  def each(separator = $/, limit = nil, chomp: false, &block)
    each_line(separator, limit, chomp: chomp, &block)
  end

  def readlines(separator = $/, limit = nil, chomp: false)
    reading_allowed
    separator, limit = StringIO.line_arguments separator, limit
    raise ArgumentError, "invalid limit: 0 for readlines" if limit == 0
    collected = []
    while (line = read_line(separator, limit, chomp))
      # A limit of zero reads nothing, so there is no next line to reach.
      break if line.empty?
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
    if @misaligned
      raise ArgumentError, "invalid byte sequence in #{external_encoding.name}"
    end
    return sized_enum(:each_codepoint) unless block_given?
    while (letter = self.getc)
      yield letter.ord
    end
    self
  end

  # ── Writing ──────────────────────────────────────────────────────────────

  # Text written to a stream tagged with an encoding of its own is carried
  # into that encoding first. Bytes stay as they are, since there is nothing
  # to read them as.
  def __for_writing__(text)
    named = @encoding
    return text if named.nil?
    return text if text.encoding.name == "ASCII-8BIT"
    return text if named.to_s == text.encoding.name
    begin
      text.encode named
    rescue StandardError
      text
    end
  end

  def write(*values)
    writing_allowed
    written = 0
    values.each do |value|
      text = __for_writing__ value.to_s
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
      spread = value.is_a?(Array) ? value : StringIO.__as_array__(value)
      unless spread.nil?
        if walking.any? { |held| held.equal?(value) }
          self.write "[...]\n"
          next
        end
        walking.push(value)
        spread.empty? ? self.write("\n") : write_lines(spread, walking)
        walking.pop
        next
      end
      text = value.nil? ? "" : value.to_s
      # A `to_s` that hands back something other than a String says nothing
      # about the object, so the object describes itself.
      text = Object.instance_method(:to_s).bind(value).call unless text.is_a? String
      self.write text
      self.write "\n" unless text.end_with? "\n"
    end
  end
  private :write_lines

  # The Array an object stands for, or nil where it stands for none. An
  # object that answers for missing names is asked too, and one that refuses
  # the name stands for no Array.
  def self.__as_array__(value)
    return nil if value.nil? || value.is_a?(String)
    held = begin
      value.to_ary
    rescue NoMethodError
      nil
    end
    held.is_a?(Array) ? held : nil
  end

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

  # Reading in binary tags what comes back as bytes rather than as text.
  def binmode
    @binary = true
    self
  end

  def external_encoding
    return Encoding::BINARY if @binary
    return @encoding unless @encoding.nil?
    @string.encoding
  end

  def internal_encoding
    nil
  end

  # The encoding a stream reads its text as. The buffer keeps whatever it was
  # tagged with, so a frozen string is left alone.
  def set_encoding(external, internal = nil)
    @encoding = external.is_a?(String) ? Encoding.find(external) : external
    @binary = false
    # The buffer is tagged along with the stream unless it refuses to change.
    @string.force_encoding @encoding unless @string.frozen?
    self
  end

  # Read the byte-order mark the stream starts with, if any, and take the
  # encoding it names. The mark is consumed; anything else is left in place.
  def set_encoding_by_bom
    raise FrozenError, "can't modify frozen StringIO: #{inspect}" if frozen?
    return nil unless @readable
    source = @string.bytes
    found, width = StringIO.bom_encoding(source[@position, 4] || [])
    return nil if found.nil?
    @position = @position + width
    @encoding = found
    @binary = false
    found
  end

  # The encoding a leading byte-order mark names, with how many bytes it
  # takes. A mark that runs out part way names nothing.
  def self.bom_encoding(bytes)
    IO.bom_encoding(bytes)
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
    # Several values yielded together reach the block as one array, which is
    # what a walk over pairs hands over.
    each { |*values| collected.push(block.call(packed(values))) }
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
    Enumerator.over(self, :each, [], @lazy_size)
  end

  def to_enum(method_name = :each, *args)
    Enumerator.over(self, method_name, args, nil)
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

# A source with nothing in it, which `lazy.take(0)` walks.
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
    return given if given.is_a?(Integer)
    unless given.respond_to?(:to_int)
      raise TypeError, "no implicit conversion of #{given.class} into Integer"
    end
    read = given.to_int
    unless read.is_a?(Integer)
      raise TypeError, "can\'t convert #{given.class} to Integer (#{given.class}#to_int gives #{read.class})"
    end
    read
  end
  private :fill_count

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
# a local time follows the zone rules the operating system holds.
class Time
  include Comparable

  MICROSECONDS_IN_SECOND = 1000000
  NANOSECONDS_IN_SECOND = 1000000000
  MONTH_NAMES = %w[jan feb mar apr may jun jul aug sep oct nov dec]

  def self.now(**options)
    counted = Time.__now__
    made = from_exact(counted[0].to_r + Rational(counted[1], NANOSECONDS_IN_SECOND), false, nil)
    zone = options[:in]
    return made if zone.nil?
    return made.utc if names_utc?(zone)
    made.localtime zone
  end

  def self.new(*args, **options)
    return now(**options) if args.empty?
    zone = args.size > 6 ? args[6] : options[:in]
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
    return from_exact(calendar_seconds(fields, true), true, nil) if names_utc?(zone)
    offset = offset_seconds(zone)
    from_exact(calendar_seconds(fields, true) - offset, false, offset)
  end

  def self.at(seconds, extra = nil, **options)
    made = if seconds.is_a? Time
      from_exact(seconds.to_r, seconds.utc?, nil)
    elsif extra.nil?
      from_exact(seconds.to_r, false, nil)
    else
      from_exact(seconds.to_r + extra.to_r / MICROSECONDS_IN_SECOND, false, nil)
    end
    zone = options[:in]
    return made if zone.nil?
    return made.utc if names_utc?(zone)
    made.localtime zone
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
  # Whether a zone names UTC rather than an offset from it.
  def self.names_utc?(zone)
    return false if zone.nil? || zone.is_a?(Numeric)
    return false if zone.respond_to? :local_to_utc
    text = zone.to_s
    # A zone written as a negative zero offset is UTC itself rather than a
    # place that happens to sit on it.
    text == "UTC" || text == "Z" || text == "-00:00" || text == "-0000"
  end

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

  # The name the zone goes by, which is written in ASCII whatever the text
  # of the program is written in.
  def zone
    return @zone_object unless @zone_object.nil?
    return nil unless @offset.nil?
    return "UTC".force_encoding(Encoding::US_ASCII) if @utc
    named = calendar[10]
    named.nil? ? nil : named.force_encoding(Encoding::US_ASCII)
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
    return utc if Time.names_utc? offset
    @utc = false
    @offset = offset.nil? ? nil : Time.offset_seconds(offset)
    @calendar = nil
    self
  end

  def getlocal(offset = nil)
    return getutc if Time.names_utc? offset
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

end


class Proc
  # `self >> other` reads left to right: self is called first and hands its
  # answer to other. The composition is strict about its arguments when self
  # is, since self is the one the arguments reach.
  def >>(other)
    raise TypeError, "callable object is expected" unless other.respond_to?(:call)
    held = self
    if lambda?
      lambda { |*given, &block| other.call(held.call(*given, &block)) }
    else
      proc { |*given, &block| other.call(held.call(*given, &block)) }
    end
  end

  # `self << other` reads right to left: other is called first and hands its
  # answer to self, so the composition follows other's strictness.
  def <<(other)
    raise TypeError, "callable object is expected" unless other.respond_to?(:call)
    held = self
    strict = other.respond_to?(:lambda?) && other.lambda?
    if strict
      lambda { |*given, &block| held.call(other.call(*given, &block)) }
    else
      proc { |*given, &block| held.call(other.call(*given, &block)) }
    end
  end

  # A curried proc gathers arguments until it holds as many as the proc it
  # stands for takes, and answers another curried proc until then. A lambda
  # curries into lambdas, a plain proc into plain procs.
  def curry(count = nil)
    Proc.__curry_for__(self, count, lambda?)
  end

  # The count of arguments a curried callable waits for, which is how many it
  # requires. A strict callable refuses a count it could never be called with.
  def self.__curry_for__(callable, count, strict)
    required = 0
    optional = 0
    rest = false
    callable.parameters.each do |entry|
      case entry[0]
      when :req then required += 1
      when :opt then optional += 1
      when :rest then rest = true
      end
    end
    if count.nil?
      count = callable.arity
      count = -count - 1 if count < 0
    else
      count = count.to_int
      if strict && (count < required || (!rest && count > required + optional))
        raise ArgumentError, "wrong number of arguments (given #{count}, expected #{required})"
      end
    end
    __curried__(callable, count, [], strict)
  end

  def self.__curried__(callable, count, collected, strict)
    held = if strict
      lambda { |*given| Proc.__curry_step__(callable, count, collected, strict, given) }
    else
      proc { |*given| Proc.__curry_step__(callable, count, collected, strict, given) }
    end
    # A curried callable stands for the library's own, which names no
    # arguments of its own and has no scope behind it.
    held.instance_variable_set :@__curried, true
    held
  end

  def self.__curry_step__(callable, count, collected, strict, given)
    gathered = collected + given
    return callable.call(*gathered) if gathered.size >= count
    __curried__(callable, count, gathered, strict)
  end
end

class Random
  # The Mersenne Twister, which is the generator Ruby's own numbers come
  # from, so a seed gives the same sequence here as it does there.
  N = 624
  M = 397
  MATRIX_A = 0x9908b0df
  UPPER_MASK = 0x80000000
  LOWER_MASK = 0x7fffffff
  MASK32 = 0xffffffff

  def initialize seed = nil
    @seed = seed.nil? ? Random.new_seed : Random.seed_number(seed)
    @state = Array.new N, 0
    @index = N + 1
    seed_with Random.seed_words(@seed)
  end

  attr_reader :seed

  # A seed may be written as any number, and its whole part is what seeds.
  def self.seed_number held
    return held if held.is_a? Integer
    return held.to_i if held.is_a?(Float) || held.is_a?(Rational)
    if held.is_a? Complex
      unless held.imaginary == 0
        raise RangeError, "can't convert #{held} into Integer"
      end
      return held.real.to_i
    end
    unless held.respond_to? :to_int
      raise TypeError, "no implicit conversion of #{held.class} into Integer"
    end
    held.to_int
  end

  # The 32-bit words a seed is made of, smallest first.
  def self.seed_words held
    held = held.abs
    return [0] if held == 0
    words = []
    while held > 0
      words << (held & MASK32)
      held = held >> 32
    end
    words
  end

  def self.new_seed
    # A seed of its own is drawn from the operating system rather than from
    # the shared generator, so `srand` never decides it.
    bytes = Random.urandom 8
    bytes.each_char.each_with_index.inject(0) do |held, (character, at)|
      held | (character.ord << (at * 8))
    end
  end

  def self.urandom count
    count = count.to_int if !count.is_a?(Integer) && count.respond_to?(:to_int)
    unless count.is_a? Integer
      raise TypeError, "no implicit conversion of #{count.class} into Integer"
    end
    if count < 0
      raise ArgumentError, "negative string size (or size too big)"
    end
    held = File.open("/dev/urandom", "rb") { |source| source.read count }
    held.force_encoding Encoding::BINARY
  end

  def self.srand number = nil
    held = @last_seed.nil? ? Random.new_seed : @last_seed
    @last_seed = number.nil? ? Random.new_seed : Random.seed_number(number)
    @shared = Random.new @last_seed
    # The seedless `rand` draws from the same seed, so seeding here settles
    # both what Random answers and what Kernel does.
    Kernel.srand @last_seed
    held
  end

  def self.shared
    @shared = Random.new if @shared.nil?
    @shared
  end

  def self.rand limit = nil
    shared.rand limit
  end

  def self.random_number limit = nil
    shared.random_number limit
  end

  def self.bytes count
    shared.bytes count
  end

  # The words the state holds, which two generators share when they were
  # seeded the same way.
  def state
    @seed
  end
  private :state

  def == other
    other.is_a?(Random) && other.seed == @seed
  end

  # A run of bytes, four to each word the generator answers.
  def bytes count
    held = []
    taken = 0
    while taken < count
      word = next_word
      4.times do
        break if taken >= count
        held.push((word >> ((taken % 4) * 8)) & 0xff)
        taken += 1
      end
    end
    held.pack "C*"
  end

  # A Float in [0, 1) when nothing bounds it, and a number under the bound
  # otherwise.
  def rand limit = nil
    return next_real if limit.nil?
    return rand_in_range limit if limit.is_a? Range
    if limit.is_a? Float
      raise ArgumentError, "invalid argument - #{limit}" unless limit > 0
      return next_real * limit
    end
    held = Random.seed_number limit
    raise ArgumentError, "invalid argument - #{limit}" unless held > 0
    (next_real * held).floor
  end

  def random_number limit = nil
    limit.nil? || limit == 0 ? next_real : rand(limit)
  end

  # A number drawn from a Range, which Ruby settles by the width between the
  # two ends rather than by what the ends themselves are. A width that counts
  # gives a whole number, and one that measures gives a Float.
  def rand_in_range span
    first = span.begin
    last = span.end
    if first.nil? || last.nil?
      raise ArgumentError, "cannot get the random number from an endless range"
    end
    width = begin
      last - first
    rescue StandardError
      raise ArgumentError, "bad value for range"
    end
    return first + next_real * width.to_f if width.is_a?(Numeric) && !width.is_a?(Integer)
    unless width.is_a?(Integer) || width.respond_to?(:to_int)
      raise ArgumentError, "bad value for range"
    end
    width = width.to_int unless width.is_a? Integer
    count = span.exclude_end? ? width : width + 1
    raise ArgumentError, "invalid argument - #{span}" unless count > 0
    first + rand(count)
  end
  private :rand_in_range

  # A seed of one word is spread by multiplying it out, and a wider one is
  # mixed in word by word, which is the split Ruby makes as well.
  def seed_with words
    if words.length <= 1
      seed_one words[0]
      return
    end
    words = words[0, words.length - 1] if words[words.length - 1] == 1
    if words.length <= 1
      seed_one words[0]
      return
    end
    seed_one 19650218
    at = 1
    from = 0
    count = N > words.length ? N : words.length
    while count > 0
      previous = @state[at - 1]
      @state[at] = ((@state[at] ^ ((previous ^ (previous >> 30)) * 1664525)) +
                    words[from] + from) & MASK32
      at += 1
      from += 1
      if at >= N
        @state[0] = @state[N - 1]
        at = 1
      end
      from = 0 if from >= words.length
      count -= 1
    end
    count = N - 1
    while count > 0
      previous = @state[at - 1]
      @state[at] = ((@state[at] ^ ((previous ^ (previous >> 30)) * 1566083941)) - at) & MASK32
      at += 1
      if at >= N
        @state[0] = @state[N - 1]
        at = 1
      end
      count -= 1
    end
    @state[0] = UPPER_MASK
    @index = N
  end
  private :seed_with

  def seed_one number
    @state[0] = number & MASK32
    at = 1
    while at < N
      previous = @state[at - 1]
      @state[at] = ((previous ^ (previous >> 30)) * 1812433253 + at) & MASK32
      at += 1
    end
    @index = N
  end
  private :seed_one

  def twist
    at = 0
    while at < N
      held = (@state[at] & UPPER_MASK) | (@state[(at + 1) % N] & LOWER_MASK)
      mixed = @state[(at + M) % N] ^ (held >> 1)
      mixed = mixed ^ MATRIX_A if held.odd?
      @state[at] = mixed & MASK32
      at += 1
    end
    @index = 0
  end
  private :twist

  def next_word
    twist if @index >= N
    held = @state[@index]
    @index += 1
    held = held ^ (held >> 11)
    held = held ^ ((held << 7) & 0x9d2c5680)
    held = held ^ ((held << 15) & 0xefc60000)
    (held ^ (held >> 18)) & MASK32
  end
  private :next_word

  # Ruby draws a Float from two words, keeping the 53 bits a double holds.
  def next_real
    high = next_word >> 5
    low = next_word >> 6
    (high * 67108864.0 + low) / 9007199254740992.0
  end
  private :next_real
end

module Process
  # The same ids Process itself answers, gathered under the words Ruby
  # gathers them under.
  module GID
    def self.rid
      Process.gid
    end

    def self.eid
      Process.egid
    end

    def self.eid= wanted
      Process.egid = wanted
    end

    def self.change_privilege wanted
      Process.gid = wanted
      wanted
    end
  end

  module UID
    def self.rid
      Process.uid
    end

    def self.eid
      Process.euid
    end

    def self.eid= wanted
      Process.euid = wanted
    end

    def self.change_privilege wanted
      Process.uid = wanted
      wanted
    end
  end

  module Sys
    def self.getgid
      Process.gid
    end

    def self.getuid
      Process.uid
    end

    def self.getegid
      Process.egid
    end

    def self.geteuid
      Process.euid
    end

    def self.setgid wanted
      Process.gid = wanted
    end

    def self.setuid wanted
      Process.uid = wanted
    end

    def self.setegid wanted
      Process.egid = wanted
    end

    def self.seteuid wanted
      Process.euid = wanted
    end
  end
end

class IO
  SEEK_SET = 0
  SEEK_CUR = 1
  SEEK_END = 2
  # The name of the stream that keeps nothing it is given and reads back as
  # empty, which every system carries under this name.
  NULL = "/dev/null"

  # Which of the streams handed in have something to read, or room to write,
  # right now. Nothing is ready answers nil, which is what a caller waits on.
  def self.select(readers = nil, writers = nil, errored = nil, timeout = nil)
    ready_readers = (readers || []).select { |held| IO.__ready__ held, false }
    ready_writers = (writers || []).select { |held| IO.__ready__ held, true }
    if ready_readers.empty? && ready_writers.empty?
      # Metorex runs one thing at a time, so a stream nothing is left to
      # write to stays unready however long the caller waits.
      return nil unless timeout.nil?
      return nil
    end
    [ready_readers, ready_writers, errored || []]
  end

  def self.__ready__(held, writing)
    stream = held.respond_to?(:to_io) ? held.to_io : held
    return true unless stream.is_a? IO
    return true if stream.__stream_handle__.nil?
    IO.__stream__ "ready?", stream.__stream_handle__, "", writing ? 1 : 0
  rescue IOError
    false
  end

  # Two joined streams: what is written to the second is read from the first.
  # With a block the pair is handed over and closed once the block is done.
  def self.pipe(_external = nil, _internal = nil, **_options)
    reading, writing = IO.__stream__ "pipe", 0, "", 0
    pair = [IO.__over__(reading, nil, "r"), IO.__over__(writing, nil, "w")]
    return pair unless block_given?
    begin
      yield pair[0], pair[1]
    ensure
      pair.each { |held| held.close unless held.closed? }
    end
  end

  # An IO over a handle the interpreter already holds.
  def self.__over__(handle, path = nil, mode = nil)
    held = allocate
    held.__send__ :__take__, handle, path, mode
    held
  end

  # The whole of a file, or a run of it, named by path. `File` reads these
  # itself, and an IO reads them the same way.
  def self.read(name, *rest, **options)
    File.read name, *rest, **options
  end

  def self.binread(name, *rest)
    File.binread name, *rest
  end

  # Text written to a file named by path. Without an offset the file is
  # written from the start and cut down to what was written, and with one the
  # rest of the file is left as it was.
  def self.write(name, text, offset = :__none__, *extra, **options)
    unless extra.empty?
      raise ArgumentError,
            "wrong number of arguments (given #{3 + extra.size}, expected 2..3)"
    end
    spelled = text.is_a?(String) ? text : text.to_s
    at = offset == :__none__ ? nil : offset
    named = options[:mode]
    mode = if !named.nil?
      named
    elsif at.nil?
      "w"
    else
      "r+"
    end
    held = begin
      File.open File.path(name), mode
    rescue Errno::ENOENT
      # A write brings the file into being, whatever the mode says about
      # reading it.
      File.open File.path(name), "w"
    end
    begin
      held.seek at, IO::SEEK_SET unless at.nil?
      held.write spelled
    ensure
      held.close
    end
  end

  # The bytes a String stands for written to a file, with nothing carried
  # into another encoding on the way.
  def self.binwrite(name, text, offset = :__none__, *extra, **options)
    IO.write name, text, offset, *extra, **options
  end

  def self.readlines(name, separator = $/, limit = nil, chomp: false, **options)
    held = File.open File.path(name), options[:mode].nil? ? "r" : options[:mode]
    begin
      held.readlines separator, limit, chomp: chomp
    ensure
      held.close
    end
  end

  # Each line of a file in turn. Without a block the lines are handed back as
  # a walk over them.
  def self.foreach(name, separator = :__none__, limit = nil, chomp: false, **options, &block)
    if block.nil?
      return Enumerator.new do |yielder|
        IO.foreach(name, separator, limit, chomp: chomp, **options) { |line| yielder << line }
      end
    end
    held = File.open File.path(name), options[:mode].nil? ? "r" : options[:mode]
    # Reading every line of a file leaves no last line read behind.
    $_ = nil
    begin
      if separator == :__none__
        held.each_line(chomp: chomp) { |line| block.call line }
      else
        held.each_line(separator, limit, chomp: chomp) { |line| block.call line }
      end
    ensure
      held.close
    end
    nil
  end

  # The stream an object stands for, or nil where it stands for none. Only an
  # object answering `to_io` is asked.
  def self.try_convert(held)
    # `IO === held` asks IO rather than the object, so an object that answers
    # nothing of Kernel's, a BasicObject among them, is read here too.
    return held if IO === held
    converted = begin
      held.to_io
    rescue NoMethodError => missing
      raise unless missing.name == :to_io
      return nil
    end
    return converted if IO === converted
    raise TypeError,
          "can't convert #{held.class} into IO (#{held.class}#to_io gives #{converted.class})"
  end

  # An IO over a descriptor, handed to a block when one is given and closed
  # once the block is done.
  def self.open(number, mode = nil, *extra, **options)
    held = new number, mode, *extra, **options
    return held unless block_given?
    begin
      yield held
    ensure
      begin
        held.close unless held.closed?
      rescue IOError => closing
        # A stream already closed inside the block is not a failure of the
        # close here, so that one error alone is let go.
        raise unless closing.message == "closed stream"
      end
    end
  end

  # The descriptor a name opens under, handed back by number. Whoever asked
  # for it owns it, so nothing here closes it.
  def self.sysopen(name, mode = nil, permissions = nil)
    written = IO.__written_mode__(mode).to_s
    written = "r" if written.empty?
    IO.__stream__ "sysopen", 0, File.path(name), IO.__opening_number__(written)
  end

  # How a mode string says the file is opened: 0 reads, 1 writes from the
  # start, 2 adds to the end, and 3 does both.
  def self.__opening_number__(written)
    return 2 if written.start_with? "a"
    return 3 if written.start_with?("r") && written.include?("+")
    return 1 if written.start_with? "w"
    0
  end

  # A mode as a String, whatever it was written as: a String stands as it is,
  # a number names the flags, and anything else spells itself as one.
  def self.__written_mode__(mode)
    return nil if mode.nil?
    return mode if mode.is_a? String
    return IO.__mode_of_flags__(mode) if mode.is_a? Integer
    return mode.to_str if mode.respond_to? :to_str
    return IO.__mode_of_flags__(mode.to_int) if mode.respond_to? :to_int
    raise ArgumentError, "invalid access mode #{mode}"
  end

  # The mode string the open flags stand for.
  def self.__mode_of_flags__(flags)
    access = flags & 3
    adding = flags & File::APPEND != 0
    return adding ? "a+" : "r+" if access == File::RDWR
    return adding ? "a" : "w" if access == File::WRONLY
    "r"
  end

  # `IO.new(fd)` stands over a descriptor this program did not open. The mode
  # says how the program means to use it, which must be a use the descriptor
  # was opened for, and the options name the encodings and whether the
  # descriptor is closed along with the IO.
  def initialize(number, mode = nil, *extra, **options)
    unless extra.empty?
      raise ArgumentError,
            "wrong number of arguments (given #{2 + extra.size}, expected 1..2)"
    end
    if block_given?
      warn "warning: IO::new() does not take block; use IO::open() instead"
    end
    number = IO.__as_descriptor__ number
    written = IO.__mode_wanted__ mode, options
    handle = IO.__stream__ "adopt", 0, "", number
    opened = IO.__stream__ "accmode", handle, "", 0
    if written.nil?
      # Nothing said how the stream would be used, so it is used the way the
      # descriptor was opened.
      written = IO.__mode_of_flags__ opened
    else
      # A descriptor opened for reading cannot be written through, and one
      # opened for writing cannot be read.
      IO.__check_access__ written, opened
    end
    __take__ handle, options[:path], written
    self.autoclose = options.key?(:autoclose) ? (options[:autoclose] ? true : false) : true
    @binmode = true if IO.__binary_mode__ written, options
    @__file_encoding = IO.__named_encoding__ written, options
    self
  end

  # The descriptor a program named, which is a number or something that
  # spells itself as one.
  def self.__as_descriptor__(number)
    return number if number.is_a? Integer
    unless number.respond_to? :to_int
      raise TypeError, "no implicit conversion of #{number.class} into Integer"
    end
    held = number.to_int
    unless held.is_a? Integer
      raise TypeError, "can't convert #{number.class} to Integer"
    end
    held
  end

  # The mode a program asked for, named either as an argument or as a `mode:`
  # option, but never as both.
  def self.__mode_wanted__(mode, options)
    named = options[:mode]
    if !mode.nil? && !named.nil?
      raise ArgumentError, "mode specified twice"
    end
    given = named.nil? ? mode : named
    return nil if given.nil?
    written = IO.__written_mode__(given).to_s
    raise ArgumentError, "invalid access mode #{written}" if written.empty?
    written
  end

  # Whether the stream reads and writes bytes rather than text. A mode
  # naming binary or text and an option saying so are two ways of asking for
  # the same thing, and Ruby takes only one of them.
  def self.__binary_mode__(written, options)
    access = written.split(":", 2)[0].to_s
    named = access.include?("b") || access.include?("t")
    if named && (options.key?(:binmode) || options.key?(:textmode))
      raise ArgumentError, "binmode specified twice"
    end
    if options[:binmode] && options[:textmode]
      raise ArgumentError, "both textmode and binmode specified"
    end
    return true if access.include? "b"
    options[:binmode] ? true : false
  end

  # The encodings a stream reads and writes in, written as `"ext:int"`. They
  # may be named after the mode or as options, and never as both.
  def self.__named_encoding__(written, options)
    parts = written.split(":")
    named = parts[1..].to_a.join(":")
    keyed = [:encoding, :external_encoding, :internal_encoding].any? { |key| options.key? key }
    if !named.empty? && keyed
      raise ArgumentError, "encoding specified twice"
    end
    unless named.empty?
      return IO.__encoding_pair__ parts[1], parts[2], written
    end
    outer = options[:external_encoding]
    inner = options[:internal_encoding]
    if options.key?(:encoding)
      if outer.nil? && inner.nil?
        outer, inner = IO.__spelled_encoding__(options[:encoding]).split(":", 2)
      else
        warn "warning: Ignoring encoding parameter '#{options[:encoding]}': #{outer.nil? ? "internal" : "external"}_encoding is used"
      end
    end
    if outer.nil? && inner.nil?
      return "ASCII-8BIT" if IO.__binary_mode__ written, options
      return nil
    end
    IO.__encoding_pair__ outer, inner, written
  end

  # Two encodings as one `"ext:int"` string. An internal encoding matching
  # the external one, or named `-`, is no internal encoding at all.
  def self.__encoding_pair__(outer, inner, written)
    outer = outer.nil? ? "" : IO.__spelled_encoding__(outer)
    inner = inner.nil? ? "" : IO.__spelled_encoding__(inner)
    inner = "" if inner == "-" || inner.downcase == outer.downcase
    inner.empty? ? outer : "#{outer}:#{inner}"
  end

  # An encoding as the name it is held under, whatever it was written as.
  def self.__spelled_encoding__(named)
    return named.name if named.is_a? Encoding
    return named if named.is_a? String
    return named.to_str if named.respond_to? :to_str
    named.to_s
  end

  # Whether the descriptor may be used the way the mode says.
  def self.__check_access__(written, opened)
    access = written.split(":", 2)[0].to_s
    reading = access.start_with?("r") || access.include?("+")
    writing = access.start_with?("w") || access.start_with?("a") || access.include?("+")
    if (reading && opened == File::WRONLY) || (writing && opened == File::RDONLY)
      raise Errno::EINVAL, "invalid access mode #{written}"
    end
    nil
  end

  def self.for_fd(number, mode = nil, **options)
    new number, mode, **options
  end

  # Everything one stream holds, written to another. A name stands for a file
  # opened for the copy and closed after it, and an object that reads or
  # writes is used as it is.
  def self.copy_stream(source, destination, length = nil, offset = nil)
    opened_source = nil
    opened_target = nil
    begin
      reader = source.respond_to?(:read) ? source : (opened_source = File.open(File.path(source), "rb"))
      writer = destination.respond_to?(:write) ? destination : (opened_target = File.open(File.path(destination), "wb"))
      held = reader.read.to_s
      held = held.byteslice(offset, held.bytesize).to_s unless offset.nil?
      held = held.byteslice(0, length).to_s unless length.nil?
      writer.write held
      held.bytesize
    ensure
      opened_source.close unless opened_source.nil?
      opened_target.close unless opened_target.nil?
    end
  end

  # The three streams the program started with, each over the descriptor the
  # operating system opened for it.
  def self.__standard__(number, named)
    held = __over__ IO.__stream__("adopt", 0, "", number), named, number == 0 ? "r" : "w"
    held.__send__ :__name_standard__, named
    held
  end

  def __name_standard__(named)
    @standard = named
    # Ruby writes the error stream straight through rather than holding what
    # is written back, which is what `sync` reports for it.
    @sync = true if named == "stderr"
    self
  end

  def __take__(handle, path = nil, mode = nil)
    __note_encodings__
    @handle = handle
    @path = path
    @__file_mode = mode
    @closed = false
    @autoclose = true
    @lineno = 0
    @sync = false
    self
  end


  def __stream_handle__
    @handle
  end

  # The number the operating system holds this stream under.
  def fileno
    raise IOError, "closed stream" if closed?
    IO.__stream__ "fileno", __stream_handle__, "", 0
  end

  alias_method :to_i, :fileno

  # The file this stream was opened over, where it was opened over one.
  def path
    @path
  end

  # A stream not reading from a child process has no process to name.
  def pid
    raise IOError, "closed stream" if closed?
    nil
  end

  def closed?
    @closed == true
  end

  # A stream closes the descriptor it holds unless it was told not to, which
  # only a stream built over a descriptor from outside is.
  def autoclose?
    raise IOError, "closed stream" if closed?
    @autoclose.nil? ? true : @autoclose
  end

  def autoclose=(wanted)
    raise IOError, "closed stream" if closed?
    @autoclose = wanted ? true : false
  end

  def close
    return nil if closed?
    # What the stream was holding back is written before the descriptor goes,
    # so a broken pipe is reported here rather than lost.
    begin
      __drain__
    ensure
      IO.__stream__ "close", __stream_handle__, "", 0 if autoclose?
      @closed = true
    end
    nil
  end

  # A stream with two ends may have one of them closed on its own. A stream
  # with a single end refuses, which is what Ruby does for a file.
  def __duplex__
    !@__popen_input.nil?
  end

  # Whether this stream was opened only for the side being closed, in which
  # case closing that side closes the stream itself.
  def __opened_for__(letter)
    @__file_mode.to_s.start_with? letter
  end

  # Whether the stream was opened for both sides, which is what a mode
  # carrying a plus says.
  def __both_ways__
    @__file_mode.to_s.include? "+"
  end

  # Whether reading is allowed at all. A mode naming only writing or only
  # appending leaves no reading side.
  def __readable__
    return false if @standard == "stdout" || @standard == "stderr"
    return true if @__file_mode.nil?
    return true if __both_ways__
    !(__opened_for__("w") || __opened_for__("a"))
  end

  def close_read
    return nil if @read_closed
    unless __duplex__
      # A stream that was never writable has only the one side, so closing
      # the reading side closes the stream.
      return close if !__both_ways__ && __opened_for__("r")
      raise IOError, "closing non-duplex IO for reading"
    end
    @read_closed = true
    nil
  end

  def close_write
    return nil if @write_closed
    unless __duplex__
      return close if !__both_ways__ && (__opened_for__("w") || __opened_for__("a"))
      raise IOError, "closing non-duplex IO for writing"
    end
    @write_closed = true
    nil
  end

  def write(*parts)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for writing" if @write_closed
    raise IOError, "not opened for writing" unless __writable__
    @line_buffered = nil
    @wrote_through_buffer = true
    held = parts.length == 1 ? parts[0].to_s : parts.map { |part| part.to_s }.join
    # A stream the program told not to sync holds what is written until it is
    # flushed, which is when the descriptor hears about it.
    if @holding && @standard.nil?
      @pending = @pending.nil? ? held : @pending + held
      return held.bytesize
    end
    # The streams the program started with are written through the
    # interpreter's own writer, so what a program prints keeps the order it
    # printed it in whichever route it took.
    return IO.__stream__("write", __stream_handle__, held, 0) if @standard.nil?
    IO.__write_standard__ @standard, held
    held.bytesize
  end

  # Everything held back by a stream that does not sync, written through now.
  def __drain__
    return nil if @pending.nil? || @pending.empty?
    # What could not be written stays held, so the next flush or the close
    # reports the same trouble rather than losing it.
    IO.__stream__ "write", __stream_handle__, @pending, 0
    @pending = nil
    nil
  end
  private :__drain__

  def <<(text)
    write text
    self
  end

  # Each value written out as text, separated by `$,` where the program set
  # one, and followed by `$\`. With nothing to write the last line read is
  # written instead.
  def print(*parts)
    parts = [$_] if parts.empty?
    separator = $,
    parts.each_with_index do |part, index|
      write separator.to_s if index > 0 && !separator.nil?
      write(part.nil? ? "" : part.to_s)
    end
    write $\ unless $\.nil?
    nil
  end

  def printf(format, *rest)
    # A format may be written as anything that spells itself out.
    spelled = format.is_a?(String) ? format : format.to_str
    write spelled % rest
    nil
  end

  def puts(*lines)
    return write(__line_ending__) && nil if lines.empty?
    __write_lines__ lines, []
    nil
  end

  # What ends a line this stream writes. A stream opened with `newline:` ends
  # them the way that named, and every other stream ends them with a newline.
  def __line_ending__
    return "\r\n" if @__file_newline == :crlf
    return "\r" if @__file_newline == :cr
    "\n"
  end
  private :__line_ending__

  # One line per value, where an array is written out element by element. An
  # array that reaches itself is written as `[...]` rather than followed.
  def __write_lines__(values, walking)
    ending = __line_ending__
    values.each do |value|
      spread = value.is_a?(Array) ? value : __as_array__(value)
      if spread.nil?
        held = value.nil? ? "" : value.to_s
        # A `to_s` that hands back something other than a String says
        # nothing about the object, so the object describes itself.
        held = Object.instance_method(:to_s).bind(value).call unless held.is_a? String
        write(held.end_with?(ending) ? held : held + ending)
      elsif walking.any? { |seen| seen.equal? value }
        write "[...]#{ending}"
      else
        __write_lines__ spread, walking + [value]
      end
    end
  end
  private :__write_lines__

  # The Array an object stands for, or nil where it stands for none. An
  # object that answers for missing names is asked too, and one that refuses
  # the name stands for no Array.
  def __as_array__(value)
    return nil if value.nil? || value.is_a?(String)
    held = begin
      value.to_ary
    rescue NoMethodError
      nil
    end
    held.is_a?(Array) ? held : nil
  end
  private :__as_array__

  # As many bytes as asked for, or everything left where no count is given.
  # Whatever was put back with `ungetc` stands before what the descriptor
  # has, and is handed out first.
  def read(length = nil, buffer = nil)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" if @read_closed
    wanted = length.nil? ? 0 : __as_integer__(length)
    raise ArgumentError, "negative length #{wanted} given" if wanted < 0
    waiting = @peeked
    @peeked = nil
    waiting = "" if waiting.nil?
    held = if wanted == 0
      waiting + IO.__stream__("read", __stream_handle__, "", 0).to_s
    elsif waiting.bytesize > wanted
      taken = waiting[0, wanted]
      rest = waiting[wanted, waiting.length - wanted]
      @peeked = rest.nil? || rest.empty? ? nil : rest
      taken
    elsif waiting.bytesize == wanted
      waiting
    else
      # What was put back is already in hand, so a descriptor with nothing
      # waiting behind it does not make the read fail.
      more = begin
        IO.__stream__("read", __stream_handle__, "", wanted - waiting.bytesize).to_s
      rescue Errno::EAGAIN
        raise if waiting.empty?
        ""
      end
      waiting + more
    end
    return __fill_buffer__(buffer.nil? ? nil : __as_buffer__(buffer), held) unless buffer.nil?
    return nil if length && held.empty?
    held
  end

  # As much as is there right now, up to the count asked for. Nothing left
  # at all is the end of the stream.
  def readpartial(length = nil, buffer = nil)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    wanted = length.nil? ? 0 : __as_integer__(length)
    raise ArgumentError, "negative length #{wanted} given" if wanted < 0
    target = buffer.nil? ? nil : __as_buffer__(buffer)
    return __fill_buffer__(target, "") if wanted == 0
    held = read wanted
    if held.nil? || held.empty?
      __fill_buffer__ target, ""
      raise EOFError, "end of file reached"
    end
    __fill_buffer__ target, held
  end

  # A character put back, which the next read hands out before anything the
  # descriptor has.
  def ungetc(held)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    raise TypeError, "no implicit conversion of nil into String" if held.nil?
    text = if held.is_a? Integer
      # A codepoint stands for the character the stream reads text as.
      named = external_encoding
      held.chr(named.nil? ? Encoding::UTF_8 : named)
    elsif held.is_a? String
      held
    elsif held.respond_to? :to_str
      held.to_str
    else
      raise TypeError, "no implicit conversion of #{held.class} into String"
    end
    @peeked = @peeked.nil? ? text : text + @peeked
    # A stream holding text the descriptor already handed over cannot be
    # read around, which is what `sysread` would do.
    @line_buffered = true
    nil
  end

  # As many bytes as asked for, read straight from the descriptor. A stream
  # whose lines have been read holds text the descriptor has already handed
  # over, so reading around that is refused.
  def sysread(length = nil, buffer = nil)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    raise IOError, "sysread for buffered IO" if @line_buffered
    wanted = length.nil? ? 0 : __as_integer__(length)
    raise ArgumentError, "negative length #{wanted} given" if wanted < 0
    target = buffer.nil? ? nil : __as_buffer__(buffer)
    return target.nil? ? "" : target if wanted == 0
    held = IO.__stream__ "read", __stream_handle__, "", wanted
    if held.nil? || held.empty?
      __fill_buffer__ target, ""
      raise EOFError, "end of file reached"
    end
    return __fill_buffer__(target, held) unless target.nil?
    held
  end

  # The String a program handed over to be read into.
  def __as_buffer__(buffer)
    return buffer if buffer.is_a? String
    unless buffer.respond_to? :to_str
      raise TypeError, "no implicit conversion of #{buffer.class} into String"
    end
    buffer.to_str
  end
  private :__as_buffer__

  # What was read, written into the buffer the program handed over. The
  # buffer keeps the encoding it was tagged with.
  def __fill_buffer__(target, held)
    return held if target.nil?
    was = target.encoding
    target.replace held
    target.force_encoding was
    target
  end
  private :__fill_buffer__

  # Where the stream stands, counted in bytes from the start. A byte put back
  # with `ungetc` stands before that place.
  def pos
    raise IOError, "closed stream" if closed?
    __drain__
    standing = IO.__stream__ "seek", __stream_handle__, "cur", 0
    standing - (@peeked.nil? ? 0 : @peeked.bytesize)
  end

  alias_method :tell, :pos

  def pos=(offset)
    seek offset, IO::SEEK_SET
    offset
  end

  # An offset the operating system can hold. It counts bytes in a file, and
  # no file is longer than a machine word counts.
  def __as_offset__(offset)
    held = __as_integer__ offset
    if held.bit_length > 62
      raise RangeError, "bignum too big to convert into 'long'"
    end
    held
  end
  private :__as_offset__

  # The stream moved to another place: from the start, from where it stands,
  # or back from the end.
  def seek(offset, whence = IO::SEEK_SET)
    raise IOError, "closed stream" if closed?
    __drain__
    @peeked = nil
    @line_buffered = nil
    named = case whence
    when IO::SEEK_CUR, :CUR then "cur"
    when IO::SEEK_END, :END then "end"
    else "set"
    end
    IO.__stream__ "seek", __stream_handle__, named, __as_offset__(offset)
    0
  end

  # The descriptor moved straight, without the buffer a read fills. A stream
  # whose lines have been read holds text the descriptor already handed over,
  # so moving around that is refused.
  def sysseek(offset, whence = IO::SEEK_SET)
    raise IOError, "sysseek for buffered IO" if @line_buffered
    held = __as_offset__ offset
    seek held, whence
    IO.__stream__ "seek", __stream_handle__, "cur", 0
  end

  # Back to the start, with the line count starting over.
  def rewind
    seek 0, IO::SEEK_SET
    @lineno = 0
    0
  end

  # How many bytes the stream stands over.
  def size
    raise IOError, "closed stream" if closed?
    __drain__
    IO.__stream__ "size", __stream_handle__, "", 0
  end

  # The file cut down to the count of bytes, or filled out to it.
  def truncate(length)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for writing" unless __writable__
    __drain__
    IO.__stream__ "truncate", __stream_handle__, "", __as_integer__(length)
  end

  # As much as is there right now. A stream with nothing waiting says so
  # rather than holding the program up, either by raising or, when asked not
  # to, by answering what it would have waited for.
  def read_nonblock(length, buffer = nil, exception: true)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    wanted = __as_integer__ length
    raise ArgumentError, "negative length #{wanted} given" if wanted < 0
    target = buffer.nil? ? nil : __as_buffer__(buffer)
    return __fill_buffer__(target, "") if wanted == 0
    waiting = @peeked.nil? ? "" : @peeked
    unless waiting.empty?
      # A stream carrying text cannot be read a byte at a time around what
      # was put back, which is what Ruby refuses here.
      if @__newline_conversion
        raise IOError, "byte oriented read for character buffered IO"
      end
      return __fill_buffer__(target, read(wanted))
    end
    unless IO.__stream__("ready?", __stream_handle__, "", 0)
      return :wait_readable unless exception
      raise IO::EAGAINWaitReadable, "Resource temporarily unavailable - read would block"
    end
    held = read wanted
    if held.nil? || held.empty?
      __fill_buffer__ target, ""
      return nil unless exception
      raise EOFError, "end of file reached"
    end
    __fill_buffer__ target, held
  end

  # Written straight to the descriptor. A stream that has written through its
  # own buffer says so, since the two writes may land out of order.
  def syswrite(text)
    if @wrote_through_buffer
      warn "warning: syswrite for buffered IO"
    end
    held = write text
    @wrote_through_buffer = nil
    held
  end

  # As much as the descriptor will take right now. A descriptor with no room
  # says so rather than holding the program up.
  def write_nonblock(text, exception: true)
    write text
  rescue Errno::EAGAIN
    raise IO::EAGAINWaitWritable, "Resource temporarily unavailable - write would block" if exception
    :wait_writable
  end

  # The next line, up to the separator or the limit, whichever comes first.
  # The line read is what `$_` and `$.` report on.
  def gets(separator = $/, limit = nil, *extra, chomp: false)
    unless extra.empty?
      raise ArgumentError,
            "wrong number of arguments (given #{2 + extra.size}, expected 0..2)"
    end
    # `$_` names the line `gets` read. Walking the lines does not set it.
    $_ = __read_line__ separator, limit, chomp
  end

  # The next line, up to the separator or the limit, whichever comes first.
  # An empty separator reads a paragraph: the lines up to a blank one, with
  # the blank lines before it left behind.
  def __read_line__(separator, limit, chomp)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    separator, limit = StringIO.line_arguments separator, limit
    unless limit.nil?
      limit = __as_integer__ limit
      if limit.bit_length > 62
        raise RangeError, "bignum too big to convert into 'long'"
      end
      return "" if limit == 0
    end
    ending = separator.nil? ? nil : separator.to_s
    # An empty separator reads a paragraph, which ends at a blank line and
    # starts after the blank lines the paragraph before it ended with.
    paragraph = !ending.nil? && ending.empty?
    ending = "\n\n" if paragraph
    # A byte or a character put back with `ungetc` stands before whatever the
    # stream has left, and may already hold the separator.
    waiting = @peeked
    @peeked = nil
    collected = waiting.nil? ? "" : waiting
    unless collected.empty?
      if !ending.nil? && !ending.empty?
        at = collected.index ending
        unless at.nil?
          ends = at + ending.length
          rest = collected[ends, collected.length - ends]
          @peeked = rest.nil? || rest.empty? ? nil : rest
          collected = collected[0, ends]
          return __finish_line__ collected, ending, chomp
        end
      end
      if !limit.nil? && collected.bytesize >= limit
        rest = collected[limit, collected.length - limit]
        @peeked = rest.nil? || rest.empty? ? nil : rest
        return __finish_line__ collected[0, limit], ending, chomp
      end
    end
    wanted = limit.nil? ? -1 : limit - collected.bytesize
    held = IO.__stream__(
      "readline",
      __stream_handle__,
      ending.nil? ? "" : ending,
      wanted,
      paragraph && collected.empty? ? 1 : 0
    )
    collected = collected + held.to_s
    return nil if collected.empty?
    @line_buffered = true
    return __finish_line__ collected, ending, chomp
  end
  private :__read_line__

  # A line read, counted and tagged the way the stream reads text, with the
  # separator taken off where the reader asked for that.
  def __finish_line__(collected, ending, chomp)
    @lineno = (@lineno.nil? ? 0 : @lineno) + 1
    $. = @lineno
    held = __tag_read__ collected
    if chomp && !ending.nil? && !ending.empty? && held.end_with?(ending)
      held = held[0, held.length - ending.length]
    end
    held
  end
  private :__finish_line__

  def readline(separator = $/, limit = nil, *extra, chomp: false)
    line = self.gets separator, limit, *extra, chomp: chomp
    raise EOFError, "end of file reached" if line.nil?
    line
  end

  # The encodings named alongside the mode, as `"r:UTF-8:ISO-8859-1"` or as
  # an `encoding:` keyword. The first is what the stream is read as and the
  # second what its text is carried into.
  def __named_encodings__
    written = @__file_encoding.to_s
    if written.empty?
      # A stream told to follow the program's own encodings names none of
      # its own any more, whatever its mode was opened with.
      return [] if @__encodings_reset
      written = @__file_mode.to_s.split(":", 2)[1].to_s
    end
    written.split(":")
  end
  private :__named_encodings__

  # The encodings the program was reading and writing text in when the stream
  # was opened. A stream keeps them, so a later change to the program's own
  # encodings leaves an already open stream alone.
  def __note_encodings__
    return self if @__noted_encodings
    @__noted_encodings = true
    @__made_external = Encoding.default_external
    @__made_internal = Encoding.default_internal
    self
  end

  # Whether the stream was opened to write, which decides what it reports
  # when no encoding was named for it.
  def __writing_mode__
    access = @__file_mode.to_s.split(":", 2)[0].to_s
    access.start_with?("w") || access.start_with?("a") || access.include?("+")
  end
  private :__writing_mode__

  # The encoding a leading byte-order mark names, with how many bytes it
  # takes. A mark that runs out part way names nothing.
  def self.bom_encoding(bytes)
    return [Encoding::UTF_8, 3] if bytes[0, 3] == [0xEF, 0xBB, 0xBF]
    if bytes[0, 2] == [0xFF, 0xFE]
      return [Encoding::UTF_32LE, 4] if bytes[2, 2] == [0x00, 0x00]
      return [Encoding::UTF_16LE, 2]
    end
    return [Encoding::UTF_16BE, 2] if bytes[0, 2] == [0xFE, 0xFF]
    return [Encoding::UTF_32BE, 4] if bytes[0, 4] == [0x00, 0x00, 0xFE, 0xFF]
    [nil, 0]
  end

  # Read the byte-order mark the stream starts with, if any, and take the
  # encoding it names. The mark is consumed; anything else is left in place.
  def set_encoding_by_bom
    named = __named_encodings__
    unless named.length < 2
      raise ArgumentError, "encoding conversion is set"
    end
    unless named.empty? || named[0].to_s.casecmp("ASCII-8BIT").zero? ||
           named[0].to_s.casecmp("BINARY").zero?
      raise ArgumentError, "encoding is set to #{Encoding.find(named[0])} already"
    end
    unless external_encoding == Encoding::BINARY
      raise ArgumentError, "ASCII incompatible encoding needs binmode"
    end
    return nil if __writing_mode__ && !@__file_mode.to_s.include?("+")
    start = pos
    head = read(4)
    self.pos = start
    found, width = IO.bom_encoding(head.nil? ? [] : head.bytes)
    return nil if found.nil?
    self.pos = start + width
    set_encoding found
    found
  end

  # The encoding a stream reads as. A stream that names none reads as the
  # program's external encoding, which a stream opened while an internal
  # encoding was set keeps as it was at the time.
  def external_encoding
    named = __named_encodings__[0]
    return Encoding.find(named) unless named.nil? || named.empty?
    unless @__encodings_reset
      return Encoding::BINARY if @binmode
      return Encoding::BINARY if @__file_mode.to_s.split(":", 2)[0].to_s.include?("b")
    end
    return nil if __writing_mode__ && @__made_internal.nil?
    return @__made_external unless @__made_internal.nil?
    Encoding.default_external
  end

  # The encoding read text is carried into. A stream carries text nowhere
  # when the two encodings are the same, or when it reads bytes.
  def internal_encoding
    named = __named_encodings__[1]
    return Encoding.find(named) unless named.nil? || named.empty?
    return nil if @__made_internal.nil?
    outer = external_encoding
    return nil if outer.nil? || outer == Encoding::BINARY
    return nil if outer == @__made_internal
    @__made_internal
  end

  # The encodings the stream reads text as, named either as two arguments or
  # as one `"ext:int"` string.
  def set_encoding(external, internal = nil, **options)
    # A stream told to read every line ending the same way carries text
    # rather than bytes, which is what refuses a byte-wise read afterwards.
    @__newline_conversion = options[:universal_newline] ? true : false
    # Naming neither encoding puts the stream back to following the
    # program's own, as they stand now.
    if external.nil? && internal.nil?
      @__file_encoding = nil
      @__encodings_reset = true
      @__noted_encodings = nil
      __note_encodings__
      return self
    end
    @__encodings_reset = nil
    @__file_encoding = IO.__encoding_pair__(
      *(internal.nil? && external.is_a?(String) && external.include?(":") ? external.split(":", 2) : [external, internal]),
      ""
    )
    self
  end

  # Text read from the stream is tagged with the encoding the stream reads
  # as, and carried into the internal one where the stream names one.
  def __tag_read__(text)
    return text if text.nil? || text.empty?
    inner = internal_encoding
    outer = external_encoding
    return text.encode(inner, outer) unless inner.nil?
    return text if outer.nil?
    text.dup.force_encoding outer
  end
  private :__tag_read__

  def readlines(separator = $/, limit = nil, chomp: false)
    separator, limit = StringIO.line_arguments separator, limit
    raise ArgumentError, "invalid limit: 0 for readlines" if limit == 0
    collected = []
    while (held = __read_line__(separator, limit, chomp))
      collected.push held
    end
    collected
  end

  def each_line(separator = $/, limit = nil, chomp: false, &block)
    return to_enum(:each_line, separator, limit, chomp: chomp) if block.nil?
    separator, limit = StringIO.line_arguments separator, limit
    raise ArgumentError, "invalid limit: 0 for each_line" if limit == 0
    while (held = __read_line__(separator, limit, chomp))
      block.call held
    end
    self
  end

  alias_method :each, :each_line

  # One byte at the cursor, or nil where the stream has no more. A stream
  # opened only for writing has no reading side at all.
  def getbyte
    raise IOError, "not opened for reading" unless __readable__
    waiting = @peeked
    unless waiting.nil? || waiting.empty?
      listed = waiting.bytes
      first = listed.shift
      @peeked = listed.empty? ? nil : listed.pack("C*")
      return first
    end
    held = read 1
    return nil if held.nil? || held.empty?
    held.bytes.first
  end

  # An argument read as an Integer, which anything answering `to_int` can be.
  def __as_integer__(held)
    return held if held.is_a? Integer
    unless held.respond_to? :to_int
      # Ruby names nothing by its class, so a nil is reported as `nil`.
      named = held.nil? ? "nil" : held.class.to_s
      raise TypeError, "no implicit conversion of #{named} into Integer"
    end
    held.to_int
  end
  private :__as_integer__

  # A read at a named offset, which leaves the cursor where it was.
  def pread(maxlen, offset, buffer = nil)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    wanted = __as_integer__ maxlen
    at = __as_integer__ offset
    raise ArgumentError, "negative string size (or size too big)" if wanted < 0
    raise Errno::EINVAL, "Invalid argument" if at < 0
    target = nil
    unless buffer.nil?
      if buffer.is_a? String
        target = buffer
      elsif buffer.respond_to? :to_str
        target = buffer.to_str
      else
        raise TypeError, "no implicit conversion of #{buffer.class} into String"
      end
    end
    return buffer.nil? ? "" : buffer if wanted == 0
    held = pos
    self.pos = at
    read_back = read wanted
    self.pos = held
    raise EOFError, "end of file reached" if read_back.nil? || read_back.empty?
    return read_back if buffer.nil?
    named = target.encoding
    target.replace read_back
    target.force_encoding named
    buffer
  end

  # A write at a named offset, which leaves the cursor where it was.
  def pwrite(held, offset)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for writing" unless __writable__
    text = held.to_s
    at = __as_integer__ offset
    standing = pos
    self.pos = at
    write text
    self.pos = standing
    text.bytesize
  end

  # Whether writing is allowed at all. A mode naming only reading leaves no
  # writing side.
  def __writable__
    return true if @__file_mode.nil?
    return true if __both_ways__
    !__opened_for__("r")
  end

  # A hint about how the file will be read. Nothing is passed on to the
  # system, so what is left is refusing what Ruby refuses.
  def advise(kind, offset = 0, length = 0)
    raise IOError, "closed stream" if closed?
    raise TypeError, "advice must be a Symbol" unless kind.is_a? Symbol
    allowed = [:normal, :sequential, :random, :willneed, :dontneed, :noreuse]
    raise NotImplementedError, "Unsupported advice: #{kind}" unless allowed.include? kind
    [offset, length].each do |held|
      unless held.is_a? Integer
        raise TypeError, "no implicit conversion of #{held.class} into Integer"
      end
      if held > 9223372036854775807 || held < -9223372036854775808
        raise RangeError, "bignum too big to convert into 'long long'"
      end
    end
    nil
  end

  # Bytes put back are read again before anything else in the stream. An
  # Integer names one byte, and only its low eight bits are kept.
  def ungetbyte(held)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    return nil if held.nil?
    text = if held.is_a? Integer
      [held & 0xff].pack("C")
    elsif held.is_a? String
      held
    elsif held.respond_to? :to_str
      held.to_str
    else
      raise TypeError, "no implicit conversion of #{held.class} into String"
    end
    @peeked = @peeked.nil? ? text : text + @peeked
    nil
  end

  def readbyte
    held = getbyte
    raise EOFError, "end of file reached" if held.nil?
    held
  end

  # Every byte in turn. Without a block the walk is handed back, and it
  # cannot say how many bytes are left to come.
  def each_byte
    return Enumerator.over(self, :each_byte) unless block_given?
    raise IOError, "closed stream" if closed?
    while (held = getbyte)
      yield held
    end
    self
  end

  # One character at the cursor. A character spelled in several bytes is read
  # to its end rather than cut in half.
  # One character, which is as many bytes as the character is spelled with.
  def getc
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    waiting = @peeked
    unless waiting.nil? || waiting.empty?
      first = waiting[0]
      rest = waiting[1, waiting.length - 1]
      @peeked = rest.nil? || rest.empty? ? nil : rest
      return first
    end
    held = IO.__stream__ "getc", __stream_handle__, "", 0
    return nil if held.nil? || held.empty?
    __tag_read__ held
  end

  def readchar
    held = getc
    raise EOFError, "end of file reached" if held.nil?
    held
  end

  def each_char
    return Enumerator.over(self, :each_char) unless block_given?
    raise IOError, "closed stream" if closed?
    while (held = getc)
      yield held
    end
    self
  end

  def each_codepoint
    return Enumerator.over(self, :each_codepoint) unless block_given?
    each_char { |held| yield held.ord }
    self
  end

  alias_method :codepoints, :each_codepoint

  # How many lines have been read. Only a stream being read counts them, so
  # one that cannot be read has none to report.
  def lineno
    __reading_side__
    @lineno.nil? ? 0 : @lineno
  end

  def lineno=(held)
    __reading_side__
    counted = __as_integer__ held
    # The line count is one the operating system holds, which is as wide as
    # a C int and no wider.
    if counted.bit_length > 31
      raise RangeError, "integer #{counted} too big to convert to `int'"
    end
    @lineno = counted
  end

  # Refuse a stream that has no reading side: a closed one, one opened only
  # to write, and one whose reading side was closed.
  def __reading_side__
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" if @read_closed
    # A stream over a child process keeps its own mark for the side that was
    # closed, since the interpreter is what closed it.
    if @__popen_read_closed
      raise IOError, "not opened for reading"
    end
    raise IOError, "not opened for reading" unless __readable__
    nil
  end
  private :__reading_side__

  # Reading one character ahead is the only way to tell a stream that has
  # nothing left from one whose writer has not written yet, so the character
  # is held back for the next read to hand out.
  def eof?
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" if @read_closed
    raise IOError, "not opened for reading" if __opened_for__("w") && !__both_ways__
    return false unless @peeked.nil?
    held = IO.__stream__ "read", __stream_handle__, "", 1
    return true if held.nil? || held.empty?
    @peeked = held
    false
  end

  alias_method :eof, :eof?

  # Metorex hands every write to the operating system as it is made, so
  # nothing is ever waiting to be flushed.
  def flush
    raise IOError, "closed stream" if closed?
    __drain__
    @line_buffered = nil
    self
  end

  def fsync
    raise IOError, "closed stream" if closed?
    0
  end

  def sync
    raise IOError, "closed stream" if closed?
    @sync == true
  end

  def sync=(wanted)
    raise IOError, "closed stream" if closed?
    @sync = wanted ? true : false
    # Only a program asking for it makes a stream hold what is written.
    @holding = !@sync
  end

  def tty?
    raise IOError, "closed stream" if closed?
    IO.__stream__ "tty?", __stream_handle__, "", 0
  end

  alias_method :isatty, :tty?

  def nonblock?
    raise IOError, "closed stream" if closed?
    IO.__stream__ "nonblock?", __stream_handle__, "", 0
  end

  def nonblock=(wanted)
    IO.__stream__ "nonblock=", __stream_handle__, "", wanted ? 1 : 0
    wanted
  end

  def nonblock(wanted = true)
    was = nonblock?
    self.nonblock = wanted
    return self unless block_given?
    begin
      yield self
    ensure
      self.nonblock = was
    end
  end

  def close_on_exec?
    raise IOError, "closed stream" if closed?
    IO.__stream__ "close_on_exec?", __stream_handle__, "", 0
  end

  def close_on_exec=(wanted)
    raise IOError, "closed stream" if closed?
    IO.__stream__ "close_on_exec=", __stream_handle__, "", wanted ? 1 : 0
    wanted
  end

  # Reading and writing bytes rather than characters: the stream is written
  # in binary from here on, and nothing is converted on the way in.
  def binmode
    raise IOError, "closed stream" if closed?
    @binmode = true
    @__file_encoding = "ASCII-8BIT"
    self
  end

  def binmode?
    raise IOError, "closed stream" if closed?
    @binmode == true
  end

  # Ask the operating system about this stream's descriptor, or set one of
  # the flags it keeps. `Fcntl` names the numbers.
  def fcntl(command, argument = 0)
    raise IOError, "closed stream" if closed?
    held = argument == true ? 1 : (argument == false || argument.nil? ? 0 : argument.to_i)
    IO.__stream__ "fcntl", __stream_handle__, "", command.to_i, held
  end

  alias_method :ioctl, :fcntl

  # Point this stream at another place. The descriptor keeps its number, so
  # everything already reading or writing through it reaches the new place.
  def reopen(target, mode = nil)
    # A closed stream may be opened again over a file named by path, which is
    # what reopening one is for. Pointed at another stream it stays closed.
    named = target.is_a?(String) || (!target.is_a?(IO) && target.respond_to?(:to_path))
    raise IOError, "closed stream" if closed? && !named
    other = __reopen_target__ target, mode
    raise IOError, "closed stream" if other.closed?
    # A closed stream is opened again over what it was pointed at, which is
    # what reopening one is for. A stream still open keeps its descriptor,
    # so everything already reading or writing through it follows along.
    if closed?
      @handle = other.__stream_handle__
      @closed = false
    else
      IO.__stream__ "reopen", __stream_handle__, "", other.__stream_handle__
    end
    @__file_path = other.path
    @__file_mode = mode.nil? ? other.instance_variable_get(:@__file_mode) : mode
    @read_closed = false
    @write_closed = false
    @peeked = nil
    @lineno = 0
    self
  end

  # The stream a `reopen` was given. A name opens a file, and anything else
  # spells itself out as the stream it stands for.
  def __reopen_target__(target, mode)
    if target.is_a?(String) || (!target.is_a?(IO) && target.respond_to?(:to_path))
      path = target.is_a?(String) ? target : target.to_path
      return File.open(path, mode.nil? ? "r" : mode)
    end
    return target if target.is_a?(IO)
    spelled = target.to_io
    unless spelled.is_a?(IO)
      raise TypeError, "can't convert #{target.class} to IO (#{target.class}#to_io gives #{spelled.class})"
    end
    spelled
  end
  private :__reopen_target__

  # Wait until there is something to read, or until the wait runs out. A
  # timeout of nil waits for as long as it takes.
  def wait_readable(timeout = nil)
    __wait_ready__("read", timeout)
  end

  def wait_writable(timeout = nil)
    __wait_ready__("write", timeout)
  end

  def wait(timeout = nil, mode = :read)
    __wait_ready__(mode.to_s.include?("write") ? "write" : "read", timeout)
  end

  def __wait_ready__(mode, timeout)
    raise IOError, "closed stream" if closed?
    waited = timeout.nil? ? -1 : (timeout.to_f * 1000).to_i
    # A wait longer than the counter holds is the same as waiting forever.
    waited = -1 if waited > 2147483647 || waited < -1
    IO.__stream__("wait", __stream_handle__, mode, waited) ? self : nil
  end
  private :__wait_ready__


  def to_io
    self
  end

  # The numbers the operating system keeps about this stream, read through
  # the descriptor rather than through a name, since a stream is not always
  # open on a file that has one.
  def stat
    raise IOError, "closed stream" if closed?
    File::Stat.new(@handle.nil? ? IO::NULL : "/dev/fd/#{fileno}")
  end

  # A copy holds a descriptor of its own over the same file, so closing
  # either one leaves the other open. The copy is never handed to a child
  # process, which is what Ruby sets on it.
  def dup
    raise IOError, "closed stream" if closed?
    copied = self.class.allocate
    copied.__send__ :__take__, IO.__stream__("dup", __stream_handle__, "", 0), path
    # A copy of a stream opened by name is opened on the same name, which is
    # what the methods written for a file read.
    ["@__file_path", "@__file_mode", "@__file_encoding", "@binmode"].each do |named|
      held = instance_variable_get named
      copied.instance_variable_set named, held unless held.nil?
    end
    copied.close_on_exec = true
    copied
  end

  def inspect
    return "#<IO: (closed)>" if closed?
    "#<IO:fd #{fileno}>"
  end
end

# A file opened by name answers the descriptor questions an IO answers, over
# a descriptor opened the first time one of them is asked.
class File
  # Whether a name says where it is from the root, rather than from wherever
  # the program happens to be. A `~` says nothing about the root.
  def self.absolute_path?(name)
    File.__path_text__(name).start_with? File::SEPARATOR
  end

  # The name a path names from the root. A relative one is read from the
  # directory given, or from the one the program is working in.
  def self.absolute_path(name, directory = nil)
    held = File.__path_text__ name
    return held if held.start_with? File::SEPARATOR
    base = directory.nil? ? Dir.pwd : File.absolute_path(directory)
    File.join base, held
  end

  # The name a path names from the root, with `~`, `.`, and `..` read out and
  # the runs of separators inside it collapsed. The leading run is left as it
  # was written, which is what Ruby does with `//host/share`.
  def self.expand_path(name, directory = nil)
    held = File.__path_text__ name
    tag = held.encoding
    if held.start_with? "~"
      held = File.__expanded_home__ held
    elsif !held.start_with? File::SEPARATOR
      # Reading the directory the program works in means writing its name
      # ahead of this one, which text in an encoding without ASCII cannot
      # stand beside.
      unless Encoding.default_external.ascii_compatible?
        raise Encoding::CompatibilityError,
              "ASCII incompatible encoding: #{Encoding.default_external}"
      end
      base = directory.nil? ? Dir.pwd : File.expand_path(directory)
      held = held.empty? ? base : File.join(base, held)
    end
    answered = File.__without_dots__ held
    answered.force_encoding tag
    answered
  end

  # A name opening with `~` read as the home directory it names: the one the
  # program belongs to, or the one the user named after it does.
  def self.__expanded_home__(held)
    at = held.index File::SEPARATOR
    head = at.nil? ? held : held[0, at]
    rest = at.nil? ? "" : held[at, held.length - at]
    return Dir.home(head[1, head.length - 1]) + rest unless head == "~"
    # `HOME` says where the program's own files are. One set to nothing says
    # nothing, and one that does not start from the root names no home.
    named = ENV["HOME"]
    return Dir.home + rest if named.nil?
    if named.empty?
      raise ArgumentError, "couldn't find HOME environment -- expanding `~'"
    end
    unless named.start_with? File::SEPARATOR
      raise ArgumentError, "non-absolute home"
    end
    named + rest
  end

  # A path with `.` and `..` read out and the separators inside it collapsed.
  def self.__without_dots__(held)
    leading = ""
    rest = held
    while rest.start_with? File::SEPARATOR
      leading = leading + File::SEPARATOR
      rest = rest[1, rest.length - 1]
    end
    walked = []
    rest.split(File::SEPARATOR).each do |part|
      next if part.empty? || part == "."
      if part == ".."
        # Nothing stands above the root, so a step up from there is no step
        # at all. A relative path keeps the steps it cannot take.
        if walked.empty?
          walked.push part if leading.empty?
        elsif walked.last == ".."
          walked.push part
        else
          walked.pop
        end
        next
      end
      walked.push part
    end
    answered = leading + walked.join(File::SEPARATOR)
    answered.empty? ? "." : answered
  end

  # A file cut down to the count of bytes it is told to keep, or filled out
  # with zero bytes to reach it.
  def self.truncate(name, length)
    path = File.__path_text__ name
    counted = length
    unless counted.is_a? Integer
      unless counted.respond_to? :to_int
        named = counted.nil? ? "nil" : counted.class.to_s
        raise TypeError, "no implicit conversion of #{named} into Integer"
      end
      counted = counted.to_int
    end
    held = File.open path, "r+"
    begin
      held.truncate counted
    ensure
      held.close
    end
    0
  end

  # A path as the text it stands for, whatever it was written as.
  def self.__path_text__(name)
    # A String subclass carries the text a path is spelled with, and the path
    # itself is that text rather than the object.
    return name.to_s if name.is_a? String
    return name.to_path if name.respond_to? :to_path
    return name.to_str if name.respond_to? :to_str
    raise TypeError, "no implicit conversion of #{name.class} into String"
  end

  # The parts of a path joined with a separator between them. Two parts that
  # each carry one at the boundary keep the right one's, and a part that is
  # itself a list of parts is joined before it is used.
  def self.join(*parts)
    return "" if parts.empty?
    held = ""
    parts.each_with_index do |part, index|
      spelled = File.__joined_part__ part, []
      held = index == 0 ? spelled : File.__join_two__(held, spelled)
    end
    held.dup
  end

  # One part of a path as the text it stands for. An Array is a path of its
  # own, and one that reaches itself names no path at all.
  def self.__joined_part__(part, walking)
    if part.is_a? Array
      if walking.any? { |seen| seen.equal? part }
        raise ArgumentError, "recursive array"
      end
      inside = walking + [part]
      held = ""
      part.each_with_index do |inner, index|
        spelled = File.__joined_part__ inner, inside
        held = index == 0 ? spelled : File.__join_two__(held, spelled)
      end
      return held
    end
    spelled = if part.is_a? String
      part
    elsif part.respond_to? :to_str
      part.to_str
    elsif part.respond_to? :to_path
      part.to_path
    else
      raise TypeError, "no implicit conversion of #{part.class} into String"
    end
    if spelled.include? "\0"
      raise ArgumentError, "string contains null byte"
    end
    spelled
  end

  # Two parts of a path, one after the other. A separator at the boundary is
  # left as the part carrying it wrote it, and where both carry one the right
  # part's stands.
  def self.__join_two__(left, right)
    separator = File::SEPARATOR
    if left.end_with?(separator) && right.start_with?(separator)
      trimmed = left
      while trimmed.end_with? separator
        trimmed = trimmed[0, trimmed.length - separator.length]
      end
      return trimmed + right
    end
    if left.end_with?(separator) || right.start_with?(separator)
      return left + right
    end
    left + separator + right
  end

  def __stream_handle__
    return @handle unless @handle.nil?
    __note_encodings__
    written = @__file_mode.to_s
    both = written.include? "+"
    opening = if written.start_with?("a")
      both ? 5 : 2
    elsif written.start_with?("r")
      both ? 3 : 0
    elsif written.start_with?("w")
      both ? 4 : 1
    else
      0
    end
    @handle = IO.__stream__ "open", 0, @__file_path.to_s, opening
    __apply_options__
    @handle
  end

  # What the options a file was opened with say beyond the mode: the
  # encodings it reads and writes in, what ends the lines it writes, and
  # whether it reads and writes bytes.
  def __apply_options__
    held = @__file_options
    return self if held.nil?
    outer = held[:external_encoding]
    inner = held[:internal_encoding]
    if !outer.nil? || !inner.nil?
      @__file_encoding = IO.__encoding_pair__ outer, inner, ""
    end
    @binmode = true if held[:binmode]
    self
  end
  private :__apply_options__

  def path
    @__file_path
  end

  def pid
    raise IOError, "closed stream" if closed?
    nil
  end

end

# The numbers the operating system keeps about a file, presented the way Ruby
# presents them.
class File
  class Stat
    include Comparable

    def initialize(path, follow = true)
      unless path.is_a?(String)
        if path.respond_to?(:to_path)
          path = path.to_path
        elsif path.respond_to?(:to_str)
          path = path.to_str
        else
          raise TypeError, "no implicit conversion of #{path.class} into String"
        end
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

  def self.atime(path)
    File::Stat.new(path, true).atime
  end

  def self.mtime(path)
    File::Stat.new(path, true).mtime
  end

  def self.ctime(path)
    File::Stat.new(path, true).ctime
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

  # The numbers come from the descriptor rather than the name, so a file
  # that was removed while it is still open still reports its own.
  def stat
    raise IOError, "closed stream" if closed?
    File::Stat.new("/dev/fd/#{fileno}")
  end

  # The one file a handle stands for, which Ruby answers 0 for.
  def chown(owner, group)
    File.chown(owner, group, path)
    0
  end

  def lstat
    raise IOError, "closed stream" if closed?
    File::Stat.new(self.path, false)
  end

  def atime
    stat.atime
  end

  def mtime
    stat.mtime
  end

  def ctime
    stat.ctime
  end

  def birthtime
    stat.birthtime
  end

  # A handle that was never opened has nothing behind it, so every reading
  # of it is refused rather than answered.
  def read(*)
    raise IOError, "uninitialized stream" if @__file_path.nil?
    super
  end

  # The name the handle was opened under. An IO that never came from a name
  # has none, which is what `to_path` answers for.
  # The name an object stands for: a String as it is, and anything else
  # through `to_path`. Ruby leaves the name exactly as it was written.
  def self.path(held)
    return __checked_path__ held if held.is_a? String
    unless held.respond_to? :to_path
      raise TypeError, "no implicit conversion of #{held.class} into String"
    end
    named = held.to_path
    unless named.is_a? String
      raise TypeError, "no implicit conversion of #{named.class} into String"
    end
    __checked_path__ named
  end

  # A name the operating system can take: NUL ends a C string, and a name
  # spelled in an encoding without ASCII cannot be compared with one.
  def self.__checked_path__(named)
    unless named.encoding.ascii_compatible?
      raise Encoding::CompatibilityError,
            "incompatible character encodings: #{named.encoding} and US-ASCII"
    end
    if named.include? "\0"
      raise ArgumentError, "path name contains null byte"
    end
    named
  end

  # Ruby hands back a fresh String each time, tagged the way the name it was
  # opened under was, so a program may change what it is given.
  def path
    return nil if @__file_path.nil?
    @__file_path.dup
  end

  def to_path
    path
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
    # An event may be named with a String or with anything answering
    # `to_sym`, and only a Symbol comes back from that.
    events = events.map do |event|
      named = if event.is_a?(Symbol)
        event
      elsif event.respond_to?(:to_sym)
        event.to_sym
      else
        raise TypeError, "#{event.inspect} is not a symbol nor a string"
      end
      unless named.is_a?(Symbol)
        raise TypeError, "#{event.inspect} is not a symbol nor a string"
      end
      named
    end
    events.each do |event|
      unless KNOWN_EVENTS.include?(event)
        raise ArgumentError, "unknown event: #{event}"
      end
    end
    raise ArgumentError, "must be called with a block" if block.nil?
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
    __reentrant__ { yield }
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

  # The scope the event fired in, as the Binding that reads its locals.
  def binding
    __reading__(:binding)
  end

  # The source a `script_compiled` event compiled, where it came from a
  # string rather than a file.
  def eval_script
    __reading__(:eval_script)
  end

  # The exception a `raise` or a `rescue` event is about.
  def raised_exception
    __reading__(:raised_exception)
  end

  # What the block a `b_call` or `b_return` event is about takes.
  def parameters
    __reading__(:parameters)
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

class Struct
  # The members a struct holds, set from the values given. A struct class of
  # the program's own may write its own and reach this one with `super`.
  def initialize(*values, **keywords)
    named = self.class.members
    if values.empty? && !keywords.empty?
      keywords.each do |key, value|
        instance_variable_set "@__struct_member_#{key}", value
      end
    else
      named.each_with_index do |member, index|
        instance_variable_set "@__struct_member_#{member}", values[index]
      end
    end
    self
  end
  private :initialize
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

    # The map's address, and the pair of addresses each entry holds. What a
    # key or a value writes itself as is read through Kernel, since an object
    # rooted at BasicObject answers no `inspect` of its own.
    def inspect
      written = @entries.map do |entry|
        "#{ObjectSpace::WeakMap.written(entry[0])} => #{ObjectSpace::WeakMap.written(entry[1])}"
      end
      named = format("#<ObjectSpace::WeakMap:0x%016x", object_id)
      written.empty? ? "#{named}>" : "#{named}: #{written.join(", ")}>"
    end

    def self.written(held)
      ::Kernel.instance_method(:inspect).bind(held).call
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

    # What an object calls its class, asked without going through the object
    # itself, since one may answer nothing at all.
    def self.named(held)
      ::Kernel.instance_method(:class).bind(held).call.to_s
    end

    # Whether an object has a `hash` at all, asked of its class so an object
    # answering nothing of Kernel's can be asked too.
    def self.answers_hash?(held)
      holder = ::Kernel.instance_method(:class).bind(held).call
      named = holder.ancestors.map { |ancestor| ancestor.to_s }
      named.include?("Object") || named.include?("Kernel") || holder.method_defined?(:hash)
    end

    def []=(key, value)
      unless collectable?(key)
        raise ArgumentError, "WeakKeyMap must be garbage collectable"
      end
      # A key is found again by its hash, so one that has none cannot be
      # stored at all. An object that descends from BasicObject alone answers
      # none of the names Kernel gives, `hash` among them, so the question is
      # put to its class rather than to the object.
      unless ObjectSpace::WeakKeyMap.answers_hash?(key)
        raise NoMethodError,
              "undefined method 'hash' for an instance of #{ObjectSpace::WeakKeyMap.named(key)}"
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
      # An object rooted at BasicObject answers none of Kernel's names, so
      # the classes are asked about the key rather than the key about itself.
      return false if NilClass === key || TrueClass === key || FalseClass === key
      return false if Numeric === key || Symbol === key
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

# A queue is built by the interpreter, and `initialize` is the private method
# Ruby reports for it.
class Queue
  def initialize(*items)
    self
  end
  private :initialize
end

class SizedQueue
  def initialize(*counted)
    self
  end
  private :initialize
end

class Dir
  include Enumerable

  # The directory a descriptor names, made the one the program works from.
  # With a block the program works from there only while the block runs.
  def self.fchdir(number)
    was = Dir.pwd
    IO.__stream__ "fchdir", 0, "", number
    return 0 unless block_given?
    begin
      yield
    ensure
      Dir.chdir was
    end
  end

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
    @names = options.key?(:encoding) ? Dir.entries(@path, encoding: options[:encoding]) : Dir.entries(@path)
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
    # A descriptor another Dir already closed is gone, which is what the
    # operating system says when this one is asked to close it too.
    unless @handle.nil?
      unless IO.__stream__("live?", @handle, "", 0)
        raise Errno::EBADF, "closedir"
      end
      IO.__stream__ "close", @handle, "", 0
    end
    @closed = true
    nil
  end

  # The number the operating system holds this directory under, opened the
  # first time one is asked for.
  def fileno
    self.refuse_closed
    @handle = IO.__stream__("open", 0, @path, 0) if @handle.nil?
    IO.__stream__ "fileno", @handle, "", 0
  end

  # Another Dir over a descriptor already open, which reads the same
  # directory and closes the same descriptor.
  def self.for_fd(number)
    made = allocate
    made.__send__ :__adopt__, number
    made
  end

  def __adopt__(number)
    @handle = IO.__stream__ "adopt", 0, "", number
    @path = nil
    @names = []
    @position = 0
    @closed = false
    self
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
    return self.to_enum(:each_child) if block.nil?
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
  # `putc` writes one character to the standard output stream.
  def putc(held)
    $stdout.putc held
  end
  module_function :putc

  # `test` names a file test by a single character, the way the shell's own
  # tests are spelled. The two-file tests take a second path.
  def test(command, first, second = nil)
    named = command.is_a?(Integer) ? command.chr : command.to_s
    case named
    when "b" then File.blockdev?(first)
    when "c" then File.chardev?(first)
    when "d" then File.directory?(first)
    when "e" then File.exist?(first)
    when "f" then File.file?(first)
    when "g" then File.setgid?(first)
    when "G" then File.grpowned?(first)
    when "k" then File.sticky?(first)
    when "l" then File.symlink?(first)
    when "o" then File.owned?(first)
    when "O" then File.owned?(first)
    when "p" then File.pipe?(first)
    when "r" then File.readable?(first)
    when "R" then File.readable_real?(first)
    when "s" then File.size?(first)
    when "S" then File.socket?(first)
    when "u" then File.setuid?(first)
    when "w" then File.writable?(first)
    when "W" then File.writable_real?(first)
    when "x" then File.executable?(first)
    when "X" then File.executable_real?(first)
    when "z" then File.zero?(first)
    when "A" then File.atime(first)
    when "C" then File.ctime(first)
    when "M" then File.mtime(first)
    when "-" then File.identical?(first, second)
    when "=" then File.mtime(first) == File.mtime(second)
    when "<" then File.mtime(first) < File.mtime(second)
    when ">" then File.mtime(first) > File.mtime(second)
    else
      raise ArgumentError, "unknown command #{named.inspect}"
    end
  end
  module_function :test

  # Which of the streams handed in are ready, which is IO.select under a name
  # every object answers to.
  def select(readers = nil, writers = nil, errored = nil, timeout = nil)
    IO.select readers, writers, errored, timeout
  end

  private :select

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
  include Enumerable

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
    return "-" if @reading_stdin
    held = self.file
    return "-" if @reading_stdin
    held.path
  end

  def filename
    self.path
  end

  # The encodings the files are read as. ARGF keeps them for the files it has
  # yet to open as well as the one it is reading.
  def set_encoding(external, internal = nil)
    outer = external
    inner = internal
    if internal.nil? && external.is_a?(String) && external.include?(":")
      outer, inner = external.split(":", 2)
    end
    @external_encoding = outer.nil? || outer == "" ? nil : Encoding.find(outer)
    @internal_encoding = inner.nil? || inner == "" ? nil : Encoding.find(inner)
    self
  end

  def external_encoding
    return @external_encoding unless @external_encoding.nil?
    Encoding.default_external
  end

  def internal_encoding
    return @internal_encoding unless @internal_encoding.nil?
    Encoding.default_internal
  end

  # Text read from a file is tagged with the encoding ARGF reads as, and
  # carried into the internal one where there is one.
  def __tagged__(text)
    return text if text.nil? || text.empty?
    inner = self.internal_encoding
    outer = self.external_encoding
    return text.encode(inner, outer) unless inner.nil?
    return text if outer.nil?
    text.dup.force_encoding outer
  end
  private :__tagged__

  def fileno
    if @drained || (self.__names__.empty? && @current.nil?)
      raise ArgumentError, "closed stream"
    end
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
    $. = counted
  end

  def binmode
    @binmode = true
    self
  end

  def binmode?
    @binmode == true
  end

  def closed?
    # Standard input belongs to the program, so ARGF never reports it closed.
    return false if @reading_stdin
    self.file.closed?
  end

  def close
    self.file
    return self if @reading_stdin
    self.file.close
    self
  end

  # Move past whatever is left of the file being read, so the next line comes
  # from the one after it.
  def skip
    @current = nil unless self.__names__.empty?
    self
  end

  # Reading a line records which file it came from and how many have been
  # read, which is what `$FILENAME` and `$.` report.
  def gets(*separator)
    loop do
      self.__open_current__
      if @current.nil?
        $FILENAME = nil
        return nil
      end
      $FILENAME = @current.path
      line = @current.gets(*separator)
      if line.nil?
        if self.__names__.empty?
          @drained = true
          __finish_edit__
          break
        end
        @current = nil
        __finish_edit__
        next
      end
      @lineno = self.__lineno__ + 1
      $. = @lineno
      # Reading in binary hands back bytes rather than text.
      # `$_` names the line last read, which is what a program written
      # without a variable of its own reads back.
      $_ = @binmode ? line.b : __tagged__(line)
      return $_
    end
    nil
  end

  def readline(*separator)
    line = self.gets(*separator)
    raise EOFError, "end of file reached" if line.nil?
    line
  end

  # Every line of every file, read as though the list were one file. A
  # separator stands in for the newline, the way `IO#each_line` takes one.
  def each_line(*separator, &block)
    return to_enum(:each_line, *separator) if block.nil?
    while (line = self.gets(*separator))
      block.call line
    end
    self
  end

  def each(*separator, &block)
    self.each_line(*separator, &block)
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
  def read(length = nil, buffer = nil)
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
    if collected.empty? && !length.nil? && length > 0
      buffer.replace "" unless buffer.nil?
      return nil
    end
    held = @binmode ? collected.b : __tagged__(collected)
    return buffer.replace held unless buffer.nil?
    held
  end

  # As much as is asked for of the file being read, which is never carried
  # across into the next one. The file running out hands back an empty String
  # and moves on, and running out of the last file is the end.
  def readpartial(length = nil, buffer = nil)
    if length.nil?
      raise ArgumentError, "wrong number of arguments (given 0, expected 1..2)"
    end
    buffer.replace "" unless buffer.nil?
    self.__open_current__
    raise EOFError, "end of file reached" if @current.nil?
    held = @current.read length
    if held.nil? || held.empty?
      if self.__names__.empty?
        @drained = true
        __finish_edit__
        raise EOFError, "end of file reached"
      end
      @current = nil
      __finish_edit__
      held = ""
    end
    held = @binmode ? held.b : __tagged__(held)
    return buffer.replace held unless buffer.nil?
    held
  end

  # As much of the file being read as is there right now, which is as much as
  # `readpartial` hands back for a file and never crosses into the next one.
  def read_nonblock(length = nil, buffer = nil, exception: true)
    if length.nil?
      raise ArgumentError, "wrong number of arguments (given 0, expected 1..2)"
    end
    buffer.replace "" unless buffer.nil?
    self.__open_current__
    raise EOFError, "end of file reached" if @current.nil?
    held = begin
      @current.read_nonblock length, nil, exception: exception
    rescue EOFError
      ""
    end
    return held unless held.is_a? String
    if held.empty?
      if self.__names__.empty?
        @drained = true
        __finish_edit__
        raise EOFError, "end of file reached"
      end
      @current = nil
      __finish_edit__
    end
    held = @binmode ? held.b : __tagged__(held)
    return buffer.replace held unless buffer.nil?
    held
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

  def each_byte(&block)
    return to_enum(:each_byte) if block.nil?
    while (byte = self.__next_byte__)
      block.call byte
    end
    self
  end

  def each_char(&block)
    return to_enum(:each_char) if block.nil?
    while (character = self.getc)
      block.call character
    end
    self
  end

  def each_codepoint(&block)
    return to_enum(:each_codepoint) if block.nil?
    while (character = self.getc)
      block.call character.ord
    end
    self
  end

  # The next byte of the list, crossing into the following file when the one
  # being read runs out.
  def __next_byte__
    loop do
      self.__open_current__
      return nil if @current.nil?
      byte = @current.getbyte
      return byte unless byte.nil?
      return nil if self.__names__.empty?
      @current = nil
    end
  end
  private :__next_byte__

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

  # Seeking moves the file now being read. A stream that has not opened one
  # opens the first name first, so the offset lands somewhere.
  def seek(offset, whence = IO::SEEK_SET)
    self.file.seek offset, whence
  end

  # A stream whose last file has been read to the end is closed, and closed
  # streams cannot be put back to the start.
  def rewind
    raise ArgumentError, "closed stream" if @drained
    self.file.rewind
    @lineno = 0
    $. = 0
    0
  end

  # Open the next name when there is no file being read. The name comes off
  # the list as it opens, which is what `argv` reports on.
  def __open_current__
    return if @current
    # A stream over no names at all reads standard input, which is what a
    # program handed nothing on the command line reads from.
    if self.__names__.empty?
      return if @walked
      @walked = true
      @reading_stdin = true
      @current = $stdin
      return @current
    end
    @walked = true
    named = self.__names__.shift
    # A name of `-` stands for standard input, which the program owns rather
    # than ARGF.
    if named == "-"
      @reading_stdin = true
      @current = $stdin
      return @current
    end
    @reading_stdin = false
    # `-i` edits each file as it is read: the file is set aside under the
    # backup name, and what the program writes takes its place.
    extension = $-i
    unless extension.nil?
      @edited_name = named
      @backup_name = named + (extension.empty? ? ".__metorex_edit__" : extension)
      File.rename named, @backup_name
      @current = File.open(@backup_name, "r")
      @written = File.open(named, "w")
      $stdout = @written
      return @current
    end
    @current = File.open(named, "r")
  end
  private :__open_current__

  # Close the file the program was writing in place of the one being read,
  # and put standard output back where it was.
  def __finish_edit__
    return if @written.nil?
    @written.close
    @written = nil
    $stdout = STDOUT
    # A backup asked for under no extension is not kept.
    if !@backup_name.nil? && @backup_name.end_with?(".__metorex_edit__")
      File.unlink @backup_name
    end
    @backup_name = nil
    nil
  end
  private :__finish_edit__

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

class Float
  # The simplest fraction standing no further away than the tolerance given.
  # Without one the tolerance is half the gap to the next Float, so the
  # answer reads back as this same Float.
  def rationalize(*limits)
    if limits.size > 1
      raise ArgumentError, "wrong number of arguments (given #{limits.size}, expected 0..1)"
    end
    raise FloatDomainError, to_s if nan? || infinite?
    return to_r.rationalize(limits[0]) unless limits.empty?
    to_r.rationalize(Rational(Math.ldexp(1, Math.frexp(self)[1] - 53).to_r, 2))
  end
end

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

module ObjectSpace
  # The finalizers a program has asked for, under the id of the object each
  # one belongs to. Metorex frees an object when its last reference goes and
  # nothing watches for that, so a finalizer runs as the program ends.
  def self.__finalizers__
    @finalizers = {} if @finalizers.nil?
    @finalizers
  end

  def self.define_finalizer(held, callable = nil, &block)
    finalizer = callable.nil? ? block : callable
    raise ArgumentError, "no finalizer given" if finalizer.nil?
    unless finalizer.respond_to? :call
      raise ArgumentError, "no finalizer given"
    end
    named = held.object_id
    ObjectSpace.__finalizers__[named] = ObjectSpace.__finalizers__.fetch(named, []) + [finalizer]
    unless @armed
      @armed = true
      at_exit do
        ObjectSpace.__finalizers__.each do |id, listed|
          listed.each { |one| one.call id }
        end
      end
    end
    [0, finalizer]
  end

  # A clone carries the finalizers its original was given, which is what
  # makes both of them run one as the program ends.
  def self.__carry_finalizers__(from, to)
    listed = __finalizers__[from.object_id]
    return nil if listed.nil?
    __finalizers__[to.object_id] = __finalizers__.fetch(to.object_id, []) + listed
    nil
  end

  def self.undefine_finalizer(held)
    if held.frozen?
      raise FrozenError, "can't modify frozen #{held.class}: #{held.inspect}"
    end
    ObjectSpace.__finalizers__.delete held.object_id
    held
  end

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
  # The collector's settings. `:implementation` names the collector and is
  # read-only, and the rest are whatever the collector takes. Metorex frees
  # an object when its last reference goes, so it takes none.
  def self.config(options = :__none__)
    @settings = {} if @settings.nil?
    current = { implementation: "metorex" }.merge(@settings)
    return current if options == :__none__ || options.nil?
    unless options.is_a? Hash
      raise ArgumentError, "expecting a Hash, got #{options.class}"
    end
    if options.key? :implementation
      raise ArgumentError, 'Attempting to set read-only key "Implementation"'
    end
    current
  end

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


class Encoding
  # The encoding the machine's locale names. It is read once, so a program
  # writing to the environment afterwards does not change it.
  def self.locale_charmap
    return @locale_charmap unless @locale_charmap.nil?
    named = ENV["LC_ALL"] || ENV["LC_CTYPE"] || ENV["LANG"] || ""
    @locale_charmap = if named.empty? || named == "C" || named == "POSIX"
      "US-ASCII"
    elsif named.include?(".")
      named.split(".", 2)[1]
    else
      "UTF-8"
    end
  end

  # Every name an encoding answers to, its own and the aliases pointing at it.
  def self.name_list
    named = list.map { |held| held.name }
    aliases.each_key { |held| named.push(held) unless named.include?(held) }
    named
  end

  # The names this encoding answers to, its own first.
  def names
    held = [name]
    Encoding.aliases.each do |spelled, stands_for|
      held.push(spelled) if stands_for == name && !held.include?(spelled)
    end
    held
  end

  # The encoding the source of a program is read as. `-K` names it, and
  # without that flag it is UTF-8.
  def self.__source__
    @__source__ || __running_source__
  end

  def self.__source__= named
    @__source__ = named.is_a?(Encoding) ? named : Encoding.find(named)
  end

  # A conversion from one encoding to another, along with the flags saying
  # what to do with what the destination cannot spell.
  # What a conversion reports when the destination cannot spell a character.
  class UndefinedConversionError
    attr_reader :source_encoding
    attr_reader :destination_encoding
    attr_reader :error_char

    def initialize message = nil, source = nil, destination = nil, character = nil
      super message
      @source_encoding = source
      @destination_encoding = destination
      @error_char = character
    end

    def source_encoding_name
      @source_encoding.nil? ? nil : @source_encoding.name
    end

    def destination_encoding_name
      @destination_encoding.nil? ? nil : @destination_encoding.name
    end
  end

  # What a conversion reports when the source cannot read its own bytes.
  class InvalidByteSequenceError
    attr_reader :source_encoding
    attr_reader :destination_encoding
    attr_reader :error_bytes
    attr_reader :readagain_bytes

    def initialize message = nil, source = nil, destination = nil, wrong = nil, rest = nil, truncated = nil
      super message
      @source_encoding = source
      @destination_encoding = destination
      @error_bytes = wrong.nil? ? "" : wrong
      @readagain_bytes = rest.nil? ? "" : rest
      @truncated = truncated
    end

    def source_encoding_name
      @source_encoding.nil? ? nil : @source_encoding.name
    end

    def destination_encoding_name
      @destination_encoding.nil? ? nil : @destination_encoding.name
    end

    # A run cut off at the end of the text is incomplete rather than wrong.
    # An error built by hand says nothing either way.
    def incomplete_input?
      @truncated
    end
  end

  class Converter
    INVALID_MASK = 0x0f
    INVALID_REPLACE = 0x02
    UNDEF_MASK = 0xf0
    UNDEF_REPLACE = 0x20
    UNDEF_HEX_CHARREF = 0x30
    PARTIAL_INPUT = 0x10000
    AFTER_OUTPUT = 0x20000
    UNIVERSAL_NEWLINE_DECORATOR = 0x100
    CRLF_NEWLINE_DECORATOR = 0x1000
    CR_NEWLINE_DECORATOR = 0x2000
    XML_TEXT_DECORATOR = 0x4000
    XML_ATTR_CONTENT_DECORATOR = 0x8000
    XML_ATTR_QUOTE_DECORATOR = 0x10000

    def initialize source, destination, options = 0
      @source = Encoding::Converter.named source
      @destination = Encoding::Converter.named destination
      if @source == @destination
        raise Encoding::ConverterNotFoundError,
              "code converter not found (#{@source.name} to #{@destination.name})"
      end
      @options = options
      @convpath = Encoding::Converter.search_convpath @source, @destination, options
      @replacement = Encoding::Converter.replacement_for @destination, options
    end

    # What stands in for a character the destination cannot spell. UTF-8 has
    # a character of its own for that, and everything else uses a question
    # mark.
    def self.replacement_for destination, options
      standard = if destination == Encoding::UTF_8
        "\u{fffd}"
      else
        "?".force_encoding Encoding::US_ASCII
      end
      return standard unless options.is_a? Hash
      return standard unless options.key? :replace
      held = options[:replace]
      return standard if held.nil?
      return held if held.is_a? String
      unless held.respond_to? :to_str
        raise TypeError, "no implicit conversion of #{held.class} into String"
      end
      held = held.to_str
      unless held.is_a? String
        raise TypeError, "can't convert #{held.class} to String"
      end
      held
    end

    def source_encoding
      @source
    end

    def destination_encoding
      @destination
    end

    def convpath
      @convpath
    end

    def options
      @options.is_a?(Integer) ? @options : 0
    end

    def replacement
      @replacement
    end

    def replacement= held
      unless held.is_a? String
        raise TypeError, "no implicit conversion of #{held.class} into String"
      end
      # A replacement the destination cannot spell is refused, and the one
      # already in use stays.
      held.each_char do |character|
        next if spellable? character
        spelled = "U+" + character.ord.to_s(16).upcase.rjust(4, "0")
        from, to = stage_for :undefined
        raise Encoding::UndefinedConversionError.new(
          "#{spelled} #{undefined_path}", from, to, character
        )
      end
      @replacement = held
    end

    # Whether the conversion was told to stand a replacement in for what the
    # destination cannot spell.
    def replacing_undefined?
      return (@options & UNDEF_MASK) == UNDEF_REPLACE if @options.is_a? Integer
      return false unless @options.is_a? Hash
      @options[:undef] == :replace
    end
    private :replacing_undefined?

    def inspect
      "#<Encoding::Converter: #{@source.name} to #{@destination.name}>"
    end

    def to_s
      inspect
    end

    # The encoding a name stands for, which may be written as an Encoding, a
    # String, or anything that reads as one.
    def self.named held
      name = if held.is_a? String
        held
      elsif held.respond_to? :to_str
        held.to_str
      elsif held.respond_to? :name
        held.name
      else
        raise TypeError, "no implicit conversion of #{held.class} into String"
      end
      found = Encoding.find name
      if found.nil?
        raise Encoding::ConverterNotFoundError, "code converter not found (#{name})"
      end
      found
    end

    # The steps a conversion takes. Anything that is not already UTF-8 on one
    # side passes through UTF-8 on the way.
    def self.search_convpath source, destination, options = 0
      from = Encoding::Converter.named source
      to = Encoding::Converter.named destination
      if from.name == "ASCII-8BIT" && to.name != "ASCII-8BIT"
        raise Encoding::ConverterNotFoundError,
              "code converter not found (#{from.name} to #{to.name})"
      end
      path = if from == to || from == Encoding::UTF_8 || to == Encoding::UTF_8
        [[from, to]]
      else
        [[from, Encoding::UTF_8], [Encoding::UTF_8, to]]
      end
      path = path + ["crlf_newline"] if Encoding::Converter.crlf_wanted options
      path
    end

    def self.crlf_wanted options
      return false unless options.is_a? Hash
      options[:crlf_newline] ? true : false
    end

    # Carry text from the source encoding to the destination, refusing what
    # neither one can spell.
    def convert text
      held = text.to_s
      refuse_invalid held
      unless @pending.nil? || @pending.empty?
        held = held.byteslice(0, held.bytesize - @pending.bytesize)
      end
      converted = ""
      held.dup.force_encoding(@source.name).each_char do |character|
        refuse_undefined character unless spellable? character
        converted = converted + character
      end
      @errinfo = [:source_buffer_empty, nil, nil, nil, nil]
      @last_error = nil
      converted.dup.force_encoding @destination.name
    end

    # What went wrong the last time text was carried over, or nil when the
    # last attempt made it through.
    def last_error
      @last_error
    end

    # Carry what it can from `source` into `destination`, reporting how it
    # stopped rather than raising. The source is left holding whatever was
    # not read.
    def primitive_convert source, destination, destination_byteoffset = nil, destination_bytesize = nil, options = 0
      unless destination_byteoffset.nil?
        destination.replace destination.byteslice(0, destination_byteoffset)
      end
      held = source.dup.force_encoding "ASCII-8BIT"
      trouble = invalid_run held
      readable = trouble.nil? ? held : held.byteslice(0, trouble[0])
      written = ""
      consumed = 0
      status = nil
      readable.dup.force_encoding(@source.name).each_char do |character|
        if !destination_bytesize.nil? && written.bytesize + character.bytesize > destination_bytesize
          @errinfo = [:destination_buffer_full, nil, nil, nil, nil]
          @last_error = nil
          status = :destination_buffer_full
          break
        end
        unless spellable? character
          if replacing_undefined?
            written = written + @replacement
            consumed = consumed + character.bytesize
            next
          end
          from, to = stage_for :undefined
          bytes = character.dup.force_encoding("ASCII-8BIT")
          spelled = "U+" + character.ord.to_s(16).upcase.rjust(4, "0")
          @errinfo = [:undefined_conversion, from.name, to.name, bytes, ""]
          @last_error = Encoding::UndefinedConversionError.new(
            "#{spelled} from #{from.name} to #{to.name}", from, to, character
          )
          status = :undefined_conversion
          break
        end
        written = written + character
        consumed = consumed + character.bytesize
      end
      destination.replace destination + written.dup.force_encoding(@destination.name)
      if status.nil? && !trouble.nil?
        from, to = stage_for :invalid
        wrong = trouble[1]
        rest = trouble[2]
        truncated = trouble[3]
        status = truncated ? :incomplete_input : :invalid_byte_sequence
        @errinfo = [status, from.name, to.name, wrong, rest]
        @last_error = Encoding::InvalidByteSequenceError.new(
          "#{wrong.inspect} on #{from.name}", from, to, wrong, rest, truncated
        )
        # The bytes that could not carry on are read too, and held for a
        # `putback` to hand to the next piece of text.
        consumed = trouble[0] + wrong.bytesize + rest.bytesize
      end
      if status.nil?
        status = partial_input_wanted(options) ? :source_buffer_empty : :finished
        @errinfo = [status, nil, nil, nil, nil]
        @last_error = nil
      end
      source.replace held.byteslice(consumed, held.bytesize - consumed)
      status
    end

    # Whether the caller said more text is still to come, which leaves the
    # conversion open rather than finishing it.
    def partial_input_wanted options
      return false unless options.is_a? Hash
      options[:partial_input] ? true : false
    end
    private :partial_input_wanted


    # Close the conversion, reporting a character the text stopped part-way
    # through. There is nothing more to carry over once it has been called.
    def finish
      pending = @pending
      @pending = nil
      unless pending.nil? || pending.empty?
        from, to = stage_for :invalid
        @errinfo = [:incomplete_input, from.name, to.name, pending, ""]
        trouble = Encoding::InvalidByteSequenceError.new(
          "#{pending.inspect} on #{from.name}", from, to, pending, "", true
        )
        @last_error = trouble
        raise trouble
      end
      "".dup.force_encoding @destination.name
    end

    # The bytes held back after a run the source encoding could not read,
    # which the caller may put in front of the next piece of text. Reading
    # them takes them, so a second call answers nothing.
    def putback count = nil
      held = @errinfo.nil? ? "" : @errinfo[4]
      held = "" if held.nil?
      wanted = count.nil? ? held.bytesize : count
      taken = held.byteslice(held.bytesize - wanted, wanted)
      taken = "" if taken.nil?
      unless @errinfo.nil?
        @errinfo = @errinfo.dup
        @errinfo[4] = held.byteslice(0, held.bytesize - taken.bytesize)
      end
      taken.dup.force_encoding @source.name
    end

    # What the last conversion ran into, as the tuple Ruby reports.
    def primitive_errinfo
      @errinfo.nil? ? [:source_buffer_empty, nil, nil, nil, nil] : @errinfo
    end

    # Whether the destination can spell a character at all. An encoding that
    # covers only the ASCII letters spells nothing above them, and one that
    # covers a single byte spells nothing wider.
    def spellable? character
      code = character.ord
      case @destination.name
      when "UTF-8", "UTF-16", "UTF-16BE", "UTF-16LE", "UTF-32", "UTF-32BE", "UTF-32LE", "CESU-8", "GB18030"
        true
      when "ISO-8859-1"
        code < 256
      else
        code < 128
      end
    end
    private :spellable?

    # The run of bytes the source encoding cannot read, as
    # `[offset, wrong, rest, truncated]`, or nil when every byte reads.
    def invalid_run held
      return nil if @source.name == "ASCII-8BIT"
      return nil if held.dup.force_encoding(@source.name).valid_encoding?
      # The bytes are cut apart rather than joined onto text, since joining
      # would read each one as the character it spells.
      bytes = held.bytes
      start = 0
      while start < bytes.length
        break unless bytes[0..start].pack("C*").force_encoding(@source.name).valid_encoding?
        start = start + 1
      end
      start = bytes.length - 1 if start >= bytes.length
      # The run that opened the character, and the one byte after it that
      # could not carry on.
      stop = start + 1
      stop = stop + 1 while stop < bytes.length && carries_on?(bytes[stop])
      wrong = bytes[start..(stop - 1)].pack("C*").force_encoding "ASCII-8BIT"
      rest = stop < bytes.length ? [bytes[stop]].pack("C").force_encoding("ASCII-8BIT") : ""
      [start, wrong, rest, stop >= bytes.length]
    end
    private :invalid_run

    # A run of bytes the source encoding cannot read is refused before any
    # of it is carried over.
    def refuse_invalid held
      found = invalid_run held
      return if found.nil?
      # A character the text stops part-way through is held back, since more
      # of it may still arrive. `finish` is where that is reported.
      if found[3]
        @pending = found[1]
        return
      end
      wrong = found[1]
      rest = found[2]
      truncated = found[3]
      from, to = stage_for :invalid
      @errinfo = [:invalid_byte_sequence, from.name, to.name, wrong, rest]
      trouble = Encoding::InvalidByteSequenceError.new(
        "#{wrong.inspect} on #{from.name}", from, to, wrong, rest, truncated
      )
      @last_error = trouble
      raise trouble
    end
    private :refuse_invalid

    def refuse_undefined character
      spelled = "U+" + character.ord.to_s(16).upcase.rjust(4, "0")
      from, to = stage_for :undefined
      # A conversion that passes through another encoding refuses the
      # character at that step, so the character is reported as that step
      # reads it rather than as the text was read.
      refused = character.dup.force_encoding from.name
      @errinfo = [:undefined_conversion, from.name, to.name, refused, ""]
      trouble = Encoding::UndefinedConversionError.new(
        "#{spelled} #{undefined_path}", from, to, refused
      )
      @last_error = trouble
      raise trouble
    end

    # The steps the message names, which is every encoding the conversion
    # passes through rather than only the step that refused the character.
    def undefined_path
      steps = @convpath.select { |step| step.is_a? Array }
      return "from #{@source.name} to #{@destination.name}" if steps.empty?
      names = [steps.first[0].name]
      steps.each { |step| names.push step[1].name }
      "from " + names.join(" to ")
    end
    private :undefined_path
    private :refuse_undefined

    # Whether a byte carries on the character the one before it opened.
    def carries_on? byte
      return byte >= 0xa1 && byte <= 0xfe if @source.name == "EUC-JP"
      byte >= 0x80 && byte <= 0xbf
    end
    private :carries_on?

    # The step of the conversion the trouble belongs to. Bytes the source
    # cannot read stop the first step, while a character the destination
    # cannot spell stops the last one.
    def stage_for kind
      steps = @convpath.select { |step| step.is_a? Array }
      return [@source, @destination] if steps.empty?
      kind == :invalid ? steps.first : steps.last
    end
    private :stage_for

    # The ASCII-compatible encoding that stands in for one that is not, and
    # nil for one that already is.
    def self.asciicompat_encoding held
      found = begin
        Encoding::Converter.named held
      rescue Encoding::ConverterNotFoundError, ArgumentError
        nil
      end
      return nil if found.nil?
      return nil if found.ascii_compatible?
      return Encoding.find "stateless-ISO-2022-JP" if found.name.start_with? "ISO-2022-JP"
      Encoding::UTF_8
    end
  end
end

class IO
  # `putc` writes one character: the first of a String, or the low byte of a
  # number. It answers what it was given rather than what it wrote.
  def putc(held)
    if held.is_a? String
      write held[0]
      return held
    end
    number = if held.is_a? Integer
      held
    elsif held.respond_to? :to_int
      held.to_int
    else
      raise TypeError, "no implicit conversion of #{held.class} into Integer"
    end
    write (number & 0xFF).chr
    held
  end
end

class IO
  # A run of bytes a program reads and writes directly. A buffer either holds
  # memory of its own or stands over a String or a file, and a slice of one
  # shares the bytes it was cut from.
  class Buffer
    PAGE_SIZE = 4096
    DEFAULT_SIZE = 65536

    EXTERNAL = 1
    INTERNAL = 2
    MAPPED = 4
    SHARED = 8
    LOCKED = 32
    PRIVATE = 64
    READONLY = 128

    class LockedError < RuntimeError
    end

    class AllocationError < RuntimeError
    end

    class AccessError < RuntimeError
    end

    class InvalidatedError < RuntimeError
    end

    class MaskError < ArgumentError
    end

    # A size or an offset arrives as an Integer and nothing else.
    def self.whole_number(held)
      raise TypeError, "not an Integer" unless held.is_a? Integer
      if held > 9223372036854775807 || held < -9223372036854775808
        raise RangeError, "bignum too big to convert into `long'"
      end
      held
    end

    # The text a run of bytes stands for. Bytes that spell characters in UTF-8
    # read back as those characters, and bytes that spell nothing stand for
    # themselves.
    def self.text_of(bytes)
      return "".b if bytes.empty?
      bytes.pack "C*"
    end

    # Without flags the buffer picks where its bytes live by how many there
    # are. Flags that name neither place leave it nowhere to put them.
    def initialize(size = DEFAULT_SIZE, flags = nil)
      size = IO::Buffer.whole_number size
      flags = IO::Buffer.whole_number(flags) unless flags.nil?
      raise ArgumentError, "Size can't be negative!" if size < 0
      raise ArgumentError, "Flags can't be negative!" if !flags.nil? && flags < 0
      @locked = false
      @source = nil
      @string_backed = false
      if size == 0
        nullify
        return
      end
      kind = if flags.nil?
        size < PAGE_SIZE ? INTERNAL : MAPPED
      elsif (flags & MAPPED) != 0
        MAPPED
      elsif (flags & INTERNAL) != 0
        INTERNAL
      else
        raise AllocationError, "Could not allocate buffer!"
      end
      @text = "\0" * size
      @offset = 0
      @size = size
      @flags = kind | ((flags || 0) & (SHARED | PRIVATE | READONLY))
    end

    # A buffer over a String. Without a block the bytes are copied and the
    # copy is read only, and with one the String itself is written through
    # and left alone until the block ends.
    def self.for(string)
      unless block_given?
        made = IO::Buffer.new 0
        made.send :adopt_string, string.bytes.pack("C*"), EXTERNAL | READONLY
        return made
      end
      made = IO::Buffer.new 0
      made.send :adopt_string, string, EXTERNAL | (string.frozen? ? READONLY : 0)
      string.__borrow__ unless string.frozen?
      begin
        yield made
      ensure
        string.__release__ unless string.frozen?
        made.free unless made.null?
      end
    end

    # A buffer over a String of the size asked for, which is answered once the
    # block is done with it.
    def self.string(length)
      raise LocalJumpError, "no block given" unless block_given?
      length = IO::Buffer.whole_number length
      raise ArgumentError, "negative string size (or size too big)" if length < 0
      held = "\0" * length
      made = IO::Buffer.new 0
      made.send :adopt_string, held, EXTERNAL
      begin
        yield made
      ensure
        made.free unless made.null?
      end
      held
    end

    # A buffer over what a file holds.
    def self.map(file, size = nil, offset = 0, flags = 0)
      offset = IO::Buffer.whole_number offset
      flags = IO::Buffer.whole_number flags
      raise ArgumentError, "Offset can't be negative!" if offset < 0
      content = File.read(file.path).b
      whole = content.bytesize
      raise ArgumentError, "Invalid negative or zero file size!" if whole == 0
      unless size.nil?
        size = IO::Buffer.whole_number size
        raise ArgumentError, "Size can't be negative!" if size < 0
        raise ArgumentError, "Size can't be zero!" if size == 0
        raise ArgumentError, "Size can't be larger than file size!" if size > whole
        raise ArgumentError, "Offset too large!" if offset + size > whole
      end
      size = whole - offset if size.nil?
      bytes = content.bytes[offset, size] || []
      made = IO::Buffer.new 0
      kind = if (flags & PRIVATE) != 0
        MAPPED | PRIVATE
      else
        MAPPED | EXTERNAL | SHARED
      end
      made.send :adopt_string, IO::Buffer.text_of(bytes), kind | (flags & READONLY)
      made.send :follow_file, file, offset if (flags & PRIVATE) == 0
      made
    end

    def size
      @size
    end

    def empty?
      @size == 0
    end

    def null?
      @text.nil?
    end

    def external?
      !null? && (@flags & EXTERNAL) != 0
    end

    def internal?
      !null? && (@flags & INTERNAL) != 0
    end

    def mapped?
      !null? && (@flags & MAPPED) != 0
    end

    def shared?
      !null? && (@flags & SHARED) != 0
    end

    def private?
      !null? && (@flags & PRIVATE) != 0
    end

    def readonly?
      !null? && (@flags & READONLY) != 0
    end

    def locked?
      @locked == true
    end

    # A buffer under a lock refuses every change to itself, though what it
    # holds is still read and written.
    def locked
      raise LockedError, "Buffer already locked!" if @locked
      @locked = true
      begin
        yield
      ensure
        @locked = false
      end
    end

    # A slice is valid while what it was cut from is still there and still
    # covers it.
    def valid?
      return true if @source.nil?
      return true if @source_string_backed
      return false if @source.null?
      return false unless @source.send(:holds_text?, @source_text)
      @offset + @size <= @source.send(:end_offset)
    end

    def free
      raise LockedError, "Buffer is locked!" if @locked
      nullify
      self
    end

    def transfer
      raise LockedError, "Cannot transfer ownership of locked buffer!" if @locked
      made = IO::Buffer.new 0
      made.send :adopt, @text, @offset, @size, @flags, @source, @source_text,
                @source_string_backed, @file, @file_offset
      nullify
      made
    end

    def slice(at = 0, length = nil)
      ensure_valid
      at = IO::Buffer.whole_number at
      length = length.nil? ? @size - at : IO::Buffer.whole_number(length)
      made = IO::Buffer.new 0
      made.send :adopt, @text, @offset + at, length, @flags & ~READONLY, self,
                @text, @string_backed, nil, 0
      made
    end

    def resize(size)
      raise LockedError, "Cannot resize locked buffer!" if @locked
      size = IO::Buffer.whole_number size
      raise ArgumentError, "Size can't be negative!" if size < 0
      raise AccessError, "Cannot resize external buffer!" if external?
      if size == 0
        nullify
        return self
      end
      held = null? ? [] : byte_view
      made = Array.new size, 0
      counted = size < held.length ? size : held.length
      counted.times { |at| made[at] = held[at] }
      kind = if null?
        size < PAGE_SIZE ? INTERNAL : MAPPED
      elsif mapped? && !private?
        INTERNAL
      else
        @flags & (INTERNAL | MAPPED)
      end
      @text = IO::Buffer.text_of made
      @offset = 0
      @size = size
      @flags = kind | (@flags & (SHARED | PRIVATE | READONLY))
      self
    end

    def get_string(at = 0, length = nil, encoding = nil)
      ensure_valid
      at = IO::Buffer.whole_number at
      length = length.nil? ? @size - at : IO::Buffer.whole_number(length)
      taken = byte_view[at, length] || []
      IO::Buffer.text_of taken
    end

    def set_string(text, at = 0, length = nil, source_offset = 0)
      ensure_valid
      raise AccessError, "Buffer is not writable!" if readonly?
      at = IO::Buffer.whole_number at
      source_offset = IO::Buffer.whole_number source_offset
      given = text.bytes
      given = given[source_offset..-1] || []
      given = given[0, length] || [] unless length.nil?
      room = @size - at
      given = given[0, room] || [] if given.length > room
      set_bytes given, at
      given.length
    end

    def clear(value = 0, at = 0, length = nil)
      ensure_valid
      raise AccessError, "Buffer is not writable!" if readonly?
      length = length.nil? ? @size - at : IO::Buffer.whole_number(length)
      set_bytes Array.new(length, value), at
      self
    end

    def each(kind = :U8)
      held = byte_view
      unless block_given?
        made = []
        at = 0
        while at < held.length
          made.push [at, held[at]]
          at = at + 1
        end
        return made.each
      end
      at = 0
      while at < held.length
        yield at, held[at]
        at = at + 1
      end
      self
    end

    def &(mask)
      combined mask, :and, false
    end

    def |(mask)
      combined mask, :or, false
    end

    def ^(mask)
      combined mask, :xor, false
    end

    def ~
      combined nil, :invert, false
    end

    def and!(mask)
      combined mask, :and, true
    end

    def or!(mask)
      combined mask, :or, true
    end

    def xor!(mask)
      combined mask, :xor, true
    end

    def not!
      combined nil, :invert, true
    end

    def to_s
      return "#<IO::Buffer 0x0000000000000000 +0 0 NULL>" if null?
      "#<IO::Buffer 0x%016x +%d %d %s>" % [@text.object_id, @offset, @size, flag_names]
    end

    def inspect
      to_s
    end

    def hexdump(at = 0, length = nil, width = 16)
      taken = byte_view[at, length.nil? ? @size - at : length] || []
      taken.map { |value| "%02x" % value }.join(" ")
    end

    # ── What a buffer keeps to itself ────────────────────────────────────────

    def adopt(text, offset, size, flags, source, source_text, source_string_backed, file, file_offset)
      @text = text
      @offset = offset
      @size = size
      @flags = flags
      @source = source
      @source_text = source_text
      @source_string_backed = source_string_backed
      @string_backed = source_string_backed
      @file = file
      @file_offset = file_offset
      @locked = false
      self
    end

    def adopt_string(text, flags)
      @text = text
      @offset = 0
      @size = text.bytesize
      @flags = flags
      @source = nil
      @source_text = nil
      @source_string_backed = false
      @string_backed = true
      @locked = false
      self
    end

    def follow_file(file, offset)
      @file = file
      @file_offset = offset
      @string_backed = false
      self
    end

    def nullify
      @text = nil
      @offset = 0
      @size = 0
      @flags = 0
      @file = nil
      @file_offset = 0
    end

    def holds_text?(other)
      !@text.nil? && @text.equal?(other)
    end

    def end_offset
      @offset + @size
    end

    def byte_view
      return [] if @text.nil?
      @text.bytes[@offset, @size] || []
    end

    def ensure_valid
      raise InvalidatedError, "Buffer has been invalidated!" unless valid?
    end

    def set_bytes(values, at = 0)
      held = @text.bytes
      values.each_with_index do |value, step|
        place = @offset + at + step
        held[place] = value & 0xff if place < @offset + @size
      end
      made = IO::Buffer.text_of held
      # The buffer is allowed to write through the very lock it put on the
      # string it stands over.
      @text.__release__
      @text.replace made
      @text.__borrow__ if @string_backed && !@source.nil? == false && borrowing?
      write_through
      self
    end

    def borrowing?
      false
    end

    def write_through
      return if @file.nil?
      return unless shared?
      File.write @file.path, IO::Buffer.text_of(@text.bytes)
    end

    def flag_names
      named = []
      named << "EXTERNAL" if external?
      named << "INTERNAL" if internal?
      named << "MAPPED" if mapped?
      named << "SHARED" if shared?
      named << "PRIVATE" if private?
      named << "READONLY" if readonly?
      named.empty? ? "NULL" : named.join("|")
    end

    def combined(mask, how, in_place)
      ensure_valid
      values = byte_view
      made = if how == :invert
        values.map { |value| ~value & 0xff }
      else
        unless mask.is_a? IO::Buffer
          named = mask.nil? ? "nil" : mask.class.to_s
          raise TypeError, "wrong argument type #{named} (expected IO::Buffer)"
        end
        over = mask.send :byte_view
        raise MaskError, "Zero-length mask given!" if over.empty?
        values.each_with_index.map do |value, at|
          other = over[at % over.length]
          if how == :and
            value & other
          elsif how == :or
            value | other
          else
            value ^ other
          end
        end
      end
      if in_place
        set_bytes made
        return self
      end
      answer = IO::Buffer.new made.length, INTERNAL
      answer.send :set_bytes, made
      answer
    end

    private :adopt, :adopt_string, :follow_file, :nullify, :holds_text?,
            :end_offset, :byte_view, :ensure_valid, :set_bytes, :borrowing?,
            :write_through, :flag_names, :combined
  end
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
        self.send_to_object(enumerator_class, "over", arguments, position)
    }

    /// A walk whose count is named by an object rather than a plain Integer,
    /// which is how an endless walk reports a size of Infinity.
    pub(crate) fn build_enumerator_of_size(
        &mut self,
        receiver: crate::object::Object,
        method_name: &str,
        arguments: Vec<crate::object::Object>,
        size: crate::object::Object,
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
            size,
        ];
        self.send_to_object(enumerator_class, "over", arguments, position)
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
