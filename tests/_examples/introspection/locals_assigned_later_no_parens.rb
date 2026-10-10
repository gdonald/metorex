# `local_variables` lists every local of a scope, the ones assigned further
# down among them, inside blocks as well, while a block written above an
# assignment still cannot read that local.
def listed
  p local_variables
  first = 1
  [1].each do |item|
    [2].each { p local_variables, binding.local_variables }
    inner = 2
  end
  later = proc { p local_variables }
  last = 3
  later.call
end
listed

def early_read
  [1].each do
    p missing
  rescue NameError => error
    p error.class
  end
  missing = 1
  [1].each { p missing }
end
early_read

p local_variables
[1].each { p local_variables }
top = 1
