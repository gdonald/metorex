# Module#constants — written with parentheses. Covers own constants,
# mixin constants (transitively), the inherit flag, private_constant
# filtering, Symbol indexing, and Array difference.
module Deep
  DEEP_CONST = 1
end

module Shallow
  include Deep
  SHALLOW_CONST = 2
end

class Holder
  include Shallow
  OWN_CONST = 3
  SECRET = 4
  private_constant(:SECRET)
end

puts(Holder.constants.sort.inspect)
puts(Holder.constants(false).sort.inspect)
puts(Holder.constants(nil).sort.inspect)

inherited_only = Holder.constants - Holder.constants(false)
puts(inherited_only.sort.inspect)

sym = :Word
puts(sym[0])
puts(sym[0] == sym[0].upcase)

count = Module.constants.size
module AddedTop
end
puts(Module.constants.size == count + 1)
puts(Module.constants.include?(:Array))
Object.send(:remove_const, :AddedTop)
