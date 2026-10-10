# Numeric#coerce pairs two values of the same class as they are, and turns
# both into Floats otherwise. A value given a singleton class answers to that
# class, so it is no longer the same class as its sibling. A number refuses
# a singleton method, which is taken away again before the error is raised.
class Measure < Numeric
  def initialize(amount)
    @amount = amount
  end

  def to_f
    @amount.to_f
  end
end

first = Measure.new(2)
second = Measure.new(3)
p(first.coerce(second).map(&:to_f))
p(first.coerce(4))

begin
  def first.unit
    "meters"
  end
rescue TypeError => error
  puts(error.message)
end
p(first.coerce(second))

lone = Class.new(Numeric).new
lone.singleton_class
begin
  lone.coerce(lone.class.new)
rescue TypeError => error
  puts(error.message == "can't convert #{lone.class.inspect} into Float")
end
