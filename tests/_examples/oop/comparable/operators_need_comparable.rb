# The ordering operators come from Comparable, built on `<=>`. An object
# whose class does not include Comparable has no `<`, whatever `<=>` it
# answers, and a Comparable whose `<=>` answers nil names both sides in the
# ArgumentError.
class Plain
  def <=>(_other)
    0
  end
end

class Ranked
  include Comparable

  def initialize(rank)
    @rank = rank
  end

  attr_reader :rank

  def <=>(other)
    other.is_a?(Ranked) ? rank <=> other.rank : nil
  end
end

begin
  Plain.new < Plain.new
rescue NoMethodError => error
  puts(error.message)
end

p(Ranked.new(1) < Ranked.new(2))

[nil, 2.5, :sym, "text", [1]].each do |other|
  begin
    Ranked.new(1) < other
  rescue ArgumentError => error
    puts(error.message)
  end
end

extended = Plain.new
extended.extend(Comparable)
p(extended <= Plain.new)
