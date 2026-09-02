$load_count = 0
lib = File.expand_path("load_lib/counted.rb", __dir__)

load(lib)
load(lib)
puts($load_count)
puts($LOADED_FEATURES.include?(lib))

require(lib)
puts($load_count)
require(lib)
puts($load_count)

class PathLike
  def to_path
    File.expand_path "load_lib/counted.rb", File.dirname(__FILE__)
  end
end

load(PathLike.new)
puts($load_count)

begin
  load("./does_not_exist_here.rb")
rescue LoadError => error
  puts(error.message)
end

begin
  load(42)
rescue TypeError => error
  puts(error.message)
end

puts(File::Separator)
puts(Process.euid == Process.uid)
puts(Kernel.private_instance_methods.include?(:load))

wrapper = Module.new
load(File.expand_path("load_lib/wrapped.rb", __dir__), wrapper)
puts(Object.const_defined?(:WRAPPED_CONSTANT))
puts(wrapper.const_get(:WRAPPED_CONSTANT))
puts(wrapper.instance_methods.include?(:wrapped_method))
