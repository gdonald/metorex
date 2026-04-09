# File.expand_path with non-existent path (manual normalization)
result = File.expand_path "/tmp/nonexistent_metorex_test/../foo/bar"
puts result

# With base dir (needs parens for two args on dotted call)
result2 = File.expand_path("baz", "/tmp/base")
puts result2
