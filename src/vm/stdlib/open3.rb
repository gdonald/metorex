# Running a command and reaching its streams. Each of these starts the
# command with `Process.spawn`, joined to pipes this process holds the other
# ends of, alongside a thread whose value is the status the command ended
# with.

module Open3
  # The command's input, output and error streams, and its waiting thread.
  def self.popen3(*command, &block)
    words, options = Open3.split_command command
    in_read, in_write = IO.pipe
    out_read, out_write = IO.pipe
    err_read, err_write = IO.pipe
    options[:in] = in_read
    options[:out] = out_write
    options[:err] = err_write
    Open3.run words, options, [in_read, out_write, err_write], [in_write, out_read, err_read], &block
  end

  # The command's input and output streams.
  def self.popen2(*command, &block)
    words, options = Open3.split_command command
    in_read, in_write = IO.pipe
    out_read, out_write = IO.pipe
    options[:in] = in_read
    options[:out] = out_write
    Open3.run words, options, [in_read, out_write], [in_write, out_read], &block
  end

  # The command's input, and its output with its error stream joined on.
  def self.popen2e(*command, &block)
    words, options = Open3.split_command command
    in_read, in_write = IO.pipe
    out_read, out_write = IO.pipe
    options[:in] = in_read
    options[:out] = out_write
    options[:err] = [:child, :out]
    Open3.run words, options, [in_read, out_write], [in_write, out_read], &block
  end

  # What the command wrote and the status it ended with, having been handed
  # `stdin_data` to read.
  def self.capture2(*command, stdin_data: nil, binmode: false, **options)
    popen2(*command, **options) do |input, output, waiter|
      output.binmode if binmode
      Open3.feed input, stdin_data, binmode
      [output.read, waiter.value]
    end
  end

  # The same, with the error stream joined onto the output stream.
  def self.capture2e(*command, stdin_data: nil, binmode: false, **options)
    popen2e(*command, **options) do |input, output, waiter|
      output.binmode if binmode
      Open3.feed input, stdin_data, binmode
      [output.read, waiter.value]
    end
  end

  # The same again, with the two streams kept apart.
  def self.capture3(*command, stdin_data: nil, binmode: false, **options)
    popen3(*command, **options) do |input, output, errors, waiter|
      if binmode
        output.binmode
        errors.binmode
      end
      Open3.feed input, stdin_data, binmode
      [output.read, errors.read, waiter.value]
    end
  end

  # Each command reading what the one before it wrote, answering the status
  # each ended with.
  def self.pipeline(*commands, **options)
    Open3.spawn_pipeline(commands, options, nil, nil).map(&:value)
  end

  # The last command's output.
  def self.pipeline_r(*commands, **options, &block)
    out_read, out_write = IO.pipe
    waiters = Open3.spawn_pipeline commands, options, nil, out_write
    out_write.close
    Open3.hand_over [out_read], [out_read, waiters], waiters, &block
  end

  # The first command's input.
  def self.pipeline_w(*commands, **options, &block)
    in_read, in_write = IO.pipe
    waiters = Open3.spawn_pipeline commands, options, in_read, nil
    in_read.close
    in_write.sync = true
    Open3.hand_over [in_write], [in_write, waiters], waiters, &block
  end

  # The first command's input and the last command's output.
  def self.pipeline_rw(*commands, **options, &block)
    in_read, in_write = IO.pipe
    out_read, out_write = IO.pipe
    waiters = Open3.spawn_pipeline commands, options, in_read, out_write
    in_read.close
    out_write.close
    in_write.sync = true
    Open3.hand_over [in_write, out_read], [in_write, out_read, waiters], waiters, &block
  end

  # The commands started, with their input and output left as they were.
  def self.pipeline_start(*commands, **options, &block)
    waiters = Open3.spawn_pipeline commands, options, nil, nil
    Open3.hand_over [], [waiters], waiters, &block
  end

  # A command's words apart from how it is to be run, which a Hash at the
  # end names.
  def self.split_command(command)
    words = command.dup
    options = words.last.is_a?(Hash) && !(words.length == 1) ? words.pop.dup : {}
    [words, options]
  end

  # Start the command, close the ends only the child uses, and hand the rest
  # to the block, closing them and waiting for the child once it is done.
  def self.run(words, options, child_ends, parent_ends)
    pid = Process.spawn(*words, **options)
    child_ends.each(&:close)
    parent_ends.each { |stream| stream.nonblock = false }
    parent_ends[0].sync = true
    waiter = Open3.waiter pid
    handed = [*parent_ends, waiter]
    return handed unless block_given?
    begin
      yield(*handed)
    ensure
      parent_ends.each { |stream| stream.close unless stream.closed? }
      waiter.join
    end
  end

  # What a pipeline hands the block, closing the streams and waiting for
  # every command once it is done.
  def self.hand_over(streams, handed, waiters)
    streams.each { |stream| stream.nonblock = false }
    return handed unless block_given?
    begin
      yield(*handed)
    ensure
      streams.each { |stream| stream.close unless stream.closed? }
      waiters.each(&:join)
    end
  end

  # Start each command with its input joined to the output of the one before
  # it, answering a waiting thread for each.
  def self.spawn_pipeline(commands, options, input, output)
    reading = input
    waiters = []
    commands.each_with_index do |command, index|
      last = index == commands.length - 1
      next_read, writing = last ? [nil, output] : IO.pipe
      words, own = Open3.split_command(command.is_a?(Array) ? command : [command])
      own = options.merge(own)
      own[:in] = reading unless reading.nil?
      own[:out] = writing unless writing.nil?
      waiters << Open3.waiter(Process.spawn(*words, **own))
      reading.close unless reading.nil? || reading.equal?(input)
      writing.close unless last
      reading = next_read
    end
    waiters
  end

  # Write what the command is to read, then close its input so it sees the
  # end. A command that stopped reading early is no trouble.
  def self.feed(input, data, binmode)
    unless data.nil?
      data = data.to_s
      data = data.b if binmode
      begin
        input.write data
      rescue Errno::EPIPE
        nil
      end
    end
    input.close
  end

  # A thread whose value is the status the command ended with, and which
  # names the command's process id. It asks without waiting and hands over
  # its turn, so the other threads carry on while the command runs.
  def self.waiter(pid)
    thread = Thread.new do
      status = nil
      loop do
        _, status = Process.wait2 pid, Process::WNOHANG
        break unless status.nil?
        Thread.pass
        sleep 0.001
      end
      status
    end
    thread.define_singleton_method(:pid) { pid }
    thread
  end
end
