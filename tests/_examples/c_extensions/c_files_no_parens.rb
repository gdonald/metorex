# A C extension reading a String's bytes through RSTRING_PTR and RSTRING_LEN,
# converting a path with FilePathValue, and opening files.
require "tmpdir"
require_relative "build_helper"

class PathHolder
  def initialize(path)
    @path = path
  end

  def to_path
    @path
  end
end

class TextHolder
  def initialize(text)
    @text = text
  end

  def to_str
    @text
  end
end

directory = Dir.mktmpdir
require build_extension("c_files.c", "c_files", directory)
files = CFiles.new

p files.bytes("héllo")
p files.bytes("\xFF\x00".b)
p files.bytes ""
text = +"before"
p files.terminated text
text << " and after"
p files.bytes(text).size
p files.same_pointer text
report { files.bytes 42 }

path = "written.txt"
p files.path_value(path).equal? path
p files.path_value PathHolder.new "from_path"
p files.path_value TextHolder.new "from_str"
p files.path_value PathHolder.new TextHolder.new "through_both"
report { files.path_value TextHolder.new 3 }
report { files.path_value PathHolder.new 3 }
report { files.path_value nil }

name = File.join directory, "c_files.txt"
file = files.open_name name, "w"
p file.class
file.write "written from C"
file.close
p File.read name
file = files.open_path PathHolder.new(name), "r"
p file.read
file.close
report { files.open_name name, "" }
report { files.open_path 42, "r" }

FileUtils.rm_rf directory
