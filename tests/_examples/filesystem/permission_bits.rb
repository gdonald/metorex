# `chmod` sets the permission bits through a name or through an open handle,
# and a name that is not there raises.
require "tmpdir"

Dir.mktmpdir do |scratch|
  named = File.join scratch, "notes.txt"
  File.write named, "hello"

  p File.chmod 0o444, named
  p format "%o", File.stat(named).mode & 0o777

  handle = File.open named, "r"
  p handle.chmod 0o600
  p format "%o", File.stat(named).mode & 0o777
  handle.close

  begin
    File.chmod 0o644, File.join(scratch, "missing.txt")
  rescue Errno::ENOENT => problem
    puts problem.class
  end

  begin
    File.chmod 2**64, named
  rescue RangeError => problem
    puts problem.class
  end
end
