class Target
  def ping
    "pong"
  end
end

class Forwarder
  def initialize(actual)
    @actual = actual
  end

  def method_missing(name, *args, &block)
    @actual.__send__(name, *args, &block)
  end
end

t = Target.new
f = Forwarder.new(t)
puts f.ping
