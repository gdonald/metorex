# Module#const_defined? — written with parentheses. Covers own constants,
# ancestor lookup, the inherit flag (with boolean coercion), scoped and
# toplevel-anchored names, unicode constant names, and NameError on
# invalid names.
module Outer
  CS_CONSTλ = :unicode

  module Inc
    FROM_MODULE = 1
  end

  class Parent
    FROM_PARENT = 2
  end

  class Child < Parent
    include Inc
  end
end

puts(Outer.const_defined?(:CS_CONSTλ))
puts(Outer::Child.const_defined?(:FROM_PARENT))
puts(Outer::Child.const_defined?(:FROM_MODULE))
puts(Outer::Child.const_defined?(:FROM_PARENT, false))
puts(Outer::Child.const_defined?(:FROM_PARENT, nil))
puts(Outer::Child.const_defined?(:FROM_PARENT, :truthy))
puts(Outer.const_defined?("Child::FROM_PARENT"))
puts(Outer.const_defined?("Child::FROM_PARENT_MISSING"))
puts(Outer.const_defined?("::Outer"))
puts(Outer.const_defined?("::Missing"))
puts(Outer.const_defined?(:CS_MISSING))

begin
  Outer.const_defined?("lowercase")
rescue NameError => e
  puts(e.class)
end

begin
  Outer.const_defined?("Name?")
rescue NameError => e
  puts(e.class)
end
