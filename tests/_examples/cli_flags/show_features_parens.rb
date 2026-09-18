# What `--enable` and `--disable` left behind, and how loud the run is.
puts(defined?(Gem).inspect)
puts(defined?(DidYouMean).inspect)
puts($VERBOSE.inspect)
puts("frozen #{"literal".frozen?}")
