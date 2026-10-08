# `ruby -run -e` runs the file commands the un library defines, each reading
# its options with OptionParser and working through FileUtils, and prints the
# help for each.
require "rbconfig"
require "tmpdir"

def run *words
  read, write = IO.pipe
  child = spawn RbConfig.ruby, "-run", "-e", *words, out: write, err: write
  write.close
  output = read.read
  Process.wait child
  first = output.lines.first.to_s
  # A failure is reported by its message and class, whose place differs.
  output = "#{first[/: (.*\(\w+(::\w+)*\))$/, 1]}\n" if first.include?(".rb:") || first.start_with?("-e:")
  puts "$ #{words.join ' '}", output, "exit #{$?.exitstatus}"
end

def listing
  Dir.glob("**/*", base: ".").sort.map do |entry|
    if File.symlink? entry
      "#{entry} -> #{File.readlink entry}"
    elsif File.directory? entry
      "#{entry}/ #{'%o' % (File.stat(entry).mode & 0777)}"
    else
      "#{entry} #{'%o' % (File.stat(entry).mode & 0777)} #{File.read(entry).inspect}"
    end
  end
end

Dir.mktmpdir do |root|
  Dir.chdir root do
    File.write "one.txt", "one\n"
    run "mkdir", "--", "-v", "a"
    run "mkdir", "--", "-p", "-v", "deep/er/est"
    run "mkdir", "--", "a"
    run "touch", "--", "-v", "t1", "t2"
    run "cp", "--", "-v", "one.txt", "copy.txt"
    run "cp", "--", "-rv", "deep", "a"
    run "cp", "--", "-p", "one.txt", "kept.txt"
    run "cp", "--", "-l", "-v", "deep", "linked"
    run "ln", "--", "-v", "one.txt", "hard.txt"
    run "ln", "--", "-s", "-v", "one.txt", "soft.txt"
    run "ln", "--", "-sf", "-v", "t1", "soft.txt"
    run "mv", "--", "-v", "t2", "moved.txt"
    run "mv", "--", "one.txt", "t1", "a"
    run "chmod", "--", "-v", "600", "copy.txt"
    run "chmod", "--", "-v", "u+x,go-r", "kept.txt"
    run "install", "--", "-v", "-m", "640", "copy.txt", "inst/copy.txt"
    run "install", "--", "-p", "-v", "copy.txt", "inst2/"
    run "rm", "--", "-v", "hard.txt"
    run "rm", "--", "nope"
    run "rm", "--", "-f", "-v", "nope"
    run "rm", "--", "-r", "-v", "linked"
    run "rmdir", "--", "-p", "-v", "deep/er/est"
    File.write "globa", ""
    File.write "globb", ""
    run "rm", "--", "-v", "glob*"
    run "wait_writable", "--", "-v", "-n", "1", "moved.txt"
    run "wait_writable", "--", "missing"
    run "help", "cp", "mv"
    run "help", "nonesuch"
    run "cp", "--", "--help"
    run "httpd"
    puts listing
  end
end
