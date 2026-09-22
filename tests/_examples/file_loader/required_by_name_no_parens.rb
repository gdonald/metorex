# The same run written without parentheses where they can be left off.
#
# What `require` does with the name it is handed: it searches the load path
# for a `.rb` file, answers whether this call loaded it, and reads a name
# already listed as loaded without going looking for it.

$required_by_name = 0
here = __dir__

$LOAD_PATH.unshift here

p require "required_by_name_target"
p require "required_by_name_target"
p $required_by_name

# A path written from the working directory names that one file and is not
# searched for along the load path.
begin
  require "./not_beside_the_working_directory"
rescue LoadError => refused
  p refused.path
end

# What the interpreter carries itself is loaded already.
p require "set"

# A file already listed under the name asked for is not looked for at all.
$LOADED_FEATURES << "a_name_with_no_file.rb"
p require "a_name_with_no_file.rb"

$LOAD_PATH.shift
