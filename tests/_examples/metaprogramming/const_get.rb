# Module#const_get — written without parentheses where possible. Covers
# ancestor lookup, the inherit flag, scoped and toplevel-anchored names,
# const_missing dispatch, NameError#name, and paren-less lambda
# parameters.
module Lookup
  module Inc
    FROM_MODULE = :from_module
  end

  class Parent
    FROM_PARENT = :from_parent
  end

  class Child < Parent
    include Inc
  end

  class Handled
    def self.const_missing name
      [:missing, name]
    end
  end
end

TOP_CONST = :top

puts Lookup::Child.const_get(:FROM_PARENT).inspect
puts Lookup::Child.const_get(:FROM_MODULE).inspect
puts Lookup::Child.const_get(:TOP_CONST).inspect
puts Lookup.const_get("Child::FROM_PARENT").inspect
puts Lookup.const_get("::TOP_CONST").inspect
puts Lookup::Handled.const_get(:ANYTHING).inspect

inspect_error = -> e { puts e.name.inspect }
begin
  Lookup::Child.const_get :FROM_PARENT, false
rescue NameError => e
  inspect_error.call e
end

begin
  Lookup.const_get "no_caps"
rescue NameError
  puts "NameError"
end
