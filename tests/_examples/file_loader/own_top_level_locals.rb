# A file loaded from another one keeps its own top-level locals, so a name it
# binds there never shows up in the file that loaded it. Constants, methods,
# and classes are shared.
require_relative "own_top_level_locals_lib"

p(LOADED_CONSTANT)
p(loaded_method)

begin
  p(loaded_local)
rescue NameError => refused
  p(refused.message)
end

p(binding.local_variables.sort)
