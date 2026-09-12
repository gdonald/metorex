# The load path counts what was put on it, including anything RUBYLIB named,
# so the example measures the change rather than the total.
puts $:.class
started = $:.length
$:.unshift("test_path")
puts $:.length - started
puts $LOAD_PATH.length - started
puts $:.equal?($LOAD_PATH)
puts $/.class
puts $/
puts $!.class
puts $DEBUG.class
