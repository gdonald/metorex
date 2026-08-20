$events = []

Tracked = Module.new do
  def self.method_removed name
    $events << [:removed, name]
  end

  def self.method_undefined name
    $events << [:undefined, name]
  end

  def doomed
  end

  def shadowed
  end

  remove_method :doomed
  undef_method :shadowed
end

puts $events.inspect
puts Tracked.instance_methods(false).inspect
puts Module.new.method_removed(:anything).inspect
puts Module.private_instance_methods.include?(:method_removed).inspect

frozen = Module.new
frozen.freeze
begin
  frozen.alias_method :a, :b
rescue FrozenError => error
  puts error.message
end
