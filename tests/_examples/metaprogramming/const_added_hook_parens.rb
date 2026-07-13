# Module#const_added — written with parentheses. The hook fires when a
# constant is assigned, defined via const_set, registered as an autoload,
# or created by a class definition under the owner.
events = []

mod = Module.new do
  def self.const_added(name)
    $events << name
  end
end

$events = events

mod.const_set(:TEST, 1)
puts(events.inspect)

mod.module_eval("SECOND = 2")
puts(events.inspect)

mod.autoload(:Autoload, "foo")
puts(events.inspect)

parent = Class.new do
  def self.const_added(name)
    $events << name
  end
end

class parent::Child < parent; end
puts(events.inspect)

mod.const_added(:DIRECT)
puts(events.inspect)
