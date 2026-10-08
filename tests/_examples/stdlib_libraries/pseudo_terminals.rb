# PTY runs a program with a terminal of its own and opens terminal pairs, and
# PTY.check reports on a child without waiting for it.
require("pty")
p(PTY.singleton_methods.sort)
p(PTY::ChildExited.superclass)

reader, writer, pid = PTY.spawn("printf 'hello\\n'; exit 3")
p([reader.class, writer.class, pid.class])
p(reader.equal?(writer))
p(reader.gets)
Process.wait(pid)
p($?.exitstatus)
p(PTY.check(pid))

controlling, terminal = PTY.open
p([controlling.class, terminal.class])
p(terminal.tty?, controlling.tty?)
p(terminal.path.start_with?("/dev/"))
terminal.write("typed\n")
p(controlling.readpartial(100))
controlling.write("back\n")
p(terminal.gets)
controlling.close
terminal.close
PTY.open { |pair| p(pair.map(&:class)) }
p(PTY.open { :answer })

child = fork { sleep 5 }
p(PTY.check(child))
Process.kill(:KILL, child)
status = nil
status = PTY.check(child) until status
p(status.class, status.signaled?)

PTY.spawn("echo", "block form") { |output, _input, spawned| p(output.gets); Process.wait(spawned) }
greeting = PTY.spawn({ "GREETING" => "hi" }, "echo $GREETING")
p(greeting[0].gets)
Process.wait(greeting[2])

ending = PTY.spawn("exit 7")[2]
begin
  loop { PTY.check(ending, true) }
rescue PTY::ChildExited => error
  p(error.status.exitstatus)
  p(error.message == "pty - exited: #{ending}")
end
