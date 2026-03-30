# Object-Oriented Design Patterns

# === Strategy Pattern ===
# Different sorting/formatting strategies

class UpperFormatter
  def format(text)
    text.upcase
  end
end

class LowerFormatter
  def format(text)
    text.downcase
  end
end

class Printer
  def initialize(formatter)
    @formatter = formatter
  end

  def print_formatted(text)
    puts @formatter.format(text)
  end
end

puts "=== Strategy Pattern ==="
upper = Printer.new(UpperFormatter.new)
lower = Printer.new(LowerFormatter.new)
upper.print_formatted("Hello World")
lower.print_formatted("Hello World")

# === Template Method Pattern ===
# Base class defines algorithm skeleton, subclasses fill in steps

class Report
  def generate
    puts self.header
    puts self.body
    puts self.footer
  end

  def header
    "--- Report ---"
  end

  def footer
    "--- End ---"
  end
end

class SalesReport < Report
  def body
    "Total sales: $1000"
  end
end

class InventoryReport < Report
  def body
    "Items in stock: 42"
  end

  def header
    "*** Inventory ***"
  end

  def footer
    "*** End ***"
  end
end

puts ""
puts "=== Template Method Pattern ==="
SalesReport.new.generate
puts ""
InventoryReport.new.generate

# === Observer Pattern ===
# Objects subscribe to events and get notified

class EventEmitter
  def initialize
    @listeners = []
  end

  def on(&block)
    @listeners.push(block)
  end

  def emit(data)
    @listeners.each do |listener|
      listener.call(data)
    end
  end
end

puts ""
puts "=== Observer Pattern ==="
emitter = EventEmitter.new
emitter.on do |data|
  puts "Listener 1: #{data}"
end
emitter.on do |data|
  puts "Listener 2: #{data}"
end
emitter.emit("event fired")

# === Decorator Pattern ===
# Wrap objects to add behavior

class BasicCoffee
  def cost
    5
  end

  def description
    "Basic coffee"
  end
end

class MilkDecorator
  def initialize(coffee)
    @coffee = coffee
  end

  def cost
    @coffee.cost + 2
  end

  def description
    "#{@coffee.description} + milk"
  end
end

class SugarDecorator
  def initialize(coffee)
    @coffee = coffee
  end

  def cost
    @coffee.cost + 1
  end

  def description
    "#{@coffee.description} + sugar"
  end
end

puts ""
puts "=== Decorator Pattern ==="
coffee = BasicCoffee.new
puts "#{coffee.description}: $#{coffee.cost}"

coffee = MilkDecorator.new(coffee)
puts "#{coffee.description}: $#{coffee.cost}"

coffee = SugarDecorator.new(coffee)
puts "#{coffee.description}: $#{coffee.cost}"
