# Compressed streams: the two checksums zlib keeps, the DEFLATE encoding
# underneath, and the wrappers that carry it. The encodings themselves are
# computed by the interpreter, since every reader has to agree on them.

module Zlib
  ZLIB_VERSION = "1.2.13"
  VERSION = "0.6.1"

  BINARY = 0
  ASCII = 1
  TEXT = 1
  UNKNOWN = 2

  NO_COMPRESSION = 0
  BEST_SPEED = 1
  BEST_COMPRESSION = 9
  DEFAULT_COMPRESSION = -1

  FILTERED = 1
  HUFFMAN_ONLY = 2
  RLE = 3
  FIXED = 4
  DEFAULT_STRATEGY = 0

  NO_FLUSH = 0
  SYNC_FLUSH = 2
  FULL_FLUSH = 3
  FINISH = 4

  MAX_WBITS = 15
  DEF_MEM_LEVEL = 8
  MAX_MEM_LEVEL = 9

  OS_UNIX = 3
  OS_CODE = 3

  class Error < StandardError; end
  class StreamEnd < Error; end
  class NeedDict < Error; end
  class DataError < Error; end
  class StreamError < Error; end
  class MemError < Error; end
  class BufError < Error; end
  class VersionError < Error; end

  # A checksum reads a whole 32-bit number, so a value written as a negative
  # one names the same bits from the other end.
  def self.wrapped(value)
    unless value.is_a? Integer
      raise TypeError, "no implicit conversion of #{value.class} into Integer"
    end
    if value.abs > 0xffffffff
      raise RangeError, "bignum too big to convert into 'unsigned long'"
    end
    value & 0xffffffff
  end

  def self.crc32(text = "", running = 0)
    Zlib.__stream__ "crc32", Zlib.coerce_text(text), Zlib.wrapped(running)
  end

  def self.adler32(text = "", running = 1)
    Zlib.__stream__ "adler32", Zlib.coerce_text(text), Zlib.wrapped(running)
  end

  def self.crc_table
    Zlib.__stream__ "crc_table", "", 0
  end

  def self.zlib_version
    ZLIB_VERSION
  end

  def self.deflate(text, _level = DEFAULT_COMPRESSION)
    Zlib.__stream__ "deflate", Zlib.coerce_text(text), 0
  end

  def self.inflate(text)
    Zlib.__stream__ "inflate", Zlib.coerce_text(text), 0
  end

  def self.gzip(text, level: nil, strategy: nil)
    Zlib.__stream__ "gzip", Zlib.coerce_text(text), 0
  end

  def self.gunzip(text)
    Zlib.__stream__ "gunzip", Zlib.coerce_text(text), 0
  end

  # The text a stream was given, which has to be a String or say how to read
  # one out of itself.
  def self.coerce_text(text)
    return text if text.is_a? ::String
    return "" if text.nil?
    unless text.respond_to? :to_str
      raise TypeError, "no implicit conversion of #{text.class} into String"
    end
    text.to_str
  end

  # What every compressed stream keeps: what it has been given, what it has
  # produced, and whether it is still open.
  class ZStream
    def initialize
      @input = ""
      @output = ""
      @closed = false
      @finished = false
    end

    def avail_in
      0
    end

    def avail_out
      0
    end

    def avail_out=(size)
      size
    end

    def total_in
      @input.length
    end

    def total_out
      @output.length
    end

    def data_type
      Zlib::UNKNOWN
    end

    def adler
      Zlib.adler32 @input
    end

    def closed?
      @closed
    end

    alias_method :ended?, :closed?

    def finished?
      @finished
    end

    alias_method :stream_end?, :finished?

    def close
      @closed = true
      nil
    end

    alias_method :end, :close

    def reset
      @input = ""
      @output = ""
      @finished = false
      nil
    end

    def flush_next_in
      held = @input
      @input = ""
      held
    end

    def flush_next_out
      held = @output
      @output = ""
      held
    end
  end

  # Reads a compressed stream back into what it stands for.
  class Inflate < ZStream
    def self.inflate(text)
      Zlib.inflate text
    end

    def initialize(window_bits = MAX_WBITS)
      @window_bits = window_bits
      super()
    end

    def inflate(text)
      return "" if text.nil?
      @input = @input + Zlib.coerce_text(text)
      answer = read_all
      @output = answer
      @finished = true
      answer
    end

    def <<(text)
      return self if text.nil?
      @input = @input + Zlib.coerce_text(text)
      @output = read_all
      @finished = true
      self
    end

    def finish
      answer = @finished ? @output : read_all
      @output = answer
      @finished = true
      answer
    end

    def set_dictionary(text)
      text
    end

    private

    # A raw stream names no header, which is what a negative window size asks
    # for. Anything else carries the zlib header and checksum.
    def read_all
      return "" if @input.empty?
      action = @window_bits.to_i < 0 ? "raw_inflate" : "inflate"
      Zlib.__stream__ action, @input, 0
    end
  end

  # Writes what it is given as a compressed stream.
  class Deflate < ZStream
    def self.deflate(text, level = DEFAULT_COMPRESSION)
      Zlib.deflate text, level
    end

    def initialize(level = DEFAULT_COMPRESSION, window_bits = MAX_WBITS,
                   mem_level = DEF_MEM_LEVEL, strategy = DEFAULT_STRATEGY)
      @level = level
      @window_bits = window_bits
      super()
    end

    def deflate(text, flush = NO_FLUSH)
      @input = @input + Zlib.coerce_text(text) unless text.nil?
      return "" unless flush == FINISH
      finish
    end

    def <<(text)
      @input = @input + Zlib.coerce_text(text) unless text.nil?
      self
    end

    def finish
      answer = Zlib.__stream__ "deflate", @input, 0
      @output = answer
      @finished = true
      answer
    end

    def flush(_kind = SYNC_FLUSH)
      ""
    end

    def params(level, strategy)
      @level = level
      nil
    end

    def set_dictionary(text)
      text
    end
  end
  # A gzip member, which carries a name and a timestamp alongside what it
  # holds. Reading and writing are two classes over the same header.
  class GzipFile
    class Error < Zlib::Error; end
    class NoFooter < Error; end
    class CRCError < Error; end
    class LengthError < Error; end

    attr_reader :level

    def mtime
      refuse_when_closed
      @mtime
    end

    def mtime=(held)
      refuse_when_closed
      @mtime = held.is_a?(Integer) ? Time.at(held) : held
    end

    def orig_name
      refuse_when_closed
      @orig_name
    end

    def orig_name=(held)
      refuse_when_closed
      @orig_name = held
    end

    def comment
      refuse_when_closed
      @comment
    end

    def comment=(held)
      refuse_when_closed
      @comment = held
    end

    # A member that has been closed has nothing left to say about itself.
    def refuse_when_closed
      raise Zlib::GzipFile::Error, "closed gzip stream" if @closed
      nil
    end

    private :refuse_when_closed

    def initialize
      @closed = false
      @mtime = Time.at 0
      @orig_name = nil
      @comment = nil
      @level = Zlib::DEFAULT_COMPRESSION
      @sync = false
    end

    def self.wrap(io, *rest)
      held = new io, *rest
      return held unless block_given?
      begin
        yield held
      ensure
        held.close unless held.closed?
      end
    end

    def closed?
      @closed
    end

    def sync
      @sync
    end

    def sync=(held)
      @sync = held
    end

    def to_io
      @io
    end

    def crc
      Zlib.crc32 @body.to_s
    end

    def os_code
      Zlib::OS_CODE
    end

    def finish
      close
    end
  end

  # Reads what a gzip member stands for.
  class GzipReader < GzipFile
    include Enumerable

    def self.open(name)
      reader = new File.open(name, "rb")
      return reader unless block_given?
      begin
        yield reader
      ensure
        reader.close
      end
    end

    def self.zcat(io)
      new(io).read
    end

    def initialize(io, **_options)
      super()
      @io = io
      # A stream is asked for everything it holds, named rather than left
      # out, since a reader may take the count as a required argument.
      held = io.read nil
      @body = Zlib.__stream__ "gunzip", held.to_s, 0
      @at = 0
      @line = 0
      @behind = 0
      read_header held.to_s
    end

    def read(length = nil)
      return "" if length == 0
      if length.nil?
        held = @body[@at..-1]
        @at = @body.length
        return held.nil? ? "" : held
      end
      raise ArgumentError, "negative length #{length} given" if length < 0
      return nil if @at >= @body.length
      held = @body[@at, length]
      @at = @at + held.length
      held
    end

    def readpartial(length, _buffer = nil)
      raise ArgumentError, "negative length #{length} given" if length < 0
      raise EOFError, "end of file reached" if @at >= @body.length
      read length
    end

    def getc
      return nil if @at >= @body.length
      held = @body[@at]
      @at = @at + 1
      held
    end

    def getbyte
      held = getc
      held.nil? ? nil : held.ord
    end

    alias_method :readchar, :getc

    def ungetc(held)
      text = held.is_a?(Integer) ? held.chr : held.to_s
      @body = @body[0, @at].to_s + text + @body[@at..-1].to_s
      @behind = @behind + text.length
      nil
    end

    def ungetbyte(held)
      ungetc held
    end

    def gets(separator = $/)
      return nil if @at >= @body.length
      return read_paragraph if separator == ""
      place = @body.index separator, @at
      held = place.nil? ? @body[@at..-1] : @body[@at, place - @at + separator.length]
      @at = @at + held.length
      @line = @line + 1
      $_ = held
      held
    end

    # A paragraph runs to the blank line that ends it, and the newlines that
    # separate it from the next one come with it.
    def read_paragraph
      @at = @at + 1 while @body[@at] == "\n"
      return nil if @at >= @body.length
      place = @body.index "\n\n", @at
      if place.nil?
        held = @body[@at..-1]
        @at = @body.length
      else
        finish = place + 2
        held = @body[@at, finish - @at]
        @at = finish
        # The blank lines past the first are the next paragraph's business,
        # so they are stepped over rather than handed back.
        @at = @at + 1 while @body[@at] == "\n"
      end
      @line = @line + 1
      held
    end

    def readline(separator = $/)
      held = self.gets separator
      raise EOFError, "end of file reached" if held.nil?
      held
    end

    def readlines(separator = $/)
      held = []
      while (line = self.gets(separator))
        held.push line
      end
      held
    end

    def each(separator = $/)
      return to_enum(:each, separator) unless block_given?
      while (line = self.gets(separator))
        yield line
      end
      self
    end

    alias_method :each_line, :each

    def each_byte
      return to_enum(:each_byte) unless block_given?
      while (held = getbyte)
        yield held
      end
      nil
    end

    def each_char
      return to_enum(:each_char) unless block_given?
      while (held = getc)
        yield held
      end
      nil
    end

    def lineno
      @line
    end

    def lineno=(held)
      @line = held
    end

    def eof
      @at >= @body.length
    end

    alias_method :eof?, :eof

    def pos
      @at - @behind
    end

    alias_method :tell, :pos

    def rewind
      @at = 0
      @line = 0
      @behind = 0
      if @io.respond_to? :seek
        @io.seek 0
      elsif @io.respond_to? :rewind
        @io.rewind
      end
      0
    rescue ArgumentError
      0
    end

    def unused
      nil
    end

    def close
      @closed = true
      @io
    end

    def finish
      close
    end

    private

    # The name and time a member was written with, which sit in its header.
    def read_header(held)
      return if held.length < 10
      stamp = held[4, 4].each_char.map { |byte| byte.ord }
      seconds = stamp[0] + (stamp[1] << 8) + (stamp[2] << 16) + (stamp[3] << 24)
      @mtime = Time.at seconds
      flags = held[3].ord
      return if flags & 0x08 == 0
      finish_at = held.index "\0", 10
      @orig_name = finish_at.nil? ? nil : held[10, finish_at - 10]
    end
  end

  # Writes what it is given as a gzip member.
  class GzipWriter < GzipFile
    def self.open(name, level = nil, strategy = nil)
      writer = new File.open(name, "wb"), level, strategy
      return writer unless block_given?
      begin
        yield writer
      ensure
        writer.close
      end
    end

    def initialize(io, level = nil, _strategy = nil, **_options)
      super()
      @io = io
      @body = ""
      @header_written = false
      @level = level.nil? ? Zlib::DEFAULT_COMPRESSION : level
    end

    def write(*pieces)
      written = 0
      pieces.each do |piece|
        held = piece.to_s
        @body = @body + held
        written = written + held.length
      end
      @header_written = true
      written
    end

    def mtime=(held)
      refuse_written_header
      super
    end

    def orig_name=(held)
      refuse_written_header
      super
    end

    def comment=(held)
      refuse_written_header
      super
    end

    # The header goes out in front of the first piece written, so nothing in
    # it can be set after that.
    def refuse_written_header
      raise Zlib::GzipFile::Error, "header is already written" if @header_written
      nil
    end

    private :refuse_written_header

    def <<(piece)
      write piece
      self
    end

    def print(*pieces)
      pieces.each { |piece| write piece }
      nil
    end

    def printf(format, *pieces)
      write format % pieces
      nil
    end

    def puts(*pieces)
      return write("\n") if pieces.empty?
      pieces.each do |piece|
        held = piece.to_s
        write(held.end_with?("\n") ? held : held + "\n")
      end
      nil
    end

    def putc(held)
      write(held.is_a?(Integer) ? held.chr : held.to_s[0])
      held
    end

    def flush(_kind = nil)
      self
    end

    def pos
      @body.length
    end

    alias_method :tell, :pos

    def close
      return @io if @closed
      @closed = true
      held_time = @mtime
      held_name = @orig_name
      stamp = held_time.nil? ? 0 : held_time.to_i
      @io.write Zlib.__stream__("gzip", @body, stamp, held_name)
      @io.close if @io.respond_to?(:close) && !@io.is_a?(StringIO)
      @io
    end

    def finish
      close
    end
  end
end
