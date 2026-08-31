begin
  raise("keyword form")
rescue RuntimeError => error
  puts error.message
end

class Thrower
  def throw_it
    send(:raise, ArgumentError, "through send")
  end
end

begin
  Thrower.new.throw_it
rescue ArgumentError => error
  puts "#{error.class}: #{error.message}"
end

raiser = Object.new
class << raiser
  public :raise
end

begin
  raiser.raise(TypeError, "with a receiver")
rescue TypeError => error
  puts "#{error.class}: #{error.message}"
end

begin
  Kernel.raise(IndexError, "on Kernel")
rescue IndexError => error
  puts "#{error.class}: #{error.message}"
end

begin
  Kernel.method(:raise).call(KeyError, "through a Method object")
rescue KeyError => error
  puts "#{error.class}: #{error.message}"
end

puts Kernel.private_instance_methods(false).include?(:raise)

begin
  raise
rescue RuntimeError => error
  puts error.message
end
