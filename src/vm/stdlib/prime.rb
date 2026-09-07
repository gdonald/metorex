# The prime numbers, walked in order, and the factorization of an integer
# into the primes that multiply to it.
require 'singleton'

class Prime
  include Enumerable
  include Singleton

  def self.prime?(number)
    instance.prime?(number)
  end

  def self.each(bound = nil, &block)
    instance.each(bound, &block)
  end

  def self.prime_division(number)
    instance.prime_division(number)
  end

  def self.int_from_prime_division(factors)
    instance.int_from_prime_division(factors)
  end

  def self.take(count)
    instance.first(count)
  end

  def self.first(count)
    instance.first(count)
  end

  def prime?(number)
    return false unless number.is_a?(Integer)
    return false if number < 2
    return true if number < 4
    return false if number % 2 == 0
    divisor = 3
    while divisor * divisor <= number
      return false if number % divisor == 0
      divisor += 2
    end
    true
  end

  # The primes in order. Without a block the walk is lazy, so it may run as
  # far as the caller asks and no further.
  def each(bound = nil, &block)
    return lazy_walk(bound) if block.nil?
    found = 2
    loop do
      break if !bound.nil? && found > bound
      block.call(found)
      found = next_prime(found)
    end
  end

  def lazy_walk(bound)
    walk = Enumerator::Lazy.build(self, nil)
    return walk if bound.nil?
    walk.take_while { |found| found <= bound }
  end
  private :lazy_walk

  # The first prime after the one given.
  def next_prime(after)
    candidate = after + 1
    candidate += 1 until prime?(candidate)
    candidate
  end
  private :next_prime

  def prime_division(number)
    raise ZeroDivisionError, "divided by 0" if number == 0
    factors = []
    left = number
    if left < 0
      factors.push([-1, 1])
      left = -left
    end
    divisor = 2
    while divisor * divisor <= left
      count = 0
      while left % divisor == 0
        left = left / divisor
        count += 1
      end
      factors.push([divisor, count]) if count > 0
      divisor += divisor == 2 ? 1 : 2
    end
    factors.push([left, 1]) if left > 1
    factors
  end

  def int_from_prime_division(factors)
    factors.inject(1) { |total, pair| total * pair[0] ** pair[1] }
  end
end

class Integer
  def prime?
    Prime.prime?(self)
  end

  def prime_division
    Prime.prime_division(self)
  end

  def self.from_prime_division(factors)
    Prime.int_from_prime_division(factors)
  end

  def self.each_prime(bound, &block)
    Prime.each(bound, &block)
  end
end
