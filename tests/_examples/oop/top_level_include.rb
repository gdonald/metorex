module Greeter
  def hello
    "hi from greeter"
  end
end

include Greeter
puts Object.ancestors.include?(Greeter)
puts hello
