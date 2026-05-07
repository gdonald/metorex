module Outer
  class DuringAutoloadAfterDefine
    puts "Outer.constants: #{Outer.constants.inspect}"
    puts "Outer.const_defined?(:DuringAutoloadAfterDefine): #{Outer.const_defined?(:DuringAutoloadAfterDefine, false)}"
    $during_value = $check.call
  end
end
