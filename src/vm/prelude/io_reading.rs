pub(super) const SOURCE: &str = r##"
  def close_read
    return nil if @read_closed
    unless __duplex__
      # A stream that was never writable has only the one side, so closing
      # the reading side closes the stream.
      return close if !__both_ways__ && __opened_for__("r")
      raise IOError, "closing non-duplex IO for reading"
    end
    @read_closed = true
    nil
  end

  def close_write
    return nil if @write_closed
    unless @__popen_writer.nil?
      @__popen_writer.close
      @write_closed = true
      return nil
    end
    unless __duplex__
      return close if !__both_ways__ && (__opened_for__("w") || __opened_for__("a"))
      raise IOError, "closing non-duplex IO for writing"
    end
    @write_closed = true
    nil
  end

  def write(*parts)
    texts = parts.map { |part| part.to_s }
    # Nothing to write is written without asking whether the stream can be
    # written at all.
    return 0 if texts.all? { |text| text.empty? }
    __write_texts__ texts, true
  end

  # The pieces written to the stream, carried into its encoding when
  # `converting` says so. `syswrite` and `write_nonblock` write them as they
  # are.
  # How a write waits on a descriptor with no room: WRITE_ONCE tries once,
  # WRITE_WHEN_ROOM waits and writes what fits, and WRITE_ALL waits until
  # everything is written.
  WRITE_ONCE = 0
  WRITE_WHEN_ROOM = 2
  WRITE_ALL = 1
  private_constant :WRITE_ONCE, :WRITE_WHEN_ROOM, :WRITE_ALL

  def __write_texts__(texts, converting, waiting = WRITE_ALL)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for writing" if @write_closed
    raise IOError, "not opened for writing" unless __writable__
    return @__popen_writer.__send__(:__write_texts__, texts, converting, waiting) unless @__popen_writer.nil?
    @line_buffered = nil
    @wrote_through_buffer = true
    # Each piece is carried into the stream's encoding on its own, and the
    # pieces are joined as the bytes they were written in.
    held = if texts.length == 1
      converting ? __written_text__(texts[0]) : texts[0]
    else
      texts.map { |text| (converting ? __written_text__(text) : text).b }.join
    end
    # A stream the program told not to sync holds what is written until it is
    # flushed, which is when the descriptor hears about it.
    if @holding && @standard.nil?
      @pending = @pending.nil? ? held : @pending + held
      return held.bytesize
    end
    # The streams the program started with are written through the
    # interpreter's own writer, so what a program prints keeps the order it
    # printed it in whichever route it took.
    return IO.__stream__("write", __stream_handle__, held, waiting) if @standard.nil?
    IO.__write_standard__ @standard, held
    held.bytesize
  end

  private :__write_texts__

  # Text carried into the encoding the stream was told to write in. A stream
  # told none, or told to write bytes, writes the text as it stands.
  def __written_text__(text)
    named = __named_encodings__[0]
    return text if named.nil? || named.empty?
    target = Encoding.find named
    return text if target == Encoding::BINARY || text.encoding == target
    return text.encode(target) if target.ascii_compatible?
    # An encoding that spells ASCII another way is reached through UTF-8.
    text.encode(Encoding::UTF_8).encode target
  end
  private :__written_text__

  # Everything held back by a stream that does not sync, written through now.
  def __drain__
    return nil if @pending.nil? || @pending.empty?
    # What could not be written stays held, so the next flush or the close
    # reports the same trouble rather than losing it.
    IO.__stream__ "write", __stream_handle__, @pending, WRITE_ALL
    @pending = nil unless frozen?
    nil
  end
  private :__drain__

  def <<(text)
    write text
    self
  end

  # Each value written out as text, separated by `$,` where the program set
  # one, and followed by `$\`. With nothing to write the last line read is
  # written instead.
  def print(*parts)
    parts = [$_] if parts.empty?
    separator = $,
    parts.each_with_index do |part, index|
      write separator.to_s if index > 0 && !separator.nil?
      write(part.nil? ? "" : part.to_s)
    end
    write $\ unless $\.nil?
    nil
  end

  def printf(format, *rest)
    # A format may be written as anything that spells itself out.
    spelled = format.is_a?(String) ? format : format.to_str
    write spelled % rest
    nil
  end

  def puts(*lines)
    return write(__line_ending__) && nil if lines.empty?
    __write_lines__ lines, []
    nil
  end

  # What ends a line this stream writes. A stream opened with `newline:` ends
  # them the way that named, and every other stream ends them with a newline.
  def __line_ending__
    return "\r\n" if @__file_newline == :crlf
    return "\r" if @__file_newline == :cr
    "\n"
  end
  private :__line_ending__

  # One line per value, where an array is written out element by element. An
  # array that reaches itself is written as `[...]` rather than followed.
  def __write_lines__(values, walking)
    ending = __line_ending__
    values.each do |value|
      spread = value.is_a?(Array) ? value : __as_array__(value)
      if spread.nil?
        held = value.nil? ? "" : value.to_s
        # A `to_s` that hands back something other than a String says
        # nothing about the object, so the object describes itself.
        held = Object.instance_method(:to_s).bind(value).call unless held.is_a? String
        write(held.end_with?(ending) ? held : held + ending)
      elsif walking.any? { |seen| seen.equal? value }
        write "[...]#{ending}"
      else
        __write_lines__ spread, walking + [value]
      end
    end
  end
  private :__write_lines__

  # The Array an object stands for, or nil where it stands for none. An
  # object that answers for missing names is asked too, and one that refuses
  # the name stands for no Array.
  def __as_array__(value)
    return nil if value.nil? || value.is_a?(String)
    held = begin
      value.to_ary
    rescue NoMethodError
      nil
    end
    held.is_a?(Array) ? held : nil
  end
  private :__as_array__

  # As many bytes as asked for, or everything left where no count is given.
  # Whatever was put back with `ungetc` stands before what the descriptor
  # has, and is handed out first.
  def read(length = nil, buffer = nil)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" if @read_closed
    wanted = length.nil? ? 0 : __as_integer__(length)
    raise ArgumentError, "negative length #{wanted} given" if wanted < 0
    __take_bom__
    target = buffer.nil? ? nil : __as_buffer__(buffer)
    # A count of nothing reads nothing and leaves the stream where it stands.
    if !length.nil? && wanted == 0
      empty = "".dup.force_encoding(Encoding::BINARY)
      return __fill_buffer__(target, empty) unless target.nil?
      return empty
    end
    waiting = @peeked
    @peeked = nil
    waiting = "" if waiting.nil?
    held = if length.nil?
      waiting + __read_to_the_end__
    elsif waiting.bytesize > wanted
      taken = waiting[0, wanted]
      rest = waiting[wanted, waiting.length - wanted]
      @peeked = rest.nil? || rest.empty? ? nil : rest
      taken
    elsif waiting.bytesize == wanted
      waiting
    else
      # What was put back is already in hand, so a descriptor with nothing
      # waiting behind it does not make the read fail.
      more = begin
        __stream_read__(wanted - waiting.bytesize).to_s
      rescue Errno::EAGAIN
        raise if waiting.empty?
        ""
      end
      waiting + more
    end
    # A read of the whole stream carries the text over the way the stream was
    # told to. A read of so many bytes hands those bytes back as they are.
    # A count of bytes hands those bytes back as they are, which Ruby tags
    # as a run of bytes rather than as text.
    held = length.nil? ? __tag_read__(held) : held.dup.force_encoding(Encoding::BINARY)
    unless target.nil?
      # A read of the whole stream carries its encoding into the buffer, and
      # a read of so many bytes leaves the buffer tagged as it was.
      __fill_buffer__ target, held, length.nil?
      return nil if !length.nil? && held.empty?
      return target
    end
    return nil if length && held.empty?
    held
  end

  # Up to `count` bytes read from the descriptor, or everything it has ready
  # for a count of 0. A fiber that is not blocking hands the wait for
  # something to read to its scheduler.
  def __stream_read__(count)
    scheduler = Fiber.current_scheduler
    unless scheduler.nil? || IO.__stream__("ready?", __stream_handle__, "", 0)
      scheduler.io_wait self, IO::READABLE, nil
    end
    IO.__stream__ "read", __stream_handle__, "", count
  end
  private :__stream_read__

  # Take a byte-order mark off the front of the stream, where the mode asked
  # for one and the stream opens with one.
  def __take_bom__
    return if @__bom_read
    @__bom_read = true
    return unless __asks_for_bom__
    head = __stream_read__(4).to_s
    found, width = IO.bom_encoding(head.bytes)
    if found.nil?
      @peeked = head.empty? ? nil : head
      return
    end
    @__file_encoding = found.name
    rest = head.byteslice(width, head.bytesize - width).to_s
    @peeked = rest.empty? ? nil : rest
  end
  private :__take_bom__

  # Whether the mode asked for the encoding a byte-order mark names.
  def __asks_for_bom__
    written = @__file_encoding.to_s
    written = @__file_mode.to_s.split(":", 2)[1].to_s if written.empty?
    first = written.split(":")[0].to_s
    first.length > 4 && first[0, 4].casecmp("BOM|").zero?
  end
  private :__asks_for_bom__

  # As much as is there right now, up to the count asked for. Nothing left
  # at all is the end of the stream.
  def readpartial(length = nil, buffer = nil)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    wanted = length.nil? ? 0 : __as_integer__(length)
    raise ArgumentError, "negative length #{wanted} given" if wanted < 0
    target = buffer.nil? ? nil : __as_buffer__(buffer)
    return __fill_buffer__(target, "") if wanted == 0
    unless @peeked.nil? || @peeked.empty?
      return __fill_buffer__ target, __take_ready__(wanted)
    end
    held = read wanted
    if held.nil? || held.empty?
      __fill_buffer__ target, ""
      raise EOFError, "end of file reached"
    end
    __fill_buffer__ target, held
  end

  # A character put back, which the next read hands out before anything the
  # descriptor has.
  def ungetc(held)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    raise TypeError, "no implicit conversion of nil into String" if held.nil?
    text = if held.is_a? Integer
      # A codepoint stands for the character the stream reads text as.
      named = external_encoding
      held.chr(named.nil? ? Encoding::UTF_8 : named)
    elsif held.is_a? String
      held
    elsif held.respond_to? :to_str
      held.to_str
    else
      raise TypeError, "no implicit conversion of #{held.class} into String"
    end
    @peeked = @peeked.nil? ? text : text + @peeked
    # A stream holding text the descriptor already handed over cannot be
    # read around, which is what `sysread` would do.
    @line_buffered = true
    nil
  end

  # As many bytes as asked for, read straight from the descriptor. A stream
  # whose lines have been read holds text the descriptor has already handed
  # over, so reading around that is refused.
  def sysread(length = nil, buffer = nil)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    raise IOError, "sysread for buffered IO" if @line_buffered
    wanted = length.nil? ? 0 : __as_integer__(length)
    raise ArgumentError, "negative length #{wanted} given" if wanted < 0
    target = buffer.nil? ? nil : __as_buffer__(buffer)
    return target.nil? ? "" : target if wanted == 0
    held = __stream_read__ wanted
    if held.nil? || held.empty?
      __fill_buffer__ target, ""
      raise EOFError, "end of file reached"
    end
    return __fill_buffer__(target, held) unless target.nil?
    held
  end

  # The String a program handed over to be read into.
  def __as_buffer__(buffer)
    return buffer if buffer.is_a? String
    unless buffer.respond_to? :to_str
      raise TypeError, "no implicit conversion of #{buffer.class} into String"
    end
    buffer.to_str
  end
  private :__as_buffer__

  # What was read, written into the buffer the program handed over. The
  # buffer keeps the encoding it was tagged with.
  def __fill_buffer__(target, held, carries = false)
    return held if target.nil?
    was = target.encoding
    target.replace held
    target.force_encoding(carries ? held.encoding : was)
    target
  end
  private :__fill_buffer__

  # Where the stream stands, counted in bytes from the start. A byte put back
  # with `ungetc` stands before that place.
  def pos
    raise IOError, "closed stream" if closed?
    __drain__
    standing = IO.__stream__ "seek", __stream_handle__, "cur", 0
    standing - (@peeked.nil? ? 0 : @peeked.bytesize)
  end

  alias_method :tell, :pos

  def pos=(offset)
    seek offset, IO::SEEK_SET
    offset
  end

  # An offset the operating system can hold. It counts bytes in a file, and
  # no file is longer than a machine word counts.
  def __as_offset__(offset)
    held = __as_integer__ offset
    if held.bit_length > 62
      raise RangeError, "bignum too big to convert into 'long'"
    end
    held
  end
  private :__as_offset__

  # The stream moved to another place: from the start, from where it stands,
  # or back from the end.
  def seek(offset, whence = IO::SEEK_SET)
    raise IOError, "closed stream" if closed?
    __drain__
    @peeked = nil
    @line_buffered = nil
    named = case whence
    when IO::SEEK_CUR, :CUR then "cur"
    when IO::SEEK_END, :END then "end"
    else "set"
    end
    IO.__stream__ "seek", __stream_handle__, named, __as_offset__(offset)
    0
  end

  # The descriptor moved straight, without the buffer a read fills. A stream
  # whose lines have been read holds text the descriptor already handed over,
  # so moving around that is refused.
  def sysseek(offset, whence = IO::SEEK_SET)
    raise IOError, "sysseek for buffered IO" if @line_buffered
    held = __as_offset__ offset
    seek held, whence
    IO.__stream__ "seek", __stream_handle__, "cur", 0
  end

  # Back to the start, with the line count starting over.
  def rewind
    seek 0, IO::SEEK_SET
    @lineno = 0
    0
  end

  # How many bytes the stream stands over.
  def size
    raise IOError, "closed stream" if closed?
    __drain__
    IO.__stream__ "size", __stream_handle__, "", 0
  end

  # The file cut down to the count of bytes, or filled out to it.
  def truncate(length)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for writing" unless __writable__
    __drain__
    IO.__stream__ "truncate", __stream_handle__, "", __as_integer__(length)
  end

  # As much as is there right now. A stream with nothing waiting says so
  # rather than holding the program up, either by raising or, when asked not
  # to, by answering what it would have waited for.
  def read_nonblock(length, buffer = nil, exception: true)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    wanted = __as_integer__ length
    raise ArgumentError, "negative length #{wanted} given" if wanted < 0
    target = buffer.nil? ? nil : __as_buffer__(buffer)
    return __fill_buffer__(target, "") if wanted == 0
    waiting = @peeked.nil? ? "" : @peeked
    unless waiting.empty?
      # A stream carrying text cannot be read a byte at a time around what
      # was put back, which is what Ruby refuses here.
      if @__newline_conversion
        raise IOError, "byte oriented read for character buffered IO"
      end
      return __fill_buffer__(target, __take_ready__(wanted))
    end
    unless IO.__stream__("ready?", __stream_handle__, "nonblock", 0)
      return :wait_readable unless exception
      raise IO::EAGAINWaitReadable, "read would block"
    end
    held = read wanted
    if held.nil? || held.empty?
      __fill_buffer__ target, ""
      return nil unless exception
      raise EOFError, "end of file reached"
    end
    __fill_buffer__ target, held
  end

  # The bytes already in hand, up to `wanted`, and as many more as the
  # descriptor holds right now, without waiting on it for the rest.
  def __take_ready__(wanted)
    taken = read([wanted, @peeked.bytesize].min)
    short = wanted - taken.bytesize
    if short > 0 && IO.__stream__("ready?", __stream_handle__, "", 0)
      taken += __stream_read__(short).to_s.b
    end
    taken
  end
  private :__take_ready__

  # Written straight to the descriptor. A stream that has written through its
  # own buffer says so, since the two writes may land out of order.
  def syswrite(text)
    if @wrote_through_buffer
      warn "warning: syswrite for buffered IO"
    end
    held = __write_texts__ [text.to_s], false, WRITE_WHEN_ROOM
    @wrote_through_buffer = nil
    held
  end

  # As much as the descriptor will take right now. A descriptor with no room
  # says so rather than holding the program up.
  def write_nonblock(text, exception: true)
    buffered_before = @wrote_through_buffer
    begin
      __write_texts__ [text.to_s], false, WRITE_ONCE
    rescue Errno::EAGAIN
      raise IO::EAGAINWaitWritable, "write would block" if exception
      :wait_writable
    ensure
      @wrote_through_buffer = buffered_before
    end
  end

  # The next line, up to the separator or the limit, whichever comes first.
  # The line read is what `$_` and `$.` report on.
  def gets(separator = $/, limit = nil, *extra, chomp: false)
    unless extra.empty?
      raise ArgumentError,
            "wrong number of arguments (given #{2 + extra.size}, expected 0..2)"
    end
    # `$_` names the line `gets` read. Walking the lines does not set it.
    $_ = __read_line__ separator, limit, chomp
  end

  # The next line, up to the separator or the limit, whichever comes first.
  # An empty separator reads a paragraph: the lines up to a blank one, with
  # the blank lines before it left behind.
  def __read_line__(separator, limit, chomp)
    raise IOError, "closed stream" if closed?
    raise IOError, "not opened for reading" unless __readable__
    separator, limit = StringIO.line_arguments separator, limit
    unless limit.nil?
      limit = __as_integer__ limit
      if limit.bit_length > 62
        raise RangeError, "bignum too big to convert into 'long'"
      end
      return "" if limit == 0
    end
    ending = separator.nil? ? nil : separator.to_s
    # An empty separator reads a paragraph, which ends at a blank line and
    # starts after the blank lines the paragraph before it ended with.
    paragraph = !ending.nil? && ending.empty?
    ending = "\n\n" if paragraph
    # A byte or a character put back with `ungetc` stands before whatever the
    # stream has left, and may already hold the separator.
    waiting = @peeked
    @peeked = nil
    collected = waiting.nil? ? "" : waiting
    unless collected.empty?
      if !ending.nil? && !ending.empty?
        at = collected.index ending
        unless at.nil?
          ends = at + ending.length
          rest = collected[ends, collected.length - ends]
          @peeked = rest.nil? || rest.empty? ? nil : rest
          collected = collected[0, ends]
          return __finish_line__ collected, ending, chomp
        end
      end
      if !limit.nil? && collected.bytesize >= limit
        rest = collected[limit, collected.length - limit]
        @peeked = rest.nil? || rest.empty? ? nil : rest
        return __finish_line__ collected[0, limit], ending, chomp
      end
    end
    wanted = limit.nil? ? -1 : limit - collected.bytesize
    held = IO.__stream__(
      "readline",
      __stream_handle__,
      ending.nil? ? "" : ending,
      wanted,
      (paragraph ? 2 : 0) + (paragraph && collected.empty? ? 1 : 0)
    )
    collected = collected + held.to_s
    return nil if collected.empty?
    @line_buffered = true
    return __finish_line__ collected, ending, chomp
  end
  private :__read_line__

  # A line read, counted and tagged the way the stream reads text, with the
  # separator taken off where the reader asked for that.
  def __finish_line__(collected, ending, chomp)
    @lineno = (@lineno.nil? ? 0 : @lineno) + 1
    $. = @lineno
    held = __tag_read__ collected
    if chomp && !ending.nil? && !ending.empty? && held.end_with?(ending)
      held = held[0, held.length - ending.length]
    end
    held
  end
  private :__finish_line__

  def readline(separator = $/, limit = nil, *extra, chomp: false)
    line = self.gets separator, limit, *extra, chomp: chomp
    raise EOFError, "end of file reached" if line.nil?
    line
  end

  # The encodings named alongside the mode, as `"r:UTF-8:ISO-8859-1"` or as
  # an `encoding:` keyword. The first is what the stream is read as and the
  # second what its text is carried into.
  # The encodings are read without `to_s` on nil, which a program may have
  # redefined.
  def __named_encodings__
    written = @__file_encoding.nil? ? "" : @__file_encoding.to_s
    if written.empty?
      # A stream told to follow the program's own encodings names none of
      # its own any more, whatever its mode was opened with.
      return [] if @__encodings_reset
      written = @__file_mode.nil? ? "" : (@__file_mode.to_s.split(":", 2)[1] || "")
    end
    held = written.split(":")
    # `BOM|utf-8` names the encoding to fall back on where the stream opens
    # with no mark of its own.
    first = held[0] || ""
    held[0] = first[4..-1] if first.length > 4 && first[0, 4].casecmp("BOM|").zero?
    held
  end
  private :__named_encodings__

  # The encodings the program was reading and writing text in when the stream
  # was opened. A stream keeps them, so a later change to the program's own
  # encodings leaves an already open stream alone.
  def __note_encodings__
    return self if @__noted_encodings
    @__noted_encodings = true
    @__made_external = Encoding.default_external
    @__made_internal = Encoding.default_internal
    self
  end

  # Whether the stream was opened to write, which decides what it reports
  # when no encoding was named for it.
  def __writing_mode__
    access = @__file_mode.to_s.split(":", 2)[0].to_s
    access.start_with?("w") || access.start_with?("a") || access.include?("+")
  end
  private :__writing_mode__

  # The encoding a leading byte-order mark names, with how many bytes it
  # takes. A mark that runs out part way names nothing.
  def self.bom_encoding(bytes)
    return [Encoding::UTF_8, 3] if bytes[0, 3] == [0xEF, 0xBB, 0xBF]
    if bytes[0, 2] == [0xFF, 0xFE]
      return [Encoding::UTF_32LE, 4] if bytes[2, 2] == [0x00, 0x00]
      return [Encoding::UTF_16LE, 2]
    end
    return [Encoding::UTF_16BE, 2] if bytes[0, 2] == [0xFE, 0xFF]
    return [Encoding::UTF_32BE, 4] if bytes[0, 4] == [0x00, 0x00, 0xFE, 0xFF]
    [nil, 0]
  end

  # Read the byte-order mark the stream starts with, if any, and take the
  # encoding it names. The mark is consumed; anything else is left in place.
  def set_encoding_by_bom
    named = __named_encodings__
    unless named.length < 2
      raise ArgumentError, "encoding conversion is set"
    end
    unless named.empty? || named[0].to_s.casecmp("ASCII-8BIT").zero? ||
           named[0].to_s.casecmp("BINARY").zero?
      raise ArgumentError, "encoding is set to #{Encoding.find(named[0])} already"
    end
    unless external_encoding == Encoding::BINARY
      raise ArgumentError, "ASCII incompatible encoding needs binmode"
    end
    return nil if __writing_mode__ && !@__file_mode.to_s.include?("+")
    start = pos
    head = read(4)
    self.pos = start
    found, width = IO.bom_encoding(head.nil? ? [] : head.bytes)
    return nil if found.nil?
    self.pos = start + width
    set_encoding found
    found
  end

  # The encoding a stream reads as. A stream that names none reads as the
  # program's external encoding, which a stream opened while an internal
  # encoding was set keeps as it was at the time.
  def external_encoding
    named = __named_encodings__[0]
    return Encoding.find(named) unless named.nil? || named.empty?
    unless @__encodings_reset
      return Encoding::BINARY if @binmode
      return Encoding::BINARY if @__file_mode.to_s.split(":", 2)[0].to_s.include?("b")
    end
    return nil if __writing_mode__ && @__made_internal.nil?
    return @__made_external unless @__made_internal.nil?
    Encoding.default_external
  end
"##;
