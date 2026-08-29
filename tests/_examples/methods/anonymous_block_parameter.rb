def run_it(&block)
  block.call
end

def forwards(&)
  run_it(&)
end

def forwards_with_arguments(label, &)
  "#{label}: #{run_it(&)}"
end

class Relay
  def deliver(&)
    collect(&)
  end

  def collect(&block)
    block.call * 2
  end
end

puts forwards { "plain" }
puts forwards_with_arguments("labeled") { "value" }
puts Relay.new.deliver { 21 }
