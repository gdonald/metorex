puts __dir__.end_with? "tests/_examples/runtime"
puts __dir__ == File.dirname(File.expand_path(__FILE__))

in_eval = eval "__dir__", nil, "foo.rb"
p in_eval
nested = eval "__dir__", nil, "foo/bar.rb"
p nested
with_binding = eval "__dir__", binding
p with_binding
