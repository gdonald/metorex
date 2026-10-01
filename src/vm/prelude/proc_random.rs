pub(super) const SOURCE: &str = r##"
class Binding
  # A session that reads Ruby from standard input and runs it here, with
  # this binding's locals in scope, until `exit` or the end of the input.
  def irb
    require "irb"
    IRB.run_session(self, caller(1))
  end
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
  # The composition of two callables, made for a `Method` whose `>>` and `<<`
  # hand the work here. `forward` runs `first` before `second`, and `strict`
  # says whether the composition answers to `lambda?`.
  def self.__composed__(first, second, forward, strict)
    raise TypeError, "callable object is expected" unless second.respond_to?(:call)
    if forward
      return lambda { |*given, &block| second.call(first.call(*given, &block)) } if strict
      proc { |*given, &block| second.call(first.call(*given, &block)) }
    else
      return lambda { |*given, &block| first.call(second.call(*given, &block)) } if strict
      proc { |*given, &block| first.call(second.call(*given, &block)) }
    end
  end

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

  # Two generators are equal when they are of one class, hold the same words
  # at the same place, and were seeded the same way.
  def == other
    return false unless other.instance_of?(self.class)
    words, left, seed = other.__send__(:marshal_dump)
    words == __words__ && left == N - @index + 1 && seed == @seed
  end

  # What Marshal writes for a generator, the way Ruby writes it: every word
  # it holds as one number with the first word lowest, how many words are
  # left before it makes new ones, and its seed.
  def marshal_dump
    [__words__, N - @index + 1, @seed]
  end
  private :marshal_dump

  def marshal_load(dumped)
    words, left, seed = dumped
    raise ArgumentError, "wrong value" if left > N
    @state = Array.new(N) { |at| (words >> (32 * at)) & MASK32 }
    @index = N - left + 1
    @seed = seed
    self
  end
  private :marshal_load

  def __words__
    @state.each_with_index.inject(0) { |held, (word, at)| held | (word << (32 * at)) }
  end
  private :__words__

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
    return (next_real * held).floor if held > 18446744073709551616
    limited_number held - 1
  end

  # A whole number no greater than the top, drawn the way Ruby draws one: a
  # mask wide enough for the top, filled a word at a time, and drawn again
  # whenever the value lands past it.
  def limited_number(top)
    return 0 if top <= 0
    mask = 1
    mask = (mask << 1) | 1 while mask < top
    loop do
      value = 0
      landed = true
      place = 1
      while place >= 0
        if ((mask >> (place * 32)) & MASK32) != 0
          value |= next_word << (place * 32)
          value &= mask
          if top < value
            landed = false
            break
          end
        end
        place -= 1
      end
      return value if landed
    end
  end
  private :limited_number

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
"##;
