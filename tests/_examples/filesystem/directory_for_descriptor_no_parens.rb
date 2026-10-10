# A Dir made over a descriptor another Dir holds reads the same directory,
# has no path, and refuses a descriptor that names no directory.
require "tmpdir"

Dir.mktmpdir do |root|
  File.write File.join(root, "notes.txt"), ""
  Dir.mkdir File.join(root, "archive")
  opened = Dir.new root
  adopted = Dir.for_fd opened.fileno
  p adopted.children.sort
  p adopted.path
  p adopted.fileno == opened.fileno
  opened.close
end

begin
  Dir.for_fd nil
rescue TypeError => error
  puts error.message
end

begin
  Dir.for_fd -1
rescue SystemCallError => error
  puts error.message
end

begin
  Dir.for_fd $stdout.fileno
rescue SystemCallError => error
  puts error.message
end
