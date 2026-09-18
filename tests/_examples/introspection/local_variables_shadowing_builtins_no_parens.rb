# A local named after a builtin method is a local like any other: it is
# reserved from the top of the body it is assigned in, and `local_variables`
# names it.
def shadows_p
  before = local_variables
  p = 5
  [before, local_variables, p]
end
puts shadows_p.inspect

def shadows_format
  format = "written"
  binding.local_variables
end
puts shadows_format.inspect

# The name a block closes over is reported by the binding taken inside it.
def closed_over_builtin_name
  print = "held"
  proc { |only| binding.local_variables }.call 1
end
puts closed_over_builtin_name.inspect
