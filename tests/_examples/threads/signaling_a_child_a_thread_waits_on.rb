# A thread reading from a child lets the main thread signal that child.
require "tmpdir"
require "rbconfig"
pid_file = File.join(Dir.tmpdir, "metorex_waits_for_term_#{Process.pid}")
File.delete(pid_file) if File.exist?(pid_file)
fixture = File.join(__dir__, "fixtures", "waits_for_term.rb")
reader = Thread.new do
  IO.popen([RbConfig.ruby, fixture, pid_file]) { |io| io.read }
end
Thread.pass until File.exist?(pid_file) && !File.read(pid_file).empty?
Process.kill(:TERM, File.read(pid_file).to_i)
p(reader.value)
File.delete(pid_file)
