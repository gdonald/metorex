require "io/console"
require "pty"
require "stringio"

PTY.open do |primary, terminal|
  terminal.winsize = [30, 100]
  p terminal.winsize
  p terminal.echo?
  terminal.echo = false
  p terminal.echo?
  terminal.echo = true
  p(terminal.raw { |raw| raw.echo? })
  mode = terminal.console_mode
  terminal.console_mode = mode.raw min: 2, time: 0.5
  p terminal.echo?
  terminal.console_mode = mode
  p terminal.noecho { terminal.echo? }
  p terminal.echo?
  primary.write("q")
  p terminal.getch
  terminal.goto(2, 4)
  terminal.erase_line(0)
  terminal.cursor_up(1)
  p primary.read_nonblock(100)
  p (terminal.erase_screen(9) rescue $!)
  p terminal.ttyname.start_with?("/dev/")
end

File.open(File::NULL) do |null|
  p((null.winsize rescue $!.is_a?(SystemCallError)))
  p null.ttyname
end

input = StringIO.new("s3cret\n")
p input.getpass("Password: ")
p input.string

order = {id: 1042, lines: (1..12).map { |line| {sku: "SKU-#{line}", quantity: line} }}
pp order
puts order.pretty_inspect.lines.size

p defined?(Pathname)
p Pathname("/var/log/app.log").basename
p Pathname("/var/log").join("app.log").to_s
p Pathname("logs/../app.log").cleanpath
p Pathname("a") <=> Pathname("b")
p Pathname.instance_method(:writable?).source_location.first
p require("pathname")
p Pathname("/var/log").respond_to?(:find)
