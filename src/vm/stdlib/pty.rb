# Pseudo-terminals: a pair of ends where what is written to one reads as
# typed at a terminal on the other, and programs run with one as their
# terminal.
module PTY
  # Raised by `PTY.check` when asked to, naming how the child changed.
  class ChildExited < RuntimeError
    attr_reader :status

    def initialize(message = nil, status = nil)
      super(message)
      @status = status
    end
  end

  # The controlling end as an IO and the terminal end as a File named by its
  # device path. With a block, the pair is handed to it and both ends are
  # closed afterwards.
  def self.open
    controlling, terminal, name = IO.__stream__("openpty", 0, "", 0)
    pair = [IO.for_fd(controlling, "r+"), File.for_fd(terminal, "r+", path: name)]
    pair.each { |stream| stream.sync = true }
    return pair unless block_given?

    begin
      yield pair
    ensure
      pair.each { |stream| stream.close unless stream.closed? }
    end
  end

  # Run a command with a fresh terminal of its own, answering the end to
  # read its output from, the end to type into, and its process id. One
  # String runs through the shell, and a leading Hash changes the child's
  # environment. With a block, those three are handed to it, the child is
  # left to be reaped in the background, and nil is answered.
  def self.spawn(*arguments)
    environment = arguments.first.is_a?(Hash) ? arguments.shift : {}
    if arguments.empty?
      raise ArgumentError, "wrong number of arguments (given 0, expected 1+)"
    end
    words = arguments.size == 1 ? ["/bin/sh", "-c", arguments.first.to_s] : arguments.map(&:to_s)
    changes = environment.map { |name, value| [name.to_s, value.nil? ? nil : value.to_s] }
    pid, reading, typing, name = IO.__stream__("pty_spawn", 0, "", 0, words, changes)
    reader = File.for_fd(reading, "r", path: name)
    writer = File.for_fd(typing, "w", path: name)
    writer.sync = true
    return [reader, writer, pid] unless block_given?

    begin
      yield reader, writer, pid
    ensure
      Process.detach(pid)
    end
    nil
  end

  class << self
    alias getpty spawn
  end

  # The status of a child that has stopped or ended, or nil while it runs
  # or once it has been reaped. Asked to raise, it raises ChildExited
  # rather than answering a status.
  def self.check(pid, raise_exception = false)
    reaped = begin
      Process.waitpid(pid, Process::WNOHANG | Process::WUNTRACED)
    rescue Errno::ECHILD
      nil
    end
    return nil if reaped.nil? || reaped.zero?

    status = $?
    return status unless raise_exception

    state = status.stopped? ? "stopped" : "exited"
    raise ChildExited.new("pty - #{state}: #{pid}", status)
  end
end
