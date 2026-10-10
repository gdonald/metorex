class Counter
  def self.cache = (@@cache ||= {hits: 0})
end
p Counter.cache
Limit ||= 10
p Limit

class Account
  def initialize owner
    @owner = owner
  end
  def label = (if @name = @owner&.upcase then @name else "unknown" end)
end
p Account.new("ada").label
p Account.new(nil).label

continue = :ok
p continue

class Order
  %i[id total].each { |field| attr_accessor field }
  %i[status note].each(&method(:attr_reader))
end
order = Order.new
order.total = 42
p [order.total, Order.public_method_defined?(:status)]

def bucket amount
  case amount
  when 0...10 then :small
  when 10..Float::INFINITY then :large
  end
end
p [bucket(3), bucket(400)]
p case "m" when "a".."f" then :first when "g".."z" then :second end
