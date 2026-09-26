pub(super) const SOURCE: &str = r##"
  # The encoding read text is carried into. A stream carries text nowhere
  # when the two encodings are the same, or when it reads bytes.
  def internal_encoding
    named = __named_encodings__[1]
    return Encoding.find(named) unless named.nil? || named.empty?
    return nil if @__made_internal.nil?
    outer = external_encoding
    return nil if outer.nil? || outer == Encoding::BINARY
    return nil if outer == @__made_internal
    @__made_internal
  end

  # The encodings the stream reads text as, named either as two arguments or
  # as one `"ext:int"` string.
  def set_encoding(external, internal = nil, **options)
    # A stream told to read every line ending the same way carries text
    # rather than bytes, which is what refuses a byte-wise read afterwards.
    @__newline_conversion = options[:universal_newline] ? true : false
    # What to do with a byte the encoding cannot read is remembered, so every
    # read that carries text over is told the same thing.
    @__encoding_options = options.reject { |name, _| name == :universal_newline }
    # Naming neither encoding puts the stream back to following the
    # program's own, as they stand now.
    if external.nil? && internal.nil?
      @__file_encoding = nil
      @__encodings_reset = true
      @__noted_encodings = nil
      __note_encodings__
      return self
    end
    @__encodings_reset = nil
    @__file_encoding = IO.__encoding_pair__(
      *(internal.nil? && external.is_a?(String) && external.include?(":") ? external.split(":", 2) : [external, internal]),
      ""
    )
    self
  end

  # Text read from the stream is tagged with the encoding the stream reads
  # as, and carried into the internal one where the stream names one.
  def __tag_read__(text)
    return text if text.nil? || text.empty?
    inner = internal_encoding
    outer = external_encoding
    # A stream naming no encoding of its own reads its text in the one the
    # program reads by.
    tagged = text.dup.force_encoding(outer.nil? ? Encoding.default_external : outer)
    return tagged if inner.nil?
    # A stream reading bytes hands them over as they are, whatever encoding
    # it was told to carry them into.
    return tagged if !outer.nil? && outer == Encoding::BINARY
    held = @__encoding_options
    return tagged.encode(inner) if held.nil? || held.empty?
    tagged.encode(inner, **held)
  end
  private :__tag_read__

  def readlines(separator = $/, limit = nil, chomp: false)
    separator, limit = StringIO.line_arguments separator, limit
    raise ArgumentError, "invalid limit: 0 for readlines" if limit == 0
    collected = []
    while (held = __read_line__(separator, limit, chomp))
      collected.push held
    end
    collected
  end

  def each_line(separator = $/, limit = nil, chomp: false, &block)
    return to_enum(:each_line, separator, limit, chomp: chomp) if block.nil?
    separator, limit = StringIO.line_arguments separator, limit
    raise ArgumentError, "invalid limit: 0 for each_line" if limit == 0
    while (held = __read_line__(separator, limit, chomp))
      block.call held
    end
    self
  end

  alias_method :each, :each_line

  # One byte at the cursor, or nil where the stream has no more. A stream
  # opened only for writing has no reading side at all.
  def getbyte
    raise IOError, "not opened for reading" unless __readable__
    waiting = @peeked
    unless waiting.nil? || waiting.empty?
      listed = waiting.bytes
      first = listed.shift
      @peeked = listed.empty? ? nil : listed.pack("C*")
      return first
    end
    held = read 1
    return nil if held.nil? || held.empty?
    held.bytes.first
  end

  # An argument read as an Integer, which anything answering `to_int` can be.
  def __as_integer__(held)
    return held if held.is_a? Integer
    unless held.respond_to? :to_int
      # Ruby names nothing by its class, so a nil is reported as `nil`.
      named = held.nil? ? "nil" : held.class.to_s
      raise TypeError, "no implicit conversion of #{named} into Integer"
    end
    held.to_int
  end
  private :__as_integer__

  # A read at a named offset, which leaves the cursor where it was.
  def pread(maxlen, offset, buffer = nil)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    wanted = __as_integer__ maxlen
    at = __as_integer__ offset
    raise ArgumentError, "negative string size (or size too big)" if wanted < 0
    raise Errno::EINVAL, "Invalid argument" if at < 0
    target = nil
    unless buffer.nil?
      if buffer.is_a? String
        target = buffer
      elsif buffer.respond_to? :to_str
        target = buffer.to_str
      else
        raise TypeError, "no implicit conversion of #{buffer.class} into String"
      end
    end
    return buffer.nil? ? "" : buffer if wanted == 0
    held = pos
    self.pos = at
    read_back = read wanted
    self.pos = held
    raise EOFError, "end of file reached" if read_back.nil? || read_back.empty?
    return read_back if buffer.nil?
    named = target.encoding
    target.replace read_back
    target.force_encoding named
    buffer
  end

  # A write at a named offset, which leaves the cursor where it was.
  def pwrite(held, offset)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for writing" unless __writable__
    text = held.to_s
    at = __as_integer__ offset
    standing = pos
    self.pos = at
    write text
    self.pos = standing
    text.bytesize
  end

  # Whether writing is allowed at all. A mode naming only reading leaves no
  # writing side.
  def __writable__
    return true if @__file_mode.nil?
    return true if __both_ways__
    !__opened_for__("r")
  end

  # A hint about how the file will be read. Nothing is passed on to the
  # system, so what is left is refusing what Ruby refuses.
  def advise(kind, offset = 0, length = 0)
    raise IOError, "closed stream" if closed?
    raise TypeError, "advice must be a Symbol" unless kind.is_a? Symbol
    allowed = [:normal, :sequential, :random, :willneed, :dontneed, :noreuse]
    raise NotImplementedError, "Unsupported advice: #{kind}" unless allowed.include? kind
    [offset, length].each do |held|
      unless held.is_a? Integer
        raise TypeError, "no implicit conversion of #{held.class} into Integer"
      end
      if held > 9223372036854775807 || held < -9223372036854775808
        raise RangeError, "bignum too big to convert into 'long long'"
      end
    end
    nil
  end

  # Bytes put back are read again before anything else in the stream. An
  # Integer names one byte, and only its low eight bits are kept.
  def ungetbyte(held)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    return nil if held.nil?
    text = if held.is_a? Integer
      [held & 0xff].pack("C")
    elsif held.is_a? String
      held
    elsif held.respond_to? :to_str
      held.to_str
    else
      raise TypeError, "no implicit conversion of #{held.class} into String"
    end
    @peeked = @peeked.nil? ? text : text + @peeked
    nil
  end

  def readbyte
    held = getbyte
    raise EOFError, "end of file reached" if held.nil?
    held
  end

  # Every byte in turn. Without a block the walk is handed back, and it
  # cannot say how many bytes are left to come.
  def each_byte
    return Enumerator.over(self, :each_byte) unless block_given?
    raise IOError, "closed stream" if closed?
    while (held = getbyte)
      yield held
    end
    self
  end

  # One character at the cursor. A character spelled in several bytes is read
  # to its end rather than cut in half.
  # One character, which is as many bytes as the character is spelled with.
  def getc
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    waiting = @peeked
    unless waiting.nil? || waiting.empty?
      first = waiting[0]
      rest = waiting[1, waiting.length - 1]
      @peeked = rest.nil? || rest.empty? ? nil : rest
      return first
    end
    outer = external_encoding
    held = IO.__stream__ "getc", __stream_handle__, outer.nil? ? "" : outer.name, 0
    return nil if held.nil? || held.empty?
    __tag_read__ held
  end

  def readchar
    held = getc
    raise EOFError, "end of file reached" if held.nil?
    held
  end

  def each_char
    return Enumerator.over(self, :each_char) unless block_given?
    raise IOError, "closed stream" if closed?
    while (held = getc)
      yield held
    end
    self
  end

  def each_codepoint
    return Enumerator.over(self, :each_codepoint) unless block_given?
    each_char { |held| yield held.ord }
    self
  end

  alias_method :codepoints, :each_codepoint

  # How many lines have been read. Only a stream being read counts them, so
  # one that cannot be read has none to report.
  def lineno
    __reading_side__
    @lineno.nil? ? 0 : @lineno
  end

  def lineno=(held)
    __reading_side__
    counted = __as_integer__ held
    # The line count is one the operating system holds, which is as wide as
    # a C int and no wider.
    if counted.bit_length > 31
      raise RangeError, "integer #{counted} too big to convert to `int'"
    end
    @lineno = counted
  end

  # Refuse a stream that has no reading side: a closed one, one opened only
  # to write, and one whose reading side was closed.
  def __reading_side__
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" if @read_closed
    # A stream over a child process keeps its own mark for the side that was
    # closed, since the interpreter is what closed it.
    if @__popen_read_closed
      raise IOError, "not opened for reading"
    end
    raise IOError, "not opened for reading" unless __readable__
    nil
  end
  private :__reading_side__

  # Reading one character ahead is the only way to tell a stream that has
  # nothing left from one whose writer has not written yet, so the character
  # is held back for the next read to hand out.
  def eof?
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" if @read_closed
    raise IOError, "not opened for reading" if __opened_for__("w") && !__both_ways__
    return false unless @peeked.nil?
    held = IO.__stream__ "read", __stream_handle__, "", 1
    return true if held.nil? || held.empty?
    @peeked = held
    false
  end

  alias_method :eof, :eof?

  # Metorex hands every write to the operating system as it is made, so
  # nothing is ever waiting to be flushed.
  def flush
    raise IOError, "closed stream" if closed?
    __drain__
    @line_buffered = nil
    self
  end

  def fsync
    raise IOError, "closed stream" if closed?
    0
  end

  def sync
    raise IOError, "closed stream" if closed?
    @sync == true
  end

  def sync=(wanted)
    raise IOError, "closed stream" if closed?
    @sync = wanted ? true : false
    # Only a program asking for it makes a stream hold what is written.
    @holding = !@sync
  end

  def tty?
    raise IOError, "closed stream" if closed?
    IO.__stream__ "tty?", __stream_handle__, "", 0
  end

  alias_method :isatty, :tty?

  def nonblock?
    raise IOError, "closed stream" if closed?
    IO.__stream__ "nonblock?", __stream_handle__, "", 0
  end

  def nonblock=(wanted)
    IO.__stream__ "nonblock=", __stream_handle__, "", wanted ? 1 : 0
    wanted
  end

  def nonblock(wanted = true)
    was = nonblock?
    self.nonblock = wanted
    return self unless block_given?
    begin
      yield self
    ensure
      self.nonblock = was
    end
  end

  def close_on_exec?
    raise IOError, "closed stream" if closed?
    IO.__stream__ "close_on_exec?", __stream_handle__, "", 0
  end

  def close_on_exec=(wanted)
    raise IOError, "closed stream" if closed?
    IO.__stream__ "close_on_exec=", __stream_handle__, "", wanted ? 1 : 0
    wanted
  end

  # Reading and writing bytes rather than characters: the stream is written
  # in binary from here on, and nothing is converted on the way in.
  def binmode
    raise IOError, "closed stream" if closed?
    @binmode = true
    @__file_encoding = "ASCII-8BIT"
    self
  end

  # A stream is in binary mode once `binmode` was called, or when the mode it
  # was opened with says `b`.
  def binmode?
    raise IOError, "closed stream" if closed?
    @binmode == true || @__file_mode.to_s.split(":", 2)[0].to_s.include?("b")
  end

  # Lock the whole file, or let a lock go. A lock asked for with `LOCK_NB`
  # is refused rather than waited on, which answers false.
  def flock(kind)
    raise IOError, "closed stream" if closed?
    wanted = kind.to_i
    if wanted & File::LOCK_NB != 0 || wanted & File::LOCK_UN != 0
      return IO.__stream__("flock", __stream_handle__, "", wanted)
    end
    # Waiting for the lock leaves the other threads running, so the wait is
    # made of asks that do not block.
    loop do
      held = IO.__stream__ "flock", __stream_handle__, "", wanted | File::LOCK_NB
      return held unless held == false
      sleep 0.01
    end
  end

  # Ask the operating system about this stream's descriptor, or set one of
  # the flags it keeps. `Fcntl` names the numbers.
  def fcntl(command, argument = 0)
    raise IOError, "closed stream" if closed?
    held = argument == true ? 1 : (argument == false || argument.nil? ? 0 : argument.to_i)
    IO.__stream__ "fcntl", __stream_handle__, "", command.to_i, held
  end

  # Ask the device behind this stream's descriptor to do something. A String
  # handed over is both what the request reads and where its answer is
  # written, so it comes back holding what the device put there.
  def ioctl(request, argument = 0)
    raise IOError, "closed stream" if closed?
    if argument.is_a? String
      # The request writes its answer into the buffer, so one with no room
      # in it is given some first. Ruby hands the buffer back holding what
      # the device wrote there.
      room = argument.empty? ? "\x00" * 8 : argument
      held = IO.__stream__ "ioctl", __stream_handle__, room, request.to_i, 0
      argument.replace held[1]
      return held[0]
    end
    number = argument == true ? 1 : (argument == false || argument.nil? ? 0 : argument.to_i)
    IO.__stream__("ioctl", __stream_handle__, "", request.to_i, number)[0]
  end

  # Point this stream at another place. The descriptor keeps its number, so
  # everything already reading or writing through it reaches the new place.
  def reopen(target, mode = nil)
    # A closed stream may be opened again over a file named by path, which is
    # what reopening one is for. Pointed at another stream it stays closed.
    named = target.is_a?(String) || (!target.is_a?(IO) && target.respond_to?(:to_path))
    raise IOError, "closed stream" if closed? && !named
    other = __reopen_target__ target, mode
    raise IOError, "closed stream" if other.closed?
    # A closed stream is opened again over what it was pointed at, which is
    # what reopening one is for. A stream still open keeps its descriptor,
    # so everything already reading or writing through it follows along.
    if closed?
      @handle = other.__stream_handle__
      @closed = false
    else
      IO.__stream__ "reopen", __stream_handle__, "", other.__stream_handle__
    end
    @__file_path = other.path
    @__file_mode = mode.nil? ? other.instance_variable_get(:@__file_mode) : mode
    @read_closed = false
    @write_closed = false
    @peeked = nil
    @lineno = 0
    __fresh_singleton_class__ other
    # The standard streams keep the flag they were given, so a program that
    # points STDOUT at a file still hands that file to what it runs.
    self.close_on_exec = true if fileno > 2
    self
  end

  # The stream a `reopen` was given. A name opens a file, and anything else
  # spells itself out as the stream it stands for.
  def __reopen_target__(target, mode)
    if target.is_a?(String) || (!target.is_a?(IO) && target.respond_to?(:to_path))
      path = target.is_a?(String) ? target : target.to_path
      # Written with no mode of its own, the file is opened the way this
      # stream already was, so one opened for writing makes the file.
      wanted = mode.nil? ? __reopen_mode__ : mode
      return File.open(path, wanted)
    end
    return target if target.is_a?(IO)
    spelled = target.to_io
    unless spelled.is_a?(IO)
      raise TypeError, "can't convert #{target.class} to IO (#{target.class}#to_io gives #{spelled.class})"
    end
    spelled
  end
  private :__reopen_target__

  # The mode a reopen falls back on, which is the one this stream carries with
  # everything but the encodings it named taken off.
  def __reopen_mode__
    held = @__file_mode
    return "r" if held.nil? || !held.is_a?(String)
    written = held.split(":", 2)[0]
    written.empty? ? "r" : written
  end
  private :__reopen_mode__

  # How long one turn of a wait lasts. A wait is taken in slices this long
  # so the program keeps running while one of its threads waits.
  WAIT_SLICE_MS = 20

  # How long a read or a write on this stream may take before it gives up.
  # Nothing gives up by default, which is what nil means.
  def timeout
    @__timeout__
  end

  def timeout=(seconds)
    @__timeout__ = seconds
  end

  def read_timeout
    @__read_timeout__
  end

  def read_timeout=(seconds)
    @__read_timeout__ = seconds
  end

  def write_timeout
    @__write_timeout__
  end

  def write_timeout=(seconds)
    @__write_timeout__ = seconds
  end

  # Wait until there is something to read, or until the wait runs out. A
  # timeout of nil waits for as long as it takes.
  def wait_readable(timeout = nil)
    __wait_ready__("read", timeout)
  end

  def wait_writable(timeout = nil)
    __wait_ready__("write", timeout)
  end

  def wait(timeout = nil, mode = :read)
    __wait_ready__(mode.to_s.include?("write") ? "write" : "read", timeout)
  end

  def __wait_ready__(mode, timeout)
    raise IOError, "closed stream" if closed?
    waited = timeout.nil? ? -1 : (timeout.to_f * 1000).to_i
    # A wait longer than the counter holds is the same as waiting forever.
    waited = -1 if waited > 2147483647 || waited < -1
    handle = __stream_handle__
    return IO.__stream__("wait", handle, mode, 0) ? self : nil if waited == 0
    # The wait is taken in slices so whatever else the program has to run
    # gets a turn, and so a thread waiting here can be woken or stopped.
    left = waited
    while left != 0
      slice = left < 0 || left > WAIT_SLICE_MS ? WAIT_SLICE_MS : left
      return self if IO.__stream__("wait", handle, mode, slice)
      left -= slice if left > 0
      sleep 0.001
    end
    nil
  end
  private :__wait_ready__


  def to_io
    self
  end

  # The numbers the operating system keeps about this stream, read through
  # the descriptor rather than through a name, since a stream is not always
  # open on a file that has one.
  def stat
    raise IOError, "closed stream" if closed?
    File::Stat.new(@handle.nil? ? IO::NULL : "/dev/fd/#{fileno}")
  end

  # A copy holds a descriptor of its own over the same file, so closing
  # either one leaves the other open. The copy is never handed to a child
  # process, which is what Ruby sets on it.
  def dup
    raise IOError, "closed stream" if closed?
    copied = self.class.allocate
    copied.__send__ :__take__, IO.__stream__("dup", __stream_handle__, "", 0), path
    # A copy of a stream opened by name is opened on the same name, which is
    # what the methods written for a file read.
    ["@__file_path", "@__file_mode", "@__file_encoding", "@binmode"].each do |named|
      held = instance_variable_get named
      copied.instance_variable_set named, held unless held.nil?
    end
    copied.close_on_exec = true
    copied
  end

  def inspect
    return "#<IO: (closed)>" if closed?
    "#<IO:fd #{fileno}>"
  end
end

# A file opened by name answers the descriptor questions an IO answers, over
# a descriptor opened the first time one of them is asked."##;
