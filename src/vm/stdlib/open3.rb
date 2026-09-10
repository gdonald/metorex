# Running a command and reaching its streams. Each of these starts the
# command through the shell and hands back what it wrote, what it wrote to
# its error stream, and how it ended.

require 'tmpdir'

module Open3
  # Run `command`, answering what it wrote to its output stream and the
  # status it ended with.
  def self.capture2(*command, stdin_data: nil, **_options)
    written = Open3.written_form command
    output = Open3.run_capturing written, stdin_data, nil
    [output, Process.last_status]
  end

  # The same, with the error stream joined onto the output stream.
  def self.capture2e(*command, stdin_data: nil, **_options)
    written = Open3.written_form command
    output = Open3.run_capturing "{ #{written} ; } 2>&1", stdin_data, nil
    [output, Process.last_status]
  end

  # The same again, with the two streams kept apart.
  def self.capture3(*command, stdin_data: nil, **_options)
    written = Open3.written_form command
    held = Open3.scratch_name
    output = Open3.run_capturing "{ #{written} ; } 2>#{held}", stdin_data, nil
    status = Process.last_status
    errors = File.exist?(held) ? File.read(held) : ""
    File.delete held if File.exist? held
    [output, errors, status]
  end

  # Run `command` and hand its output stream to the block, which is what
  # `popen2` gives beyond `capture2`.
  def self.popen2(*command, **_options)
    written = Open3.written_form command
    stream = IO.popen written
    return [stream, stream, Open3.waiter] unless block_given?
    begin
      yield stream, stream, Open3.waiter
    ensure
      stream.close unless stream.closed?
    end
  end

  def self.popen2e(*command, **_options)
    written = Open3.written_form command
    stream = IO.popen "{ #{written} ; } 2>&1"
    return [stream, stream, Open3.waiter] unless block_given?
    begin
      yield stream, stream, Open3.waiter
    ensure
      stream.close unless stream.closed?
    end
  end

  # The output and error streams separately, with the error stream held in a
  # scratch file the reader is opened on.
  def self.popen3(*command, **_options)
    written = Open3.written_form command
    held = Open3.scratch_name
    output = IO.popen "{ #{written} ; } 2>#{held}"
    File.write held, "" unless File.exist? held
    errors = File.open held, "r"
    waiter = Open3.waiter
    return [output, output, errors, waiter] unless block_given?
    begin
      yield output, output, errors, waiter
    ensure
      output.close unless output.closed?
      errors.close unless errors.closed?
      File.delete held if File.exist? held
    end
  end

  # Run each command in turn, each one reading what the one before it wrote.
  def self.pipeline(*commands, **_options)
    Open3.run_capturing Open3.joined_form(commands), nil, nil
    [Process.last_status]
  end

  def self.pipeline_r(*commands, **_options)
    stream = IO.popen Open3.joined_form(commands)
    return [stream, [Open3.waiter]] unless block_given?
    begin
      yield stream, [Open3.waiter]
    ensure
      stream.close unless stream.closed?
    end
  end

  def self.pipeline_w(*commands, **_options)
    stream = IO.popen Open3.joined_form(commands)
    return [stream, [Open3.waiter]] unless block_given?
    begin
      yield stream, [Open3.waiter]
    ensure
      stream.close unless stream.closed?
    end
  end

  def self.pipeline_rw(*commands, **_options)
    stream = IO.popen Open3.joined_form(commands)
    return [stream, stream, [Open3.waiter]] unless block_given?
    begin
      yield stream, stream, [Open3.waiter]
    ensure
      stream.close unless stream.closed?
    end
  end

  def self.pipeline_start(*commands, **_options)
    Open3.pipeline_r(*commands) { |stream, waiters| return [waiters] } unless block_given?
    Open3.pipeline_r(*commands) { |stream, waiters| yield waiters }
  end

  # The command as the shell reads it. An argument list is joined into one
  # line, and a leading environment Hash is written in front of it.
  def self.written_form(command)
    pieces = command.dup
    environment = pieces.first.is_a?(Hash) ? pieces.shift : {}
    named = environment.map { |key, value| "#{key}=#{value}" }
    (named + pieces.map { |piece| piece.to_s }).join " "
  end

  def self.joined_form(commands)
    commands.map { |command| Open3.written_form(Array(command)) }.join " | "
  end

  # A scratch file nobody else is using, for the stream being held apart.
  def self.scratch_name
    File.join Dir.tmpdir, "metorex_open3_#{Process.pid}_#{rand 1000000}"
  end

  # The thread each of these answers with. Metorex runs the command to
  # completion before handing anything back, so the thread has only the
  # status left to report.
  def self.waiter
    status = Process.last_status
    Thread.new { status }
  end

  # Run the command, writing `input` to it first when there is any.
  def self.run_capturing(written, input, _unused)
    return IO.popen(written) { |stream| stream.read } if input.nil?
    IO.popen(written) do |stream|
      stream.write input
      stream.read
    end
  end
end
