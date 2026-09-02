pid = fork { exit 42 }
_, status = Process.wait2 pid
puts status.exitstatus

quiet = fork {}
puts Process.wait2(quiet)[1].exitstatus

banged = fork { exit! 7 }
Process.waitpid banged
puts $?.exitstatus

marker = "/tmp/metorex_fork_marker_#{Process.pid}.txt"

writer = fork do
  File.open(marker, "w") { |file| file.write "written by child" }
  Process.exit!
end
Process.waitpid writer
puts File.read(marker)

background = Thread.new { :never_run }
puts background.alive?
puts Thread.current.alive?

seen = fork do
  File.open(marker, "w") { |file| file.write background.alive?.to_s }
  Process.exit!
end
Process.waitpid seen
puts File.read(marker)

File.delete marker

background.kill
puts background.alive?
puts Kernel.private_instance_methods.include? :fork
