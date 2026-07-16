# Module#const_missing — written with parentheses. Fires for literal
# qualified references and explicit calls; the default implementation
# raises NameError with the qualified path and the name attribute set.
# Symbol interpolation renders the bare name.
module Host
  def self.const_missing(name)
    "handled #{name}"
  end
end

module Bare
end

puts(Host::Anything)
puts(Host.const_missing(:Direct))

begin
  Bare.const_missing("Nope")
rescue NameError => e
  puts(e.message)
  puts(e.name.inspect)
end

begin
  Bare::AlsoMissing
rescue NameError => e
  puts(e.message)
end
