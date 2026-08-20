module Namespace
  Holder = :placeholder
end

path = File.expand_path("autoload_reload/defines_constant.rb", File.dirname(__FILE__))

registry = Module.new { autoload :VALUE, path }
Namespace::Holder = registry
copy = registry.dup

puts registry.autoload?(:VALUE).nil?.inspect
puts copy.autoload?(:VALUE).nil?.inspect
puts registry::VALUE.inspect

begin
  copy::VALUE
rescue NameError
  puts "copy raises NameError"
end
