# A binding and the code eval'd through it name the script by the path it
# was run with, the way __FILE__ does.
held = binding
p held.source_location == [__FILE__, 3]
p held.eval("__FILE__") == "(eval at #{__FILE__}:5)"
p Object.new.instance_eval("__FILE__") == "(eval at #{__FILE__}:6)"
p eval("__FILE__") == "(eval at #{__FILE__}:7)"
