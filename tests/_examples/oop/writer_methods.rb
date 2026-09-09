# A writer method can name its parameter without parentheses. The `=` has to
# sit against the name; with a space before it, the definition is an endless
# one whose body is the expression on the right.

class Meter
  def initialize
    @reading = 0
  end

  def reading
    @reading
  end

  def reading= value
    @reading = value
  end

  def doubled = @reading * 2

  def scaled= factor
    @reading = @reading * factor
  end
end

meter = Meter.new
meter.reading = 12
p meter.reading
p meter.doubled

meter.scaled = 3
p meter.reading

class Register
  def self.total
    @total
  end

  def self.total= amount
    @total = amount
  end
end

Register.total = 99
p Register.singleton_methods.sort

# A writer defined this way is named the way any other writer is.
p Meter.instance_methods(false).sort
