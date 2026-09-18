# TOPLEVEL_BINDING stands over the script's own top level: a name assigned
# there is reserved from the start and reads back as it is set.
p TOPLEVEL_BINDING.local_variable_get :counter
counter = 1
p TOPLEVEL_BINDING.local_variable_get :counter
counter = 2
p TOPLEVEL_BINDING.local_variable_get :counter
p TOPLEVEL_BINDING.local_variables.sort

# A name set through the binding joins the ones the script wrote.
TOPLEVEL_BINDING.local_variable_set :through_the_binding, 3
p TOPLEVEL_BINDING.local_variables.sort
