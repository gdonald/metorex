# Calls of a method with two arguments on one receiver.
class Counter
  def initialize
    @total = 0
  end

  def add(amount, times)
    @total += amount * times
  end

  attr_reader :total
end

counter = Counter.new
200_000.times { |step| counter.add(step, 2) }
puts counter.total
