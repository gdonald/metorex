def status_for(value)
  exit(value)
rescue SystemExit => error
  error.status
end

puts(status_for(0))
puts(status_for(8))
puts(status_for(-1))
puts(status_for(true))
puts(status_for(false))
puts(status_for(5.9))
puts(status_for(-2.2))

class Countable
  def to_int
    5
  end
end

puts(status_for(Countable.new))

["0", nil, [0], Object.new].each do |bad|
  begin
    exit bad
  rescue TypeError => error
    puts(error.message)
  end
end

class Runner
  public :exit
end

begin
  Runner.new.exit(3)
rescue SystemExit => error
  puts(error.status)
end

begin
  Kernel.exit(4)
rescue SystemExit => error
  puts(error.status)
end

puts(Kernel.private_instance_methods.include?(:exit))
puts(Kernel.private_instance_methods.include?(:exit!))
