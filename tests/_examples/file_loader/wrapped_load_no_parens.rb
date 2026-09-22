# The same run written without parentheses where they can be left off.
#
# `load(path, true)` runs a file inside a module of its own, and
# `load(path, SomeModule)` inside the module it is handed. Either way the
# program keeps what it had: the constants and the methods belong to the
# wrapping module, and the file runs against an object of its own that says
# it is main.

here = File.expand_path("wrapped_load_target.rb", __dir__)

$wrapped_notes = []
load here, true
p $wrapped_notes[0]
p $wrapped_notes[1].instance_of? Module
p $wrapped_notes[2]
p Object.const_defined? :WRAPPED_CONSTANT

$wrapped_notes = []
holder = Module.new
load here, holder
p $wrapped_notes[1] == holder
p holder.const_defined? :WRAPPED_CONSTANT
p holder.private_instance_methods.include? :wrapped_top_method

reader = Object.new
reader.extend holder
p reader.send :wrapped_top_method
