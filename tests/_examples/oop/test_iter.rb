class Counter
  def initialize
    @count = 0
  end

  def each
    self
  end

  def next
    "next"
  end
end

counter = Counter.new
iterator = counter.each
puts iterator.next
