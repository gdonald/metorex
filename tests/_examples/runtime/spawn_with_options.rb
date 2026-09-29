# Process.spawn reads its options Hash the way Ruby does.
require "tmpdir"
path = File.join(Dir.tmpdir, "metorex_spawn_with_options_#{Process.pid}.txt")
name = Object.new
def name.to_str = "GREETING"
pid = Process.spawn({ name => "hello" }, "echo $GREETING; echo gone >&2", [:out, :err] => path, umask: 0o077)
Process.wait(pid)
p(File.read(path))
File.delete(path)
[
  -> { Process.spawn("true", pgroup: -1) },
  -> { Process.spawn("true", pgroup: :yes) },
  -> { Process.spawn("true", nonesuch: 1) },
  -> { Process.spawn("true", "chdir" => "/") },
  -> { Process.spawn({ "A=B" => "1" }, "true") },
  -> { Process.spawn(:true) },
  -> { Process.spawn({}, {}) },
].each do |attempt|
  begin
    attempt.call
  rescue ArgumentError, TypeError => error
    p([error.class, error.message])
  end
end
