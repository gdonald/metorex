# Kernel#caller_locations — written without parentheses where possible.
# Inside a const_added hook, level 1 is the site of the constant
# definition, including lines inside a module_eval string with an explicit
# lineno. Also checks that tokens after a heredoc opener keep the opener's
# line number.
$lines = []

mod = Module.new do
  def self.const_added name
    locs = caller_locations 1, 1
    $lines << locs[0].lineno
  end
end

line = __LINE__
mod.module_eval(<<-RUBY, __FILE__, __LINE__ + 1)
  TEST = 1

  module SubModule
  end
RUBY

mod.const_set :CONST_SET, 1

puts $lines == [line + 2, line + 4, line + 8]
puts __LINE__ == line + 11
