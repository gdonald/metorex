# The same run written without parentheses where they can be left off.
#
# `require_relative` reads the path it is handed the way Ruby does: it names a
# `.rb` file, it takes anything that says how to read itself as a path, and it
# answers whether the file was loaded by this call.

here = __dir__

# A path written without `.rb` names the `.rb` file beside it.
p require_relative "required_as_ruby_target"
p require_relative "required_as_ruby_target.rb"

# The feature list holds the expanded path, with no dot components left in it.
p $LOADED_FEATURES.include? File.join(here, "required_as_ruby_target.rb")

# A path may be named by anything that says how to read itself as one.
class NamesAPath
  def initialize(named)
    @named = named
  end

  def to_path
    @named
  end
end

p require_relative NamesAPath.new("required_as_ruby_other")

begin
  require_relative 42
rescue TypeError => refused
  p refused.message
end

begin
  require_relative "no_such_file_here"
rescue LoadError => refused
  p refused.path == File.join(here, "no_such_file_here")
end

# A file that fails part-way through was never loaded.
begin
  require_relative "required_as_ruby_refuses"
rescue RuntimeError
  p $LOADED_FEATURES.include? File.join(here, "required_as_ruby_refuses.rb")
end
