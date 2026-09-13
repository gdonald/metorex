puts(eval("class Twenty; 20; end").to_s)
puts(eval("module Held; :held; end").to_s)
puts(eval("class Empty; end").inspect)

class Named
  @tally = 0
  @@shared = :shared
end
puts(Named.instance_variables.inspect)
puts(Named.class_variables.inspect)

begin
  eval("class FromString < ''; end")
rescue TypeError => error
  puts(error.message)
end

# A call written onto the `end` reads what the body answered, not the class.
module Wrapper
  class Inner
    :inner
  end.to_s.tap { |answered| puts(answered) }
end
