puts $:.class
puts $:.length
$:.unshift("test_path")
puts $:.length
puts $LOAD_PATH.length
puts $/.class
puts $/
puts $!.class
puts $DEBUG.class
