class Address
  def initialize(city)
    @city = city
  end
end

class Customer
  def initialize(name, city)
    @name = name
    @address = Address.new(city)
    @self_reference = self
  end
end

class Forwarder < BasicObject
  def initialize(target)
    @target = target
  end

  def method_missing(name, *arguments)
    @target.__send__(name, *arguments)
  end
end

puts(Customer.new("Ada", "Austin").inspect.gsub(/0x\h+/, "0x0"))
p([Address.new("Boise")].inspect.gsub(/0x\h+/, "0x0"))
p(Forwarder.new([1, 2]))
p([Forwarder.new(:on_ident)])
