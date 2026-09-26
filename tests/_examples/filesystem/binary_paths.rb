# A path tagged binary names the file its bytes spell, whichever method it is
# handed to.
name = "/tmp/metorex_binary_path_#{Process.pid}_あ".force_encoding "binary"
Dir.mkdir name
expanded = File.expand_path name
p expanded.encoding
p File.directory?(name)
p File.directory?(expanded)
p File.exist?(expanded)
p File.symlink?(expanded)
p Dir.entries(expanded).sort
File.write "#{expanded}/inside.txt".b, "held"
p File.file?("#{expanded}/inside.txt".b)
p Dir.children(expanded)
File.delete "#{expanded}/inside.txt".b
Dir.rmdir expanded
p File.exist?(name)
