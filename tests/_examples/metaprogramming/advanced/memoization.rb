# Memoization: Cache function results to avoid redundant computation
# Demonstrates: closures, hashes as caches, higher-order functions

# Manual memoization using a hash cache
class MemoizedFibonacci
  def initialize
    @cache = {}
  end

  def compute(n)
    if @cache.has_key?(n)
      return @cache[n]
    end

    result = nil
    if n <= 1
      result = n
    else
      result = self.compute(n - 1) + self.compute(n - 2)
    end

    @cache[n] = result
    result
  end

  def cache_size
    @cache.size
  end
end

puts "=== Memoized Fibonacci ==="
fib = MemoizedFibonacci.new

for n in 0..10
  puts "fib(#{n}) = #{fib.compute(n)}"
end
puts "Cache entries: #{fib.cache_size}"

# Generic memoizer using blocks
class Memoizer
  def initialize(&block)
    @func = block
    @cache = {}
  end

  def call(arg)
    if @cache.has_key?(arg)
      return @cache[arg]
    end
    result = @func.call(arg)
    @cache[arg] = result
    result
  end

  def cache_size
    @cache.size
  end
end

puts ""
puts "=== Generic Memoizer ==="

square = Memoizer.new do |n|
  puts "  (computing #{n} * #{n})"
  n * n
end

puts "First calls:"
puts square.call(5)
puts square.call(3)
puts square.call(5)

puts "Second calls (cached):"
puts square.call(5)
puts square.call(3)
puts "Cache size: #{square.cache_size}"
