# A path may arrive as anything that names one, and a predicate takes a
# stream as the file it is open on.
class Path
  def initialize name
    @name = name
  end

  def to_path
    @name
  end
end

class Stream
  def to_io
    STDIN
  end
end

p File.directory?(Path.new("/tmp"))
p FileTest.directory?(Path.new("/tmp"))
p File.directory?(Stream.new)

target = "/tmp/metorex_stream_paths_target.txt"
link = "/tmp/metorex_stream_paths_link.txt"
File.write target, "held"
File.delete link if File.symlink? link
File.symlink Path.new(target), Path.new(link)
p File.symlink?(link)
p File.read(link)
File.delete link
File.delete target
