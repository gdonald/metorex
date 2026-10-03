# Builds the C source beside this file into an extension in `directory`, the
# way an extconf.rb and make would, and answers the path to require.
require "mkmf"
require "fileutils"

def build_extension(source, target, directory)
  FileUtils.cp(File.join(__dir__, source), File.join(directory, "#{target}.c"))
  Dir.chdir(directory) do
    append_cflags("-Wall")
    create_makefile(target)
    system("make > /dev/null 2>&1") or raise "make failed for #{target}"
  end
  File.join(directory, "#{target}.#{RbConfig::CONFIG["DLEXT"]}")
end

def report
  yield
rescue StandardError => error
  puts "#{error.class}: #{error.message}"
end
