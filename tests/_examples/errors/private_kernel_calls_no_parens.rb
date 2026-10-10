# Kernel's functions and BasicObject's hooks are private methods of every
# object: a call written with a receiver other than `self` is refused, and
# `send` reaches them.
class Speaker
  public :puts
end

class Forwarder
  def method_missing(name, *args)
    "forwarded #{name} with #{args.inspect}"
  end

  def respond_to_missing?(name, include_private = false) = true
end

def attempt
  p yield
rescue NoMethodError, ArgumentError => error
  p [error.class, error.message]
end

attempt { Object.new.puts("hidden") }
attempt { nil.raise }
attempt { [1].format("%d", 1) }
attempt { Speaker.new.puts("spoken") }
attempt { Forwarder.new.puts("anything") }
attempt { Object.new.send(:initialize) }
attempt { Object.new.send(:initialize, 1) }
attempt { Object.new.initialize }
attempt { Object.new.send(:singleton_method_added, :name) }
attempt { Object.new.send(:method_missing, :absent) }
attempt { Object.new.send(:method_missing) }
attempt { Object.new.send(:method_missing, "absent") }
attempt { $stdout.respond_to?(:puts) }
attempt { Object.new.respond_to?(:puts) }
attempt { Object.new.respond_to?(:puts, true) }

def asks = iterator?
p [asks, asks {}]
p send(:instance_variables_to_inspect)

# A method the program writes with an empty body runs as the empty method,
# though a native method of the same name exists.
class Quiet
  def initialize
  end

  def to_s
  end
end
p Quiet.new.to_s
