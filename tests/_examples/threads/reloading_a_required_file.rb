# A file taken back out of the list of what has been loaded is loaded again,
# which is how one file can define a second thing under a second name.
require "tmpdir"

path = File.join Dir.tmpdir, "metorex_reload_#{Process.pid}.rb"
File.write path, "$counted = ($counted || 0) + 1\n"

require path
puts $counted

require path
puts $counted

puts $LOADED_FEATURES.include?(path)
$LOADED_FEATURES.delete path
puts $LOADED_FEATURES.include?(path)

require path
puts $counted

File.unlink path
