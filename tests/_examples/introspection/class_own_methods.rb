# Every class answers the methods Class defines, which Class lists as its
# own, apart from the ones it inherits from Module.
names = %i[allocate superclass subclasses attached_object new]
p(names.map { |name| Object.respond_to?(name) })
p(names.map { |name| RubyVM::InstructionSequence.respond_to?(name) })
p(names.map { |name| Comparable.respond_to?(name) })
p(Class.instance_methods(false).sort)
p(Class.public_instance_methods(false).sort)
p(Class.private_instance_methods(false).sort)
p(Class.instance_methods.include?(:alias_method))
p(Object.superclass)
