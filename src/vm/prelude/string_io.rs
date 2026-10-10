pub(super) const SOURCE: &str = r##"
class StringIO
  include Enumerable

  VERSION = "3.1.2"

  def initialize(*given, **options)
    if given.length > 2
      raise ArgumentError,
            "wrong number of arguments (given #{given.length}, expected 0..2)"
    end
    held = if given.empty?
             "".dup.force_encoding(Encoding.default_external)
           else
             StringIO.__backend_string__(given[0])
           end
    asked = StringIO.__named_mode__(given[1], options)
    @binary = options[:binmode] == true
    read_mode(asked)
    if held.frozen?
      # A frozen buffer can only be read: a mode asking to write it is
      # refused, and one asking to empty it says so as a frozen string would.
      raise Errno::EACCES if @writable && !asked.nil?
      if @truncates
        raise FrozenError, "can\'t modify frozen String: #{held.inspect}"
      end
      @writable = false
    end
    @string = held
    @position = 0
    @lineno = 0
    @closed_read = false
    @closed_write = false
    @ungotten = ""
    # Truncating empties the buffer itself without changing what it is
    # written in, so the string the caller handed over is emptied too.
    @string.replace("".dup.force_encoding(@string.encoding)) if @truncates
    self
  end
  private :initialize

  # `StringIO.open` leaves the stream with nothing behind it once the block
  # is over, which is what `string` then answers.
  def __release_string__
    @string = nil
    nil
  end
  private :__release_string__

  # The buffer a stream is opened over, which anything answering `to_str`
  # names.
  def self.__backend_string__(held)
    return held if held.is_a? String
    unless held.respond_to? :to_str
      raise TypeError, "no implicit conversion of #{held.class} into String"
    end
    held.to_str
  end

  # The mode a stream was asked for, written either as the second argument or
  # as the `mode:` option. Naming it both ways at once is refused.
  def self.__named_mode__(mode, options)
    named = options[:mode]
    if !mode.nil? && !named.nil?
      raise ArgumentError, "mode specified twice"
    end
    held = mode.nil? ? named : mode
    written = held.is_a?(String) ? held.split(":").first.to_s : ""
    named_binary = options.key?(:binmode) || options.key?(:textmode)
    if named_binary && (written.include?("b") || written.include?("t"))
      raise ArgumentError, "binmode specified twice"
    end
    if options[:binmode] == true && options[:textmode] == true
      raise ArgumentError, "both textmode and binmode specified"
    end
    unless options[:encoding].nil? && options[:external_encoding].nil? &&
           options[:internal_encoding].nil?
      if held.is_a?(String) && held.include?(":")
        raise ArgumentError, "encoding specified twice"
      end
    end
    held
  end

  # What a mode says about which sides of the stream are open. A missing mode
  # leaves both open.
  def read_mode(mode)
    @readable = true
    @writable = true
    @appends = false
    @truncates = false
    return if mode.nil?
    return read_numbered_mode(mode) if mode.is_a? Integer
    unless mode.is_a? String
      unless mode.respond_to? :to_str
        raise ArgumentError, "invalid access mode #{mode}"
      end
      mode = mode.to_str
    end
    spelling = mode.split(":").first.to_s.gsub("b", "").gsub("t", "")
    case spelling
    when "r"
      @writable = false
    when "r+"
      nil
    when "w"
      @readable = false
      @truncates = true
    when "w+"
      @truncates = true
    when "a"
      @readable = false
      @appends = true
    when "a+"
      @appends = true
    else
      raise ArgumentError, "invalid access mode #{mode}"
    end
  end
  private :read_mode

  # The same, for a mode written as the flags `File` names.
  def read_numbered_mode(number)
    access = number & 3
    @readable = access != File::WRONLY
    @writable = access != File::RDONLY
    @appends = (number & File::APPEND) != 0
    @truncates = (number & File::TRUNC) != 0
  end
  private :read_numbered_mode

  def self.new(*given, **options)
    if block_given?
      warn "warning: StringIO::new() does not take block; use StringIO::open() instead"
    end
    made = allocate
    made.send(:initialize, *given, **options)
    made
  end

  def self.open(*given, **options)
    held = new(*given, **options)
    return held unless block_given?
    begin
      yield held
    ensure
      held.close
      # The block leaves the stream with nothing behind it, which is what
      # tells a stream that was opened for a block from one that was not.
      held.send(:__release_string__)
    end
  end

  # ── What the stream holds ────────────────────────────────────────────────

  def string
    @string
  end

  def string=(text)
    @string = text.is_a?(String) ? text : text.to_str
    @position = 0
    @lineno = 0
    text
  end

  def size
    @string.length
  end

  def length
    @string.length
  end

  # The cursor is reported in bytes, so a character made of several bytes
  # moves it by that many.
  def pos
    return @position if single_byte_characters?
    beyond = @position - @string.length
    beyond = 0 if beyond < 0
    @string[0, @position].bytesize + beyond
  end

  def tell
    pos
  end

  def pos=(offset)
    raise Errno::EINVAL if offset < 0
    if single_byte_characters?
      @position = offset
      @misaligned = false
      return offset
    end
    @position = characters_before offset
    # An offset that lands inside a character leaves the cursor between
    # two of them, which only a reader of code points minds.
    @misaligned = @string[0, @position].bytesize != offset
    offset
  end

  # The number of characters standing before a byte offset.
  def characters_before(counted)
    at = 0
    seen = 0
    while seen < counted && at < @string.length
      seen += @string[at].bytesize
      at += 1
    end
    # An offset past the end counts what lies beyond it, and one that lands
    # inside a character stops at the character it landed in.
    return at + (counted - seen) if seen < counted
    at
  end
  private :characters_before

  # Whether each character of the buffer is one byte, so a byte offset is a
  # character position.
  def single_byte_characters?
    @string.bytesize == @string.length
  end
  private :single_byte_characters?

  def lineno
    @lineno
  end

  def lineno=(count)
    @lineno = count
  end

  def rewind
    @position = 0
    @lineno = 0
    0
  end

  def seek(amount, whence = 0)
    raise IOError, "closed stream" if closed?
    amount = amount.to_int if !amount.is_a?(Integer) && amount.respond_to?(:to_int)
    unless amount.is_a?(Integer)
      raise TypeError, "no implicit conversion of #{amount.class} into Integer"
    end
    base = if whence == 0
      0
    elsif whence == 1
      pos
    elsif whence == 2
      @string.bytesize
    else
      raise Errno::EINVAL
    end
    landing = base + amount
    raise Errno::EINVAL if landing < 0
    self.pos = landing
    0
  end

  def eof?
    @position >= @string.length
  end

  def eof
    eof?
  end

  def truncate(length)
    writing_allowed
    unless length.is_a?(Integer) || length.respond_to?(:to_int)
      raise TypeError, "no implicit conversion of #{length.class} into Integer"
    end
    wanted = length.is_a?(Integer) ? length : length.to_int
    raise Errno::EINVAL, "negative length" if wanted < 0
    # The buffer itself is cut or padded, so whoever handed it over sees the
    # change.
    if wanted <= @string.length
      @string.replace @string[0, wanted]
    else
      @string.replace(@string + "\0" * (wanted - @string.length))
    end
    0
  end

  def reopen(*given)
    other = given[0]
    mode = given[1]
    # One argument names another stream rather than a buffer, which is what
    # `to_strio` answers for an object standing in for one.
    if given.length < 2
      return __reopen_buffer__(other, nil) if other.is_a? String
      return __reopen_stream__(other) if given.length == 1
      return __reopen_buffer__("".dup, nil)
    end
    __reopen_buffer__(StringIO.__backend_string__(other), mode)
  end

  # Reopening over another stream, which takes its buffer whole.
  def __reopen_stream__(other)
    unless other.is_a? StringIO
      unless other.respond_to? :to_strio
        raise TypeError, "no implicit conversion of #{other.class} into StringIO"
      end
      other = other.to_strio
      unless other.is_a? StringIO
        raise TypeError, "can\'t convert to StringIO"
      end
    end
    __reopen_buffer__(other.string, nil)
  end
  private :__reopen_stream__

  # Reopening over a buffer, in the mode named alongside it.
  def __reopen_buffer__(held, mode)
    read_mode(mode)
    if held.frozen?
      raise Errno::EACCES if @writable && !mode.nil?
      if @truncates
        raise FrozenError, "can\'t modify frozen String: #{held.inspect}"
      end
      @writable = false
    end
    @string = held
    @position = 0
    @lineno = 0
    @closed_read = false
    @closed_write = false
    @ungotten = ""
    # Truncating empties the buffer itself without changing what it is
    # written in, so the string the caller handed over is emptied too.
    @string.replace("".dup.force_encoding(@string.encoding)) if @truncates
    self
  end
  private :__reopen_buffer__

  # ── Which sides are open ─────────────────────────────────────────────────

  def close
    @closed_read = true
    @closed_write = true
    nil
  end

  def close_read
    raise IOError, "closing non-duplex IO for reading" unless @readable
    @closed_read = true
    nil
  end

  def close_write
    raise IOError, "closing non-duplex IO for writing" unless @writable
    @closed_write = true
    nil
  end

  def closed?
    (!@readable || @closed_read) && (!@writable || @closed_write)
  end

  def closed_read?
    !@readable || @closed_read
  end

  def closed_write?
    !@writable || @closed_write
  end

  # Whether this stream may still be read, which every reading method asks
  # before it does anything.
  def reading_allowed
    raise IOError, "not opened for reading" unless @readable
    raise IOError, "not opened for reading" if @closed_read
  end
  private :reading_allowed

  # Whether this stream may still be written.
  def writing_allowed
    raise IOError, "not opened for writing" unless @writable
    raise IOError, "not opened for writing" if @closed_write
  end
  private :writing_allowed

  # ── Reading ──────────────────────────────────────────────────────────────

  # A buffer handed in is filled with what was read and answered in place of
  # a string of its own.
  def read(length = nil, buffer = nil)
    reading_allowed
    if length.nil?
      remaining = @string[@position..-1] || ""
      @position = @string.length
      return filled(buffer, remaining)
    end
    wanted = StringIO.whole_number length
    raise ArgumentError, "negative length #{wanted} given" if wanted < 0
    # Every character takes at least one byte, so the next `wanted`
    # characters hold the bytes this read can take.
    remaining = @string[@position, wanted] || ""
    if remaining.empty? && wanted > 0
      filled buffer, "" unless buffer.nil?
      return nil
    end
    # A count names bytes rather than characters, which is what a stream of
    # text holding multibyte characters reads out one piece at a time.
    taken = StringIO.__first_bytes__ remaining, wanted
    @position = @position + StringIO.__characters_for__(remaining, taken.bytesize)
    filled buffer, taken
  end

  # The first so many bytes of some text, as bytes rather than as text.
  def self.__first_bytes__(text, wanted)
    listed = text.bytes
    return text.b if listed.size <= wanted
    listed[0, wanted].pack("C*")
  end

  # How many characters the first so many bytes of some text spell. A count
  # landing inside a character counts that character as read.
  def self.__characters_for__(text, counted)
    used = 0
    walked = 0
    text.each_char do |held|
      break if used >= counted
      used += held.bytesize
      walked += 1
    end
    walked
  end

  # The number an argument stands for, refusing anything that names none.
  def self.whole_number(held)
    return held if held.is_a? Integer
    unless held.respond_to? :to_int
      raise TypeError, "no implicit conversion of #{held.class} into Integer"
    end
    held.to_int
  end

  # A buffer keeps the encoding it was tagged with, whatever the text put
  # into it was tagged with.
  def filled(buffer, text)
    return text if buffer.nil?
    unless buffer.is_a? String
      unless buffer.respond_to? :to_str
        raise TypeError, "no implicit conversion of #{buffer.class} into String"
      end
      buffer = buffer.to_str
    end
    kept = buffer.encoding
    buffer.replace text
    buffer.force_encoding kept
    buffer
  end
  private :filled

  def sysread(length = nil, buffer = nil)
    reading_allowed
    if eof? && !length.nil? && length > 0
      # The buffer holds what was read, so nothing read leaves it empty.
      buffer.replace "" unless buffer.nil?
      raise EOFError, "end of file reached"
    end
    self.read(length, buffer)
  end

  def readpartial(length = nil, buffer = nil)
    self.sysread(length, buffer)
  end

  def read_nonblock(length = nil, buffer = nil, exception: true)
    reading_allowed
    if eof? && !length.nil? && length > 0
      return nil unless exception
      raise EOFError, "end of file reached"
    end
    self.read(length, buffer)
  end

  def getc
    reading_allowed
    return nil if @position >= @string.length
    letter = @string[@position]
    @position = @position + 1
    letter
  end

  def readchar
    letter = self.getc
    raise EOFError, "end of file reached" if letter.nil?
    letter
  end

  def getbyte
    reading_allowed
    byte = @string.getbyte(@position)
    return nil if byte.nil?
    @position = @position + 1
    byte
  end

  def readbyte
    byte = self.getbyte
    raise EOFError, "end of file reached" if byte.nil?
    byte
  end

  def ungetc(letter)
    reading_allowed
    return nil if letter.nil?
    text = if letter.is_a? Integer
      letter.chr
    elsif letter.is_a? String
      letter
    elsif letter.respond_to? :to_str
      letter.to_str
    else
      raise TypeError, "no implicit conversion of #{letter.class} into String"
    end
    landing = @position - text.length
    landing = 0 if landing < 0
    # A position past the end leaves a gap, which Ruby fills with zero bytes
    # so the character lands where the position said.
    if landing > @string.length
      @string = @string + "\000" * (landing - @string.length)
    end
    @string = @string[0, landing] + text + (@string[landing + text.length..-1] || "")
    @position = landing
    nil
  end

  # Bytes put back land where the cursor stands, so a byte in the middle of a
  # character replaces that byte alone. What follows the cursor stays where
  # it was, which is why putting back more bytes than were read grows the
  # string.
  def ungetbyte(byte)
    return nil if byte.nil?
    listed = byte.is_a?(Integer) ? [byte & 0xff] : byte.to_s.bytes
    held = @string.bytes
    landing = @position - listed.size
    landing = 0 if landing < 0
    tail = held[@position..-1] || []
    named = @string.encoding
    @string = (held[0, landing] + listed + tail).pack("C*").force_encoding(named)
    @position = landing
    nil
  end

  def gets(separator = $/, limit = nil, chomp: false)
    separator, limit = StringIO.line_arguments separator, limit
    line = read_line separator, limit, chomp
    $_ = line
    line
  end

  # What a line reader's first two arguments stand for. A lone number in the
  # separator's place is a limit, and anything that reads as a String is a
  # separator.
  def self.line_arguments(separator, limit)
    if !separator.nil? && !separator.is_a?(String)
      if separator.respond_to? :to_str
        separator = separator.to_str
      else
        limit = separator
        separator = $/
      end
    end
    unless limit.nil?
      limit = whole_number limit
      # A negative limit is no limit at all.
      limit = nil if limit < 0
    end
    [separator, limit]
  end

  # One line, without touching `$_`, which is what every reader but `gets`
  # and `readline` does. The arguments arrive already read.
  def read_line(separator, limit, chomp)
    reading_allowed
    return "" if limit == 0
    remaining = @string[@position..-1] || ""
    return nil if remaining.empty?
    line = if separator.nil?
      remaining
    elsif separator == ""
      # A blank separator reads a paragraph: the newlines before it are
      # stepped over, and the run ends at the blank line that follows.
      skipped = 0
      skipped += 1 while skipped < remaining.length && remaining[skipped] == "\n"
      remaining = remaining[skipped..-1] || ""
      @position = @position + skipped
      return nil if remaining.empty?
      cut = remaining.index("\n\n")
      if cut.nil?
        remaining
      else
        # A paragraph keeps every blank line that closes it.
        ending = cut + 1
        ending += 1 while ending < remaining.length && remaining[ending] == "\n"
        remaining[0, ending]
      end
    else
      cut = remaining.index(separator)
      cut.nil? ? remaining : remaining[0, cut + separator.length]
    end
    line = line[0, limit] unless limit.nil?
    return nil if line.empty?
    @position = @position + line.length
    @lineno = @lineno + 1
    if chomp
      return separator == "\n" || separator.nil? ? line.chomp : line.chomp(separator)
    end
    line
  end
  private :read_line

  def readline(separator = $/, limit = nil, chomp: false)
    line = self.gets(separator, limit, chomp: chomp)
    raise EOFError, "end of file reached" if line.nil?
    line
  end

  def each_line(separator = $/, limit = nil, chomp: false)
    reading_allowed
    separator, limit = StringIO.line_arguments separator, limit
    return each_line_enumerator(separator, limit, chomp) unless block_given?
    while (line = read_line(separator, limit, chomp))
      # A limit of zero reads nothing, so there is no next line to reach.
      break if line.empty?
      yield line
    end
    self
  end

  # Without a block the lines are handed over one at a time, already read the
  # way the arguments ask for.
  def each_line_enumerator(separator, limit, chomp)
    collected = []
    while (line = read_line(separator, limit, chomp))
      # A limit of zero reads nothing, so there is no next line to reach.
      break if line.empty?
      collected.push line
    end
    collected.each
  end
  private :each_line_enumerator

  def each(separator = $/, limit = nil, chomp: false, &block)
    each_line(separator, limit, chomp: chomp, &block)
  end

  def readlines(separator = $/, limit = nil, chomp: false)
    reading_allowed
    separator, limit = StringIO.line_arguments separator, limit
    raise ArgumentError, "invalid limit: 0 for readlines" if limit == 0
    collected = []
    while (line = read_line(separator, limit, chomp))
      # A limit of zero reads nothing, so there is no next line to reach.
      break if line.empty?
      collected.push(line)
    end
    collected
  end

  def each_byte
    reading_allowed
    return sized_enum(:each_byte) unless block_given?
    while (byte = self.getbyte)
      yield byte
    end
    self
  end

  def each_char
    reading_allowed
    return sized_enum(:each_char) unless block_given?
    while (letter = self.getc)
      yield letter
    end
    self
  end

  def each_codepoint
    reading_allowed
    if @misaligned
      raise ArgumentError, "invalid byte sequence in #{external_encoding.name}"
    end
    return sized_enum(:each_codepoint) unless block_given?
    while (letter = self.getc)
      yield letter.ord
    end
    self
  end

  # ── Writing ──────────────────────────────────────────────────────────────

  # Text written to a stream tagged with an encoding of its own is carried
  # into that encoding first. Bytes stay as they are, since there is nothing
  # to read them as.
  def __for_writing__(text)
    named = @encoding
    return text if named.nil?
    return text if text.encoding.name == "ASCII-8BIT"
    return text if named.to_s == text.encoding.name
    begin
      text.encode named
    rescue StandardError
      text
    end
  end

  def write(*values)
    writing_allowed
    written = 0
    values.each do |value|
      text = __for_writing__ value.to_s
      @position = @string.length if @appends
      landing = @position
      if landing > @string.length
        @string = @string + "\0" * (landing - @string.length)
      end
      @string = @string[0, landing] + text + (@string[landing + text.length..-1] || "")
      @position = landing + text.length
      written = written + text.length
    end
    written
  end

  def syswrite(value)
    self.write(value)
  end

  def write_nonblock(value, exception: true)
    self.write(value)
  end

  def <<(value)
    self.write value
    self
  end

  def print(*values)
    writing_allowed
    values = [$_] if values.empty?
    values.each { |value| self.write(value.nil? ? "" : value.to_s) }
    self.write($\) unless $\.nil?
    nil
  end

  def printf(format, *values)
    self.write format % values
    nil
  end

  def putc(value)
    writing_allowed
    text = if value.is_a?(Integer)
      (value % 256).chr
    elsif value.is_a?(String)
      value[0, 1]
    elsif value.respond_to?(:to_int)
      (value.to_int % 256).chr
    else
      raise TypeError, "no implicit conversion of #{value.class} into Integer"
    end
    self.write text
    value
  end

  def puts(*values)
    writing_allowed
    write_lines(values, [])
    nil
  end

  # One line per value, where an array is written out element by element. An
  # array that reaches itself is written as `[...]` rather than followed.
  def write_lines(values, walking)
    if values.empty?
      self.write "\n"
      return
    end
    values.each do |value|
      spread = value.is_a?(Array) ? value : StringIO.__as_array__(value)
      unless spread.nil?
        if walking.any? { |held| held.equal?(value) }
          self.write "[...]\n"
          next
        end
        walking.push(value)
        spread.empty? ? self.write("\n") : write_lines(spread, walking)
        walking.pop
        next
      end
      text = value.nil? ? "" : value.to_s
      # A `to_s` that hands back something other than a String says nothing
      # about the object, so the object describes itself.
      text = Object.instance_method(:to_s).bind(value).call unless text.is_a? String
      self.write text
      self.write "\n" unless text.end_with? "\n"
    end
  end
  private :write_lines

  # The Array an object stands for, or nil where it stands for none. An
  # object that answers for missing names is asked too, and one that refuses
  # the name stands for no Array.
  def self.__as_array__(value)
    return nil if value.nil? || value.is_a?(String)
    held = begin
      value.to_ary
    rescue NoMethodError
      nil
    end
    held.is_a?(Array) ? held : nil
  end

  # ── What a stream reports about itself ───────────────────────────────────

  def fileno
    nil
  end

  def pid
    nil
  end

  def flush
    self
  end

  def fsync
    0
  end

  def sync
    true
  end

  def sync=(setting)
    setting
  end

  def isatty
    false
  end

  def tty?
    false
  end

  # Reading in binary tags what comes back as bytes rather than as text.
  def binmode
    @binary = true
    self
  end

  def external_encoding
    return Encoding::BINARY if @binary
    return @encoding unless @encoding.nil?
    @string.encoding
  end

  def internal_encoding
    nil
  end

  # The encoding a stream reads its text as. The buffer keeps whatever it was
  # tagged with, so a frozen string is left alone.
  def set_encoding(external, internal = nil)
    @encoding = external.is_a?(String) ? Encoding.find(external) : external
    @binary = false
    # The buffer is tagged along with the stream unless it refuses to change.
    @string.force_encoding @encoding unless @string.frozen?
    self
  end

  # Read the byte-order mark the stream starts with, if any, and take the
  # encoding it names. The mark is consumed; anything else is left in place.
  def set_encoding_by_bom
    raise FrozenError, "can't modify frozen StringIO: #{inspect}" if frozen?
    return nil unless @readable
    source = @string.bytes
    found, width = StringIO.bom_encoding(source[@position, 4] || [])
    return nil if found.nil?
    @position = @position + width
    @encoding = found
    @binary = false
    found
  end

  # The encoding a leading byte-order mark names, with how many bytes it
  # takes. A mark that runs out part way names nothing.
  def self.bom_encoding(bytes)
    IO.bom_encoding(bytes)
  end

  def fcntl(*args)
    raise NotImplementedError, "fcntl() function is unimplemented on this machine"
  end

  # A stream shows itself by class and address alone: what it holds is not
  # part of how it is written out.
  def inspect
    "#<StringIO:0x#{format("%016x", object_id * 2)}>"
  end

  def to_s
    inspect
  end

end
"##;
