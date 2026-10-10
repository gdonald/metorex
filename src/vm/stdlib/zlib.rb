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

  def crc32(text = "", running = 0)
    Zlib.__stream__ "crc32", Zlib.coerce_text(text), Zlib.wrapped(running)
  end

  def adler32(text = "", running = 1)
    Zlib.__stream__ "adler32", Zlib.coerce_text(text), Zlib.wrapped(running)
  end

  def crc_table
    Zlib.__stream__ "crc_table", "", 0
  end

  def zlib_version
    ZLIB_VERSION
  end

  def deflate(text, _level = DEFAULT_COMPRESSION)
    Zlib.__stream__ "deflate", Zlib.coerce_text(text), 0
  end

  def inflate(text)
    Zlib.__stream__ "inflate", Zlib.coerce_text(text), 0
  end

  def gzip(text, level: nil, strategy: nil)
    Zlib.__stream__ "gzip", Zlib.coerce_text(text), 0
  end

  def gunzip(text)
    Zlib.__stream__ "gunzip", Zlib.coerce_text(text), 0
  end
  # Ruby's zlib makes these module functions, which a module that includes
  # Zlib reaches as private instance methods.
  module_function :crc32, :adler32, :crc_table, :zlib_version, :deflate, :inflate, :gzip, :gunzip

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
    # The size of the pieces a block-taking read is handed, which is what
    # Ruby's inflater hands out.
    CHUNK_BYTES = 16384

    def self.inflate(text)
      Zlib.inflate text
    end

    def initialize(window_bits = MAX_WBITS)
      @window_bits = window_bits
      @done = false
      @pending = "".b
      super()
    end

    # What the stream has read so far. A block is handed the text a piece
    # at a time, in the 16 KiB chunks Ruby's inflater hands out.
    def inflate(text, buffer: nil, &block)
      self << text
      return take_read(&block) if block
      held = take_read
      return held if buffer.nil?
      buffer.replace held
    end

    # A nil argument says the compressed stream is over. Everything written
    # after that is passed through rather than read as part of it.
    def <<(text)
      if text.nil?
        @done = true
        return self
      end
      if @done
        @pending = @pending + Zlib.coerce_text(text)
      else
        @input = @input + Zlib.coerce_text(text)
        read_what_is_there
      end
      self
    end

    # What has been read and not handed back yet, which for a reader is the
    # text the stream stood for rather than a buffer of its own.
    def flush_next_out
      answer = take_read
      @finished = true if @done
      answer
    end

    def finish(&block)
      raise BufError, "buffer error" if !@done && !@input.empty?
      @finished = true
      return take_read(&block) if block
      take_read
    end

    # The dictionary the stream was written against, which it has to be
    # given before it can be read.
    def set_dictionary(text)
      @dictionary = Zlib.coerce_text text
      read_what_is_there
      text
    end

    private

    # A raw stream names no header, which is what a negative window size asks
    # for. Anything else carries the zlib header and checksum.
    # Read as much of a whole stream as the input holds. Anything after the
    # stream it names is passed through rather than read.
    def read_what_is_there
      return if @done || @input.empty?
      # A window of 16 or more names a gzip stream, and one of 32 or more
      # either kind, told apart by the gzip magic number.
      bits = @window_bits.to_i
      gzipped = bits >= 16 && (bits < 32 || @input.b.start_with?("\x1F\x8B".b))
      action = if gzipped
                 "gzip_part"
               elsif bits < 0
                 "raw_inflate_part"
               else
                 "inflate_part"
               end
      read = Zlib.__stream__ action, @input, 0, @dictionary.to_s
      raise NeedDict, "need dictionary" if read == :need_dictionary
      @handed_on ||= 0
      if read.nil?
        # What the part that arrived stands for is handed on now, as zlib
        # does, and the rest once more arrives.
        kind = gzipped ? "gzip" : (bits < 0 ? "raw" : "zlib")
        so_far = Zlib.__stream__ "inflate_so_far", @input, 0, @dictionary.to_s, kind
        fresh = so_far.byteslice(@handed_on, so_far.bytesize - @handed_on).to_s
        @pending = @pending + fresh
        @handed_on = so_far.bytesize
        return
      end
      whole = read[0]
      @pending = @pending + whole.byteslice(@handed_on, whole.bytesize - @handed_on).to_s + read[1]
      @handed_on = 0
      @input = ""
      @done = true
    end

    # Hand back what has been read and has not been handed back yet, all of
    # it at once or a chunk at a time to a block.
    def take_read
      unless block_given?
        answer = @pending
        @pending = "".b
        @output = @output + answer
        return answer.force_encoding(Encoding::BINARY)
      end
      while !@pending.empty?
        chunk = @pending.byteslice(0, CHUNK_BYTES)
        @pending = @pending.byteslice(CHUNK_BYTES, @pending.bytesize).to_s
        @output = @output + chunk
        yield chunk.force_encoding(Encoding::BINARY)
      end
      nil
    end
  end

  # Writes what it is given as a compressed stream.
  class Deflate < ZStream
    # With a block, the stream is handed over in pieces as it is made, and
    # the answer is nil.
    def self.deflate(text, level = DEFAULT_COMPRESSION, &block)
      return Zlib.deflate(text, level) if block.nil?
      new(level).deflate(text, FINISH, &block)
    end

    def initialize(level = DEFAULT_COMPRESSION, window_bits = MAX_WBITS,
                   mem_level = DEF_MEM_LEVEL, strategy = DEFAULT_STRATEGY)
      @level = level
      @window_bits = window_bits
      super()
    end

    # The pieces a block is handed are this many bytes, which is the buffer
    # zlib fills before Ruby hands it on.
    CHUNK_BYTES = 16384

    # What the stream holds past what was handed out already: everything
    # zlib has written once FINISH is asked for, and otherwise what it has
    # written while holding back the bytes a later one could still change.
    # A block is handed full pieces as they fill, and once the stream is
    # finished the piece left over too.
    def deflate(text, flush = NO_FLUSH, &block)
      @input = @input + Zlib.coerce_text(text) unless text.nil?
      @handed_out ||= 0
      if flush == FINISH
        written = Zlib.__stream__ "deflate", @input, 0, @dictionary.to_s
        @output = written
        @finished = true
      else
        reached = Zlib.__stream__ "deflate_handed_on", @input, 0, @dictionary.to_s
        written = Zlib.__stream__("deflate", @input, 0, @dictionary.to_s).byteslice(0, reached)
      end
      fresh = written.byteslice(@handed_out, written.bytesize - @handed_out) || "".b
      return hand_over(fresh) if block.nil?
      while fresh.bytesize >= CHUNK_BYTES
        piece = fresh.byteslice(0, CHUNK_BYTES)
        fresh = fresh.byteslice(CHUNK_BYTES, fresh.bytesize - CHUNK_BYTES)
        @handed_out += CHUNK_BYTES
        block.call piece
      end
      if @finished && !fresh.empty?
        @handed_out += fresh.bytesize
        block.call fresh
      end
      nil
    end

    def hand_over(fresh)
      @handed_out += fresh.bytesize
      fresh
    end
    private :hand_over

    def <<(text)
      @input = @input + Zlib.coerce_text(text) unless text.nil?
      self
    end

    def finish(&block)
      deflate nil, FINISH, &block
    end

    def flush(_kind = SYNC_FLUSH)
      "".b
    end

    def params(level, strategy)
      @level = level
      nil
    end

    # The dictionary the stream is written against, which a reader needs the
    # same of to read it back.
    def set_dictionary(text)
      @dictionary = Zlib.coerce_text text
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

    def initialize(io, **options)
      super()
      @io = io
      # What the stream holds is read as bytes and tagged with the encoding
      # the reader was told to read it in.
      @external_encoding = options[:external_encoding]
      # A stream is asked for everything it holds, named rather than left
      # out, since a reader may take the count as a required argument.
      held = io.read nil
      @body = Zlib.__stream__ "gunzip", held.to_s, 0
      @body = @body.dup.force_encoding @external_encoding unless @external_encoding.nil?
      @at = 0
      @line = 0
      @behind = 0
      read_header held.to_s
    end

    # What encoding the characters read are tagged with, which is the one the
    # reader was told to read in.
    attr_reader :external_encoding

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
      @behind = @behind + text.bytesize
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
