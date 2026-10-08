# A failed system call raises the Errno class its error number names, with
# the system's reason, the C function MRI made the call from, and the path.
# A call on two paths names only the second when it already exists.
require "tmpdir"

def attempt
  yield
rescue SystemCallError => error
  p [error.class, error.message]
end

Dir.mktmpdir do |root|
  Dir.chdir(root) do
    Dir.mkdir("made")
    File.write("plain", "text")
    attempt { Dir.mkdir("made") }
    attempt { Dir.mkdir("missing/inner") }
    attempt { Dir.rmdir("missing") }
    attempt { File.unlink("missing") }
    attempt { File.utime(nil, nil, "missing") }
    attempt { File.symlink("plain", "made") }
    attempt { File.symlink("plain", "missing/link") }
    attempt { File.link("missing", "linked") }
    attempt { File.link("plain", "made") }
    Dir.mkdir("open", 0777)
    p "%o" % (File.stat("open").mode & 0777)
    File.chmod(0644, "plain")
    File.open("plain") do |stream|
      p "%o" % stream.stat.mode
      p stream.stat.size
    end
  end
end
