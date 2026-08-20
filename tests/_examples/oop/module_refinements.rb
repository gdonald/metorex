collected = []

Refiner = Module.new do
  refine String do
    collected << self
  end

  refine Array do
    collected << self
  end
end

Plain = Module.new do
  include Refiner
end

puts Refiner.refinements.length.inspect
puts (Refiner.refinements == collected).inspect
puts Plain.refinements.inspect
puts Module.new.refinements.inspect

ignore_all = -> * { :any }
puts ignore_all.call(1, 2, 3).inspect

named_rest = -> *rest { rest }
puts named_rest.call(4, 5).inspect

parenthesized = ->(*) { :parens }
puts parenthesized.call(6).inspect
