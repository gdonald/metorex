module Loader
  def self.run
    load File.expand_path("autoload_lib/loaded_definition.rb", __dir__)
  end
end

Loader.run
puts nesting_where_defined.inspect
puts defined_at_top_level
