module Outer
end

$during_value = nil
$check = -> { defined?(Outer::DuringAutoloadAfterDefine) }

target_path = File.expand_path("autoload_during_target.rb", __dir__)
Outer.autoload :DuringAutoloadAfterDefine, target_path

before = $check.call
puts "before: #{before.inspect}"

Outer.const_get(:DuringAutoloadAfterDefine)
puts "during (after const assigned in fixture): #{$during_value.inspect}"

after = $check.call
puts "after: #{after.inspect}"
