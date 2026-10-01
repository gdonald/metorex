# The prime numbers, walked in order by a generator, and the factorization
# of an integer into the primes that multiply to it.
require 'singleton'

class Integer
  def self.from_prime_division(factors)
    Prime.int_from_prime_division(factors)
  end

  def prime_division(generator = Prime::Generator23.new)
    Prime.prime_division(self, generator)
  end

  def prime?
    return self >= 2 if self <= 3
    return false if self % 2 == 0 || self % 3 == 0
    divisor = 5
    while divisor * divisor <= self
      return false if self % divisor == 0 || self % (divisor + 2) == 0
      divisor += 6
    end
    true
  end

  def self.each_prime(bound, &block)
    Prime.each(bound, &block)
  end
end

class Prime
  include Enumerable
  include Singleton

  # The class answers every method the one instance does, and walks the
  # primes as an Enumerable of its own.
  class << self
    include Enumerable

    def method_added(name)
      singleton_class.define_method(name) do |*arguments, &block|
        instance.public_send(name, *arguments, &block)
      end
    end
  end

  # Every prime up to `bound`, or every prime when there is none, from the
  # generator given. Without a block the generator itself is the answer.
  def each(bound = nil, generator = EratosthenesGenerator.new, &block)
    generator.upper_bound = bound
    generator.each(&block)
  end

  def include?(object)
    case object
    when Integer
      prime?(object)
    when Module
      Module.instance_method(:include?).bind(Prime).call(object)
    else
      false
    end
  end

  def prime?(value, generator = Prime::Generator23.new)
    raise ArgumentError, "Expected a prime generator, got #{generator}" unless generator.respond_to? :each
    raise ArgumentError, "Expected an integer, got #{value}" unless value.respond_to?(:integer?) && value.integer?
    return false if value < 2
    generator.each do |candidate|
      quotient, remainder = value.divmod candidate
      return true if quotient < candidate
      return false if remainder == 0
    end
  end

  def int_from_prime_division(factors)
    factors.inject(1) { |value, (prime, power)| value * prime**power }
  end

  def prime_division(value, generator = Prime::Generator23.new)
    raise ZeroDivisionError if value == 0
    if value < 0
      value = -value
      factors = [[-1, 1]]
    else
      factors = []
    end
    generator.each do |prime|
      count = 0
      while (quotient, remainder = value.divmod(prime)
             remainder) == 0
        value = quotient
        count += 1
      end
      factors.push [prime, count] if count != 0
      break if quotient <= prime
    end
    factors.push [value, 1] if value > 1
    factors
  end

  # What every generator shares: a walk up to an optional bound, built on
  # the `succ` each kind of generator answers.
  class PseudoPrimeGenerator
    include Enumerable

    def initialize(bound = nil)
      @ubound = bound
    end

    def upper_bound=(bound)
      @ubound = bound
    end

    def upper_bound
      @ubound
    end

    def succ
      raise NotImplementedError, "need to define `succ'"
    end

    def next
      raise NotImplementedError, "need to define `next'"
    end

    def rewind
      raise NotImplementedError, "need to define `rewind'"
    end

    # Each value in turn. Bounded, the walk answers what the block last
    # did; without a block, a copy of the generator is the answer, so a
    # walk it starts does not move this one.
    def each
      return dup unless block_given?
      if @ubound
        last_value = nil
        loop do
          prime = succ
          break last_value if prime > @ubound
          last_value = yield prime
        end
      else
        loop do
          yield succ
        end
      end
    end

    def with_index(offset = 0, &block)
      return enum_for(:with_index, offset) { Float::INFINITY } unless block
      return each_with_index(&block) if offset == 0
      each do |prime|
        yield prime, offset
        offset += 1
      end
    end

    def with_object(object)
      return enum_for(:with_object, object) { Float::INFINITY } unless block_given?
      each do |prime|
        yield prime, object
      end
    end

    def size
      Float::INFINITY
    end
  end

  # The primes read off a sieve shared by every generator of this kind.
  class EratosthenesGenerator < PseudoPrimeGenerator
    def initialize
      @last_prime_index = -1
      super
    end

    def succ
      @last_prime_index += 1
      EratosthenesSieve.instance.get_nth_prime(@last_prime_index)
    end

    def rewind
      initialize
    end

    alias next succ
  end

  # The primes found by dividing each candidate by the primes before it.
  class TrialDivisionGenerator < PseudoPrimeGenerator
    def initialize
      @index = -1
      super
    end

    def succ
      TrialDivision.instance[@index += 1]
    end

    def rewind
      initialize
    end

    alias next succ
  end

  # 2, 3, and every number after them that is not a multiple of either.
  # Some of these are not prime, which factoring tolerates.
  class Generator23 < PseudoPrimeGenerator
    def initialize
      @prime = 1
      @step = nil
      super
    end

    def succ
      if @step
        @prime += @step
        @step = 6 - @step
      else
        case @prime
        when 1 then @prime = 2
        when 2 then @prime = 3
        when 3
          @prime = 5
          @step = 2
        end
      end
      @prime
    end

    alias next succ

    def rewind
      initialize
    end
  end

  # The primes found so far by trial division, kept for every caller.
  class TrialDivision
    include Singleton

    def initialize
      @primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101]
      @next_to_check = 103
      @ulticheck_index = 3
      @ulticheck_next_squared = 121
    end

    def [](index)
      while index >= @primes.length
        if @next_to_check + 4 > @ulticheck_next_squared
          @ulticheck_index += 1
          @ulticheck_next_squared = @primes.at(@ulticheck_index + 1)**2
        end
        @primes.push @next_to_check if @primes[2..@ulticheck_index].find { |prime| @next_to_check % prime == 0 }.nil?
        @next_to_check += 4
        @primes.push @next_to_check if @primes[2..@ulticheck_index].find { |prime| @next_to_check % prime == 0 }.nil?
        @next_to_check += 2
      end
      @primes[index]
    end
  end

  # The primes a segmented sieve of Eratosthenes has found, extended a
  # segment at a time as a walk asks for more.
  class EratosthenesSieve
    include Singleton

    def initialize
      @primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101]
      @max_checked = @primes.last + 1
    end

    def get_nth_prime(index)
      compute_primes while @primes.size <= index
      @primes[index]
    end

    private

    def compute_primes
      max_segment_size = 1_000_000
      max_cached_prime = @primes.last
      @max_checked = max_cached_prime + 1 if max_cached_prime > @max_checked
      segment_min = @max_checked
      segment_max = [segment_min + max_segment_size, max_cached_prime * 2].min
      root = Integer.sqrt(segment_max)
      segment = ((segment_min + 1)..segment_max).step(2).to_a
      sieving = 1
      loop do
        prime = @primes[sieving]
        break if prime > root
        composite_index = (-(segment_min + 1 + prime) / 2) % prime
        while composite_index < segment.size
          segment[composite_index] = nil
          composite_index += prime
        end
        sieving += 1
      end
      @primes.concat(segment.compact)
      @max_checked = segment_max
    end
  end
end
