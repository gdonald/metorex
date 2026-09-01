# `singleton_method_undefined` fires when a singleton method is undefined,
# and `undef_method` in a `class << Klass` body reaches a method that
# `def Klass.name` put there.

klass = Class.new

def klass.singleton_method_undefined(name)
  puts("class undefined #{name}")
end

def klass.to_undefine
end

puts(klass.respond_to?(:to_undefine).to_s)

class << klass
  undef_method(:to_undefine)
end

puts(klass.respond_to?(:to_undefine).to_s)

begin
  klass.to_undefine
rescue NoMethodError
  puts("NoMethodError after undef")
end

# Undefining a name nothing defines is a NameError.
begin
  class << klass
    undef_method(:never_there)
  end
rescue NameError => error
  puts(error.class.to_s)
end
