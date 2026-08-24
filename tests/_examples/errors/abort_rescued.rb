class Stopper
  public :abort
end

begin
  abort "with parentheses omitted"
rescue SystemExit => error
  puts error.message
  puts error.status
end

begin
  abort("with parentheses")
rescue SystemExit => error
  puts error.message
  puts error.status
end

begin
  abort
rescue SystemExit => error
  puts error.message
  puts error.status
end

begin
  Kernel.abort "from the Kernel module"
rescue SystemExit => error
  puts error.message
end

begin
  Stopper.new.abort "from an instance"
rescue SystemExit => error
  puts error.message
end

class Reason
  def to_str
    "coerced with to_str"
  end
end

begin
  abort Reason.new
rescue SystemExit => error
  puts error.message
end

begin
  abort 123
rescue TypeError => error
  puts error.message
end

puts Kernel.private_instance_methods.include?(:abort)
