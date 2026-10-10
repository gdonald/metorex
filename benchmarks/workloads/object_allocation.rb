# Objects made and dropped, each with two instance variables.
class Point
  def initialize(across, down)
    @across = across
    @down = down
  end

  attr_reader :across
end

total = 0
200_000.times { |step| total += Point.new(step, step).across }
puts total
