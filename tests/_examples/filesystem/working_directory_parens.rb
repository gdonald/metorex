# `Dir.chdir` names the directory the program works from, taking the name
# through `to_path`, and a `Dir` handle changes to the directory it names.
require "tmpdir"

Dir.mktmpdir do |scratch|
  Dir.mkdir(File.join(scratch, "inner"))

  named = Object.new
  named.define_singleton_method(:to_path) do
    File.join(scratch, "inner")
  end

  outside = Dir.pwd
  Dir.chdir(scratch) do
    p(Dir.pwd == File.realpath(scratch))
  end
  p(Dir.pwd == outside)

  p(Dir.chdir(named))
  p(File.basename(Dir.pwd))
  Dir.chdir(outside)

  handle = Dir.new(File.join(scratch, "inner"))
  p(handle.chdir { File.basename(Dir.pwd) })
  handle.close

  begin
    Dir.chdir(File.join(scratch, "missing"))
  rescue Errno::ENOENT => problem
    puts(problem.class)
  end
end
