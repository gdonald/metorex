# Loaded through `load(path, wrap)`. What it defines belongs to the module the
# load was wrapped in rather than to the program.
class WrappedInside
  $wrapped_notes << String
end

WRAPPED_CONSTANT = 1

def wrapped_top_method
  :wrapped_top_method
end

$wrapped_notes << method(:wrapped_top_method).owner
$wrapped_notes << self.to_s
