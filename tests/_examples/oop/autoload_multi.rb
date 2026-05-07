module ModuleSpecs
  module Autoload
  end
end

module ModuleSpecs::Autoload
  L = :autoload_l
  M = :autoload_m
end

puts ModuleSpecs::Autoload::L
puts ModuleSpecs::Autoload::M
