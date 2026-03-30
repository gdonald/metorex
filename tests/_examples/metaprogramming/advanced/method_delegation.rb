# Method Delegation: Forward method calls to another object
# Demonstrates define_method and dynamic dispatch

class Logger
  def log(message)
    puts "[LOG] #{message}"
  end
end

class Service
  def initialize
    @logger = Logger.new
  end

  def process(data)
    @logger.log("Processing: #{data}")
    "processed: #{data}"
  end
end

# Simple delegation: Service delegates logging to Logger
puts "=== Simple Delegation ==="
service = Service.new
puts service.process("input data")

# Dynamic delegation using define_method
class Delegator
  def initialize(target)
    @target = target
  end

  def delegate(method_name)
    @target
  end
end

puts ""
puts "=== Dynamic Delegation ==="
logger = Logger.new
d = Delegator.new(logger)
target = d.delegate("log")
target.log("delegated message")

# Forwarding pattern: wrap calls with before/after hooks
class LoggingProxy
  def initialize(target)
    @target = target
  end

  def call(method_name, arg)
    puts "Before: #{method_name}"
    result = @target.process(arg)
    puts "After: #{method_name} -> #{result}"
    result
  end
end

puts ""
puts "=== Forwarding with Hooks ==="
proxy = LoggingProxy.new(Service.new)
proxy.call("process", "test data")
