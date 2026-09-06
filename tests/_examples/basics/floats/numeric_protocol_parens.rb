# Numeric carries the protocol every number answers, written in terms of the
# methods a subclass supplies. Here that is `<=>`, `to_f`, `-@`, and `/`.
class Money < Numeric
  attr_reader :cents

  def initialize(cents)
    @cents = cents
  end

  def <=>(other)
    return cents <=> other.cents if other.is_a?(Money)
    to_f <=> other
  end

  def ==(other)
    other.is_a?(Money) && cents == other.cents
  end

  def -@
    Money.new(-cents)
  end

  def to_f
    cents / 100.0
  end

  def /(other)
    Money.new(cents / other)
  end
end

owed = Money.new(-250)
p(owed.negative?)
p(owed.positive?)
p(owed.zero?)
p(owed.abs.cents)
p(owed.nonzero?.cents)
p(Money.new(0).nonzero?.nil?)

# The rounding methods read the number through `to_f`.
p(Money.new(250).ceil)
p(Money.new(255).floor)
p(Money.new(255).round)
p(Money.new(-255).truncate)

# `div` floors the quotient `/` answers, and `divmod` pairs it with what is
# left over.
p(Money.new(500).div(2))
p(owed.integer?)
p(owed.real?)
p(owed.real.cents)

# `coerce` answers two of the same class, or two Floats when they differ.
p(Money.new(100).coerce(Money.new(200)).map { |m| m.cents })
p(Money.new(100).coerce(3))

# `eql?` is equality without conversion, so a different class is never equal.
p(Money.new(100).eql?(Money.new(100)))
p(Money.new(100).eql?(1.0))
