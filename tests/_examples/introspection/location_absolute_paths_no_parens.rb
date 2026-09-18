# A backtrace entry names the real path of the file it sits in, resolved when
# that file was loaded rather than when the entry is read.
here = caller_locations(0)[0]
puts here.absolute_path == File.realpath(__FILE__)
puts here.path == __FILE__

# Code eval'd under a name no file answers to has no real path at all.
puts (eval "caller_locations(0)[0].absolute_path", nil, "foo.rb").inspect
puts (eval "caller_locations(0)[0].path", nil, "foo.rb").inspect
