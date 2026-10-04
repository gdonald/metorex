pub(super) const SOURCE: &str = r##"

class IO
  SEEK_SET = 0
  SEEK_CUR = 1
  SEEK_END = 2
  # The name of the stream that keeps nothing it is given and reads back as
  # empty, which every system carries under this name.
  NULL = "/dev/null"

  # Which of the streams handed in have something to read, or room to write,
  # right now. Nothing is ready answers nil, which is what a caller waits on.
  # The streams among those named that are ready to be read or written. A
  # caller that named no timeout waits, and waiting is where every other
  # thread gets its turn.
  def self.select(readers = nil, writers = nil, errored = nil, timeout = nil)
    [readers, writers, errored].each do |given|
      next if given.nil? || given.is_a?(Array)
      raise TypeError, "wrong argument type #{given.class} (expected Array)"
    end
    [readers, writers, errored].each do |given|
      (given || []).each { |held| IO.__as_stream__ held }
    end
    deadline = nil
    unless timeout.nil?
      waited = timeout.is_a?(Numeric) ? timeout : Float(timeout)
      raise RangeError, "NaN out of Time range" if waited.is_a?(Float) && waited.nan?
      raise ArgumentError, "time interval must not be negative" if waited < 0
      deadline = Process.clock_gettime(Process::CLOCK_MONOTONIC) + waited
    end
    loop do
      ready_readers = (readers || []).select { |held| IO.__ready__ held, false }
      ready_writers = (writers || []).select { |held| IO.__ready__ held, true }
      unless ready_readers.empty? && ready_writers.empty?
        # Nothing is reported as being in error: a stream that cannot be read
        # or written says so by raising where it is used.
        return [ready_readers, ready_writers, []]
      end
      unless deadline.nil?
        return nil if Process.clock_gettime(Process::CLOCK_MONOTONIC) >= deadline
      end
      # Nothing is ready yet. Handing control over is what lets another
      # thread write to one of these streams or close it.
      break unless Thread.__hand_over__
    end
    nil
  end

  # The stream an argument names, which anything answering `to_io` gives.
  def self.__as_stream__(held)
    return held if held.is_a? IO
    unless held.respond_to? :to_io
      raise TypeError, "no implicit conversion of #{held.class} into IO"
    end
    stream = held.to_io
    unless stream.is_a? IO
      raise TypeError, "can't convert #{held.class} to IO"
    end
    stream
  end

  def self.__ready__(held, writing)
    stream = held.respond_to?(:to_io) ? held.to_io : held
    # A socket keeps its own handle rather than a stream's, and the system is
    # asked about that one.
    if defined?(BasicSocket) && stream.is_a?(BasicSocket)
      return true if stream.closed?
      return Socket.__net__("ready?", stream.handle, "", writing ? 1 : 0)
    end
    return true unless stream.is_a? IO
    return true if stream.__stream_handle__.nil?
    IO.__stream__ "ready?", stream.__stream_handle__, "", writing ? 1 : 0
  rescue IOError
    false
  end

  # Two joined streams: what is written to the second is read from the first.
  # With a block the pair is handed over and closed once the block is done.
  # Two joined streams: what is written to the second is read from the first.
  # A subclass gets two of its own, built without going through `new`, so a
  # subclass that rewrites `new` does not decide how a pipe is made.
  def self.pipe(external = nil, internal = nil, **options)
    reading, writing = IO.__stream__ "pipe", 0, "", 0
    pair = [__over__(reading, nil, "r"), __over__(writing, nil, "w")]
    # The encodings a pipe is opened with are the read end's: what comes out
    # of it is what was written in.
    unless external.nil? && internal.nil? && options.empty?
      named = external
      if !named.nil? && !named.is_a?(Encoding) && !named.is_a?(String) && named.respond_to?(:to_str)
        named = named.to_str
      end
      pair[0].set_encoding named, internal, **options
    end
    return pair unless block_given?
    begin
      yield pair[0], pair[1]
    ensure
      pair.each { |held| held.close unless held.closed? }
    end
  end

  # An IO over a handle the interpreter already holds.
  def self.__over__(handle, path = nil, mode = nil)
    held = allocate
    held.__send__ :__take__, handle, path, mode
    # A subclass that writes its own `initialize` has it run the way Ruby
    # runs one behind `new`, without going through `new` itself.
    if instance_method(:initialize).owner != IO
      held.__send__ :initialize, held.fileno, mode.nil? ? "r" : mode
    end
    held
  end

  # The whole of a file, or a run of it, named by path. `File` reads these
  # itself, and an IO reads them the same way.
  def self.read(name, *rest, **options)
    File.read name, *rest, **options
  end

  def self.binread(name, *rest)
    File.binread name, *rest
  end

  # Text written to a file named by path. Without an offset the file is
  # written from the start and cut down to what was written, and with one the
  # rest of the file is left as it was. `open_args:` names everything the file
  # is opened with, in place of the other options.
  def self.write(name, text, offset = :__none__, *extra, **options)
    unless extra.empty?
      raise ArgumentError,
            "wrong number of arguments (given #{3 + extra.size}, expected 2..3)"
    end
    spelled = text.is_a?(String) ? text : text.to_s
    at = offset == :__none__ ? nil : offset
    path = File.path(name)
    brought_into_being = !File.exist?(path)
    opened_with = options[:open_args]
    held = if !opened_with.nil?
      given = opened_with.to_a
      if given.last.is_a? Hash
        File.open path, *given[0..-2], **given.last
      else
        File.open path, *given
      end
    else
      mode = options[:mode]
      mode = at.nil? ? "w" : File::WRONLY | File::CREAT if mode.nil?
      File.open path, mode, **options.reject { |key, _| key == :mode || key == :perm }
    end
    written = begin
      held.seek at, IO::SEEK_SET unless at.nil?
      held.write spelled
    ensure
      held.close
    end
    # A permission named here stands for the file the write brought into
    # being, and says nothing about one that was already there.
    if brought_into_being && opened_with.nil? && !options[:perm].nil?
      File.chmod options[:perm], path
    end
    written
  end

  # The bytes a String stands for written to a file, with nothing carried
  # into another encoding on the way.
  def self.binwrite(name, text, offset = :__none__, *extra, **options)
    IO.write name, text, offset, *extra, **options
  end

  def self.readlines(name, separator = $/, limit = nil, chomp: false, **options)
    held = File.open File.path(name), options[:mode].nil? ? "r" : options[:mode]
    begin
      held.readlines separator, limit, chomp: chomp
    ensure
      held.close
    end
  end

  # Each line of a file in turn. Without a block the lines are handed back as
  # a walk over them.
  def self.foreach(name, separator = :__none__, limit = nil, chomp: false, **options, &block)
    if block.nil?
      return Enumerator.new do |yielder|
        IO.foreach(name, separator, limit, chomp: chomp, **options) { |line| yielder << line }
      end
    end
    held = File.open File.path(name), options[:mode].nil? ? "r" : options[:mode]
    # Reading every line of a file leaves no last line read behind.
    $_ = nil
    begin
      if separator == :__none__
        held.each_line(chomp: chomp) { |line| block.call line }
      else
        held.each_line(separator, limit, chomp: chomp) { |line| block.call line }
      end
    ensure
      held.close
    end
    nil
  end

  # The stream an object stands for, or nil where it stands for none. Only an
  # object answering `to_io` is asked.
  def self.try_convert(held)
    # `IO === held` asks IO rather than the object, so an object that answers
    # nothing of Kernel's, a BasicObject among them, is read here too.
    return held if IO === held
    converted = begin
      held.to_io
    rescue NoMethodError => missing
      raise unless missing.name == :to_io
      return nil
    end
    return converted if IO === converted
    raise TypeError,
          "can't convert #{held.class} into IO (#{held.class}#to_io gives #{converted.class})"
  end

  # An IO over a descriptor, handed to a block when one is given and closed
  # once the block is done.
  def self.open(number, mode = nil, *extra, **options)
    held = new number, mode, *extra, **options
    return held unless block_given?
    begin
      yield held
    ensure
      begin
        held.close unless held.closed?
      rescue IOError => closing
        # A stream already closed inside the block is not a failure of the
        # close here, so that one error alone is let go.
        raise unless closing.message == "closed stream"
      end
    end
  end

  # The descriptor a name opens under, handed back by number. Whoever asked
  # for it owns it, so nothing here closes it.
  def self.sysopen(name, mode = nil, permissions = nil)
    written = IO.__written_mode__(mode).to_s
    written = "r" if written.empty?
    IO.__stream__ "sysopen", 0, File.path(name), IO.__opening_number__(written)
  end

  # How a mode string says the file is opened: 0 reads, 1 writes from the
  # start, 2 adds to the end, and 3 does both.
  def self.__opening_number__(written)
    return 2 if written.start_with? "a"
    return 3 if written.start_with?("r") && written.include?("+")
    return 1 if written.start_with? "w"
    0
  end

  # A mode as a String, whatever it was written as: a String stands as it is,
  # a number names the flags, and anything else spells itself as one.
  def self.__written_mode__(mode)
    return nil if mode.nil?
    return mode if mode.is_a? String
    return IO.__mode_of_flags__(mode) if mode.is_a? Integer
    return mode.to_str if mode.respond_to? :to_str
    return IO.__mode_of_flags__(mode.to_int) if mode.respond_to? :to_int
    raise ArgumentError, "invalid access mode #{mode}"
  end

  # The mode string the open flags stand for.
  def self.__mode_of_flags__(flags)
    access = flags & 3
    adding = flags & File::APPEND != 0
    return adding ? "a+" : "r+" if access == File::RDWR
    return adding ? "a" : "w" if access == File::WRONLY
    "r"
  end

  # `IO.new(fd)` stands over a descriptor this program did not open. The mode
  # says how the program means to use it, which must be a use the descriptor
  # was opened for, and the options name the encodings and whether the
  # descriptor is closed along with the IO.
  def initialize(number, mode = nil, *extra, **options)
    unless extra.empty?
      raise ArgumentError,
            "wrong number of arguments (given #{2 + extra.size}, expected 1..2)"
    end
    if block_given?
      warn "warning: IO::new() does not take block; use IO::open() instead"
    end
    number = IO.__as_descriptor__ number
    written = IO.__mode_wanted__ mode, options
    handle = IO.__stream__ "adopt", 0, "", number
    opened = IO.__stream__ "accmode", handle, "", 0
    if written.nil?
      # Nothing said how the stream would be used, so it is used the way the
      # descriptor was opened.
      written = IO.__mode_of_flags__ opened
    else
      # A descriptor opened for reading cannot be written through, and one
      # opened for writing cannot be read.
      IO.__check_access__ written, opened
    end
    __take__ handle, options[:path], written
    self.autoclose = options.key?(:autoclose) ? (options[:autoclose] ? true : false) : true
    @binmode = true if IO.__binary_mode__ written, options
    @__file_encoding = IO.__named_encoding__ written, options
    self
  end

  # The descriptor a program named, which is a number or something that
  # spells itself as one.
  def self.__as_descriptor__(number)
    return number if number.is_a? Integer
    unless number.respond_to? :to_int
      raise TypeError, "no implicit conversion of #{number.class} into Integer"
    end
    held = number.to_int
    unless held.is_a? Integer
      raise TypeError, "can't convert #{number.class} to Integer"
    end
    held
  end

  # The mode a program asked for, named either as an argument or as a `mode:`
  # option, but never as both.
  def self.__mode_wanted__(mode, options)
    named = options[:mode]
    if !mode.nil? && !named.nil?
      raise ArgumentError, "mode specified twice"
    end
    given = named.nil? ? mode : named
    return nil if given.nil?
    written = IO.__written_mode__(given).to_s
    raise ArgumentError, "invalid access mode #{written}" if written.empty?
    written
  end

  # Whether the stream reads and writes bytes rather than text. A mode
  # naming binary or text and an option saying so are two ways of asking for
  # the same thing, and Ruby takes only one of them.
  def self.__binary_mode__(written, options)
    access = written.split(":", 2)[0].to_s
    named = access.include?("b") || access.include?("t")
    if named && (options.key?(:binmode) || options.key?(:textmode))
      raise ArgumentError, "binmode specified twice"
    end
    if options[:binmode] && options[:textmode]
      raise ArgumentError, "both textmode and binmode specified"
    end
    return true if access.include? "b"
    options[:binmode] ? true : false
  end

  # The encodings a stream reads and writes in, written as `"ext:int"`. They
  # may be named after the mode or as options, and never as both.
  def self.__named_encoding__(written, options)
    parts = written.split(":")
    named = parts[1..].to_a.join(":")
    keyed = [:encoding, :external_encoding, :internal_encoding].any? { |key| options.key? key }
    if !named.empty? && keyed
      raise ArgumentError, "encoding specified twice"
    end
    unless named.empty?
      return IO.__encoding_pair__ parts[1], parts[2], written
    end
    outer = options[:external_encoding]
    inner = options[:internal_encoding]
    if options.key?(:encoding)
      if outer.nil? && inner.nil?
        outer, inner = IO.__spelled_encoding__(options[:encoding]).split(":", 2)
      else
        warn "warning: Ignoring encoding parameter '#{options[:encoding]}': #{outer.nil? ? "internal" : "external"}_encoding is used"
      end
    end
    if outer.nil? && inner.nil?
      return "ASCII-8BIT" if IO.__binary_mode__ written, options
      return nil
    end
    IO.__encoding_pair__ outer, inner, written
  end

  # Two encodings as one `"ext:int"` string. An internal encoding matching
  # the external one, or named `-`, is no internal encoding at all.
  def self.__encoding_pair__(outer, inner, written)
    outer = outer.nil? ? "" : IO.__spelled_encoding__(outer)
    inner = inner.nil? ? "" : IO.__spelled_encoding__(inner)
    inner = "" if inner == "-" || inner.downcase == outer.downcase
    inner.empty? ? outer : "#{outer}:#{inner}"
  end

  # An encoding as the name it is held under, whatever it was written as.
  def self.__spelled_encoding__(named)
    return named.name if named.is_a? Encoding
    return named if named.is_a? String
    return named.to_str if named.respond_to? :to_str
    named.to_s
  end

  # Whether the descriptor may be used the way the mode says.
  def self.__check_access__(written, opened)
    access = written.split(":", 2)[0].to_s
    reading = access.start_with?("r") || access.include?("+")
    writing = access.start_with?("w") || access.start_with?("a") || access.include?("+")
    if (reading && opened == File::WRONLY) || (writing && opened == File::RDONLY)
      raise Errno::EINVAL, "invalid access mode #{written}"
    end
    nil
  end

  def self.for_fd(number, mode = nil, **options)
    new number, mode, **options
  end

  # Everything one stream holds, written to another. A name stands for a file
  # opened for the copy and closed after it, and an object that reads or
  # writes is used as it is.
  def self.copy_stream(source, destination, length = nil, offset = nil)
    opened_source = nil
    opened_target = nil
    begin
      reader = if source.respond_to?(:readpartial) || source.respond_to?(:read)
        source
      else
        opened_source = File.open(File.path(source), "rb")
      end
      writer = destination.respond_to?(:write) ? destination : (opened_target = File.open(File.path(destination), "wb"))
      # A whole stream is passed on as it arrives, so a copy between two
      # pipes carries each piece over without waiting for the end.
      if length.nil? && offset.nil? && reader.is_a?(IO)
        raise IOError, "not opened for reading" unless reader.__send__ :__readable__
        copied = 0
        loop do
          piece = begin
            reader.readpartial 16384
          rescue EOFError
            break
          end
          writer.write piece
          writer.flush if writer.is_a? IO
          copied += piece.bytesize
        end
        return copied
      end
      held = __copied_text__ reader, length, offset
      writer.write held
      held.bytesize
    ensure
      opened_source.close unless opened_source.nil?
      opened_target.close unless opened_target.nil?
    end
  end

  # Everything left in the stream. One read hands over what the operating
  # system had ready, which for a file of any size is less than all of it, so
  # it is asked again until there is nothing more.
  def __read_to_the_end__
    collected = +""
    loop do
      piece = __stream_read__(0).to_s
      break if piece.empty?
      collected = collected + piece
    end
    collected
  end
  private :__read_to_the_end__

  # The text a copy reads. A stream reads through its own position, which an
  # offset names a place apart from and leaves where it was.
  def self.__copied_text__(reader, length, offset)
    unless reader.is_a? IO
      return __read_in_pieces__(reader, length)
    end
    raise IOError, "not opened for reading" unless reader.__send__ :__readable__
    return __read_limited__(reader, length) if offset.nil?
    standing = reader.pos
    begin
      reader.pos = offset
      __read_limited__ reader, length
    ensure
      reader.pos = standing
    end
  end

  # What a stream hands over, up to the count asked for.
  def self.__read_limited__(reader, length)
    held = length.nil? ? reader.read : reader.read(length)
    held.nil? ? "" : held
  end

  # What an object that is not a stream hands over, asked for a piece at a
  # time the way Ruby asks.
  def self.__read_in_pieces__(reader, length)
    collected = +""
    buffer = +""
    partial = reader.respond_to? :readpartial
    begin
      loop do
        wanted = length.nil? ? COPY_PIECE : [COPY_PIECE, length - collected.bytesize].min
        break if wanted <= 0
        piece = partial ? reader.readpartial(wanted, buffer) : reader.read(wanted, buffer)
        break if piece.nil? || piece.empty?
        collected = collected + piece
        break if !length.nil? && collected.bytesize >= length
      end
    rescue EOFError
    end
    collected
  end

  # How much a copy reads at a time from an object that is not a stream.
  COPY_PIECE = 16384

  # The three streams the program started with, each over the descriptor the
  # operating system opened for it.
  def self.__standard__(number, named)
    held = __over__ IO.__stream__("adopt", 0, "", number), named, number == 0 ? "r" : "w"
    held.__send__ :__name_standard__, named
    held
  end

  def __name_standard__(named)
    @standard = named
    # Ruby writes the error stream straight through rather than holding what
    # is written back, which is what `sync` reports for it.
    @sync = true if named == "stderr"
    self
  end

  def __take__(handle, path = nil, mode = nil)
    __note_encodings__
    @handle = handle
    @path = path
    @__file_mode = mode
    @closed = false
    @autoclose = true
    @lineno = 0
    @sync = false
    self
  end


  def __stream_handle__
    @handle
  end

  # The number the operating system holds this stream under.
  def fileno
    raise IOError, "closed stream" if closed?
    raise IOError, "uninitialized stream" if __stream_handle__.nil?
    IO.__stream__ "fileno", __stream_handle__, "", 0
  end

  alias_method :to_i, :fileno

  # The file this stream was opened over, where it was opened over one.
  def path
    @path
  end

  # A stream not reading from a child process has no process to name.
  def pid
    raise IOError, "closed stream" if closed?
    @__popen_pid
  end

  # A frozen stream keeps no flag of its own, so the descriptor it held
  # says whether it was closed.
  def closed?
    return true if @closed == true
    frozen? && !@handle.nil? && !IO.__stream__("open?", @handle, "", 0)
  end

  # A stream closes the descriptor it holds unless it was told not to, which
  # only a stream built over a descriptor from outside is.
  def autoclose?
    raise IOError, "closed stream" if closed?
    @autoclose.nil? ? true : @autoclose
  end

  def autoclose=(wanted)
    raise IOError, "closed stream" if closed?
    @autoclose = wanted ? true : false
  end

  def close
    return nil if closed?
    # What the stream was holding back is written before the descriptor goes,
    # so a broken pipe is reported here rather than lost.
    begin
      __drain__
    ensure
      @__popen_writer.close unless @__popen_writer.nil? || @__popen_writer.closed?
      IO.__stream__ "close", __stream_handle__, "", 0 if autoclose?
      @closed = true unless frozen?
    end
    # A stream joined to a forked child waits for it, which is what sets `$?`.
    Process.waitpid @__popen_pid unless @__popen_pid.nil?
    nil
  end

  # `IO.popen("-")` forks. The child answers nil with its standard output
  # and input joined to pipes, and the parent answers a stream over the other
  # ends, reading what the child writes and writing what the child reads.
  def self.__popen_fork__(mode = "r")
    mode = mode.to_str unless mode.is_a? String
    reads = mode.include?("r") || mode.include?("+")
    writes = mode.include?("w") || mode.include?("+")
    from_child = IO.pipe if reads
    to_child = IO.pipe if writes
    pid = Process._fork
    if pid == 0
      if reads
        from_child[0].close
        STDOUT.reopen from_child[1]
        from_child[1].close
      end
      if writes
        to_child[1].close
        STDIN.reopen to_child[0]
        to_child[0].close
      end
      return nil
    end
    from_child[1].close if reads
    to_child[0].close if writes
    joined = reads ? from_child[0] : to_child[1]
    joined.__send__ :__popen_forked__, pid, (reads && writes ? to_child[1] : nil), mode
    joined
  end

  # A command run as a child process, with its standard output joined to the
  # stream answered for reading, its standard input for writing, or both for
  # "r+". A Hash in front names the child's environment and one behind names
  # how it is run, as `spawn` reads them, alongside the stream's own options.
  # With a block the stream is handed over and closed once the block is done,
  # which waits for the child.
  def self.popen(*given, **options, &block)
    given = given.dup
    environment = given.first.is_a?(Hash) ? given.shift : nil
    # Options may come as a Hash written as the last argument as well as by
    # keyword.
    options = given.pop.merge(options) if given.length > 1 && given.last.is_a?(Hash)
    command = given.shift
    raise ArgumentError, "wrong number of arguments (given 0, expected 1+)" if command.nil?
    mode = given.shift
    mode = options.delete(:mode) if mode.nil?
    mode = mode.nil? ? "r" : (mode.is_a?(String) ? mode : mode.to_str)
    stream_options = {}
    [:external_encoding, :internal_encoding, :encoding, :binmode].each do |key|
      stream_options[key] = options.delete(key) if options.key? key
    end
    if command == "-"
      stream = __popen_fork__(mode)
      # The child hands nil to the block and ends once the block is done,
      # whatever the block did.
      if stream.nil? && !block.nil?
        begin
          block.call nil
        rescue Exception
          nil
        end
        $stdout.flush
        $stderr.flush
        exit! 0
      end
      return __popen_block__(stream, &block)
    end
    words = if command.is_a? Array
      held = command.dup
      environment = (environment || {}).merge(held.shift) if held.first.is_a? Hash
      options = held.pop.merge(options) if held.last.is_a? Hash
      held
    else
      [command.is_a?(String) ? command : command.to_str]
    end
    reads = mode.include?("r") || mode.include?("+")
    writes = mode.include?("w") || mode.include?("+")
    from_child = IO.pipe if reads
    to_child = IO.pipe if writes
    redirects = {}
    redirects[:out] = from_child[1] if reads
    redirects[:in] = to_child[0] if writes
    spawned = environment.nil? ? [] : [environment]
    begin
      pid = Process.spawn(*spawned, *words, **options.merge(redirects))
    ensure
      from_child[1].close if reads
      to_child[0].close if writes
    end
    joined = reads ? from_child[0] : to_child[1]
    # Reading waits for the child to write, and writing for it to read.
    joined.nonblock = false
    to_child[1].nonblock = false if reads && writes
    stream = __over__ joined.__stream_handle__, nil, mode
    stream.__send__ :__popen_forked__, pid, (reads && writes ? to_child[1] : nil), mode
    unless stream_options[:external_encoding].nil? && stream_options[:internal_encoding].nil?
      stream.set_encoding stream_options[:external_encoding], stream_options[:internal_encoding]
    end
    stream.set_encoding stream_options[:encoding] unless stream_options[:encoding].nil?
    stream.binmode if stream_options[:binmode]
    __popen_block__ stream, &block
  end

  # The stream `popen` made, or what a block handed it answers. The stream is
  # closed once the block is done, whatever the block did.
  def self.__popen_block__(stream)
    return stream unless block_given?
    begin
      yield stream
    ensure
      stream.close unless stream.nil? || stream.closed?
    end
  end

  def __popen_forked__(pid, writer, mode)
    @__popen_pid = pid
    @__popen_writer = writer
    @__file_mode = mode
    self
  end
  private :__popen_forked__

  # A stream with two ends may have one of them closed on its own. A stream
  # with a single end refuses, which is what Ruby does for a file.
  def __duplex__
    !@__popen_input.nil? || !@__popen_writer.nil?
  end

  # Whether this stream was opened only for the side being closed, in which
  # case closing that side closes the stream itself.
  def __opened_for__(letter)
    @__file_mode.to_s.start_with? letter
  end

  # Whether the stream was opened for both sides, which is what a mode
  # carrying a plus says.
  def __both_ways__
    @__file_mode.to_s.include? "+"
  end

  # Whether reading is allowed at all. A mode naming only writing or only
  # appending leaves no reading side.
  def __readable__
    return false if @standard == "stdout" || @standard == "stderr"
    return true if @__file_mode.nil?
    return true if __both_ways__
    !(__opened_for__("w") || __opened_for__("a"))
  end
"##;
