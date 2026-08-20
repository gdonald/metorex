module Base
end

module Middle
  include Base
end

class Plain
end

class Layered < Plain
  include Middle
end

puts Base.included_modules.inspect
puts Middle.included_modules.inspect
puts Layered.included_modules.inspect
puts Plain.included_modules.inspect
