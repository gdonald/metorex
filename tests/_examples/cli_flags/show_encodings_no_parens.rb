internal = Encoding.default_internal
puts "source #{__ENCODING__.name}"
puts "external #{Encoding.default_external.name}"
puts "internal #{internal.nil? ? "none" : internal.name}"
