# An object's singleton class is a kind of its class's singleton class.
klass = Class.new
sub = Class.new(klass)
instance = klass.new
p instance.singleton_class.superclass.equal?(klass)
p instance.singleton_class.is_a?(klass.singleton_class)
p sub.is_a?(klass.singleton_class)
p sub.new.singleton_class.is_a?(klass.singleton_class)
p klass.singleton_class.is_a?(klass.singleton_class)
