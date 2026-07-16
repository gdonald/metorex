# Module#const_source_location — written with parentheses. Locations are recorded for class/module keywords, constant
# assignments, const_set, and eval with an explicit filename and line.
# Builtins report an empty array; missing constants report nil. `module`
# and `class` keyword bodies are scope gates: enclosing locals are not
# visible inside.
module Located
  CONST_LINE = __LINE__ + 1
  VALUE = 1

  class Inner
    LINE = __LINE__ - 1
  end
end

file = __FILE__
loc = Located.const_source_location(:VALUE)
puts(loc == [file, Located::CONST_LINE])

loc = Located.const_source_location("Inner")
puts(loc == [file, Located::Inner::LINE])

set_line = __LINE__ + 1
Located.const_set(:FROM_SET, 2)
puts(Located.const_source_location(:FROM_SET) == [file, set_line])

holder = Class.new do
  eval('self::FROM_EVAL = 3', nil, "virtual.rb", 100)
end
puts(holder.const_source_location(:FROM_EVAL).inspect)

puts(Object.const_source_location(:String).inspect)
puts(Located.const_source_location(:MISSING).inspect)
puts(Located.const_source_location(:VALUE, false) == [file, Located::CONST_LINE])

class Wrapper
  body_local = :hidden
  class Gate
    visible = defined?(body_local)
    puts(visible.inspect)
  end
end
