mod = Module.new
def mod.hello
  :hi
end

def (nil).blank
  :blank
end

class Named
end

def Named.build
  :built
end

puts(mod.methods(false).inspect)
puts(mod.dup.methods(false).inspect)
puts(mod.dup.hello.inspect)
puts(mod.instance_methods.inspect)
puts(NilClass.instance_methods(false).inspect)
puts(Named.methods(false).inspect)
puts(Named.build.inspect)
