pub(super) const SOURCE: &str = r##"
module Kernel
  # `tap` hands the object to the block and answers the object itself. Ruby
  # writes it in Ruby, so it carries a source location and names itself in a
  # backtrace the way any other Ruby method does.
  def tap
    yield self
    self
  end

  # `putc` writes one character to the standard output stream.
  def putc(held)
    $stdout.putc held
  end
  module_function :putc

  # `test` names a file test by a single character, the way the shell's own
  # tests are spelled. The two-file tests take a second path.
  def test(command, first, second = nil)
    named = command.is_a?(Integer) ? command.chr : command.to_s
    case named
    when "b" then File.blockdev?(first)
    when "c" then File.chardev?(first)
    when "d" then File.directory?(first)
    when "e" then File.exist?(first)
    when "f" then File.file?(first)
    when "g" then File.setgid?(first)
    when "G" then File.grpowned?(first)
    when "k" then File.sticky?(first)
    when "l" then File.symlink?(first)
    when "o" then File.owned?(first)
    when "O" then File.owned?(first)
    when "p" then File.pipe?(first)
    when "r" then File.readable?(first)
    when "R" then File.readable_real?(first)
    when "s" then File.size?(first)
    when "S" then File.socket?(first)
    when "u" then File.setuid?(first)
    when "w" then File.writable?(first)
    when "W" then File.writable_real?(first)
    when "x" then File.executable?(first)
    when "X" then File.executable_real?(first)
    when "z" then File.zero?(first)
    when "A" then File.atime(first)
    when "C" then File.ctime(first)
    when "M" then File.mtime(first)
    when "-" then File.identical?(first, second)
    when "=" then File.mtime(first) == File.mtime(second)
    when "<" then File.mtime(first) < File.mtime(second)
    when ">" then File.mtime(first) > File.mtime(second)
    else
      raise ArgumentError, "unknown command #{named.inspect}"
    end
  end
  module_function :test

  # Which of the streams handed in are ready, which is IO.select under a name
  # every object answers to.
  def select(readers = nil, writers = nil, errored = nil, timeout = nil)
    IO.select readers, writers, errored, timeout
  end

  private :select

  # `pretty_inspect` is what `pp` writes for an object, which is its own
  # `inspect` on a line of its own. `require "pp"` is what defines it in
  # Ruby, and metorex reports pp as already loaded.
  def pretty_inspect
    inspect.to_s + "\n"
  end

  # Ruby calls into the operating system by number here. Metorex does not
  # reach the system call layer at all, which is what Ruby itself reports on
  # a platform that cannot.
  def syscall(*args)
    raise NotImplementedError, "syscall() function is unimplemented on this machine"
  end
  private :syscall

  # Ruby hands each interpreter event to the block set here. Metorex has no
  # tracing hook for the evaluator to call, so there is nothing to set.
  def set_trace_func(callable)
    raise NotImplementedError, "set_trace_func() function is unimplemented on this machine"
  end
  private :set_trace_func
end

# The stream `gets` reads from when a script is handed filenames: each named
# file in turn, read as though the whole list were one file. `ARGF` is the one
# the interpreter set up over ARGV, and `ARGF.class.new` builds another over a
# list of names, which is how the specs read a pair of fixtures.
ArgfStream = ARGF.class

class ArgfStream
  include Enumerable

  def initialize(*names)
    @names = names.flatten
    @current = nil
    @lineno = 0
    @binmode = false
    @drained = false
  end

  def to_s
    "ARGF"
  end

  def inspect
    "ARGF"
  end

  # The names still to be read. The one being read has already been taken off,
  # which is what makes `argv` shrink as the walk goes on.
  def argv
    self.__names__
  end

  # The handle now being read. The first name opens on the first ask, so the
  # file is current before a line has been taken from it.
  def file
    self.__open_current__
    @current
  end

  def to_io
    self.file
  end

  def path
    return "-" if @reading_stdin
    held = self.file
    return "-" if @reading_stdin
    held.path
  end

  def filename
    self.path
  end

  # The encodings the files are read as. ARGF keeps them for the files it has
  # yet to open as well as the one it is reading.
  def set_encoding(external, internal = nil)
    outer = external
    inner = internal
    if internal.nil? && external.is_a?(String) && external.include?(":")
      outer, inner = external.split(":", 2)
    end
    @external_encoding = outer.nil? || outer == "" ? nil : Encoding.find(outer)
    @internal_encoding = inner.nil? || inner == "" ? nil : Encoding.find(inner)
    self
  end

  def external_encoding
    return @external_encoding unless @external_encoding.nil?
    Encoding.default_external
  end

  def internal_encoding
    return @internal_encoding unless @internal_encoding.nil?
    Encoding.default_internal
  end

  # Text read from a file is tagged with the encoding ARGF reads as, and
  # carried into the internal one where there is one.
  def __tagged__(text)
    return text if text.nil? || text.empty?
    inner = self.internal_encoding
    outer = self.external_encoding
    return text.encode(inner, outer) unless inner.nil?
    return text if outer.nil?
    text.dup.force_encoding outer
  end
  private :__tagged__

  def fileno
    if @drained || (self.__names__.empty? && @current.nil?)
      raise ArgumentError, "closed stream"
    end
    self.file.fileno
  end

  def to_i
    self.fileno
  end

  def lineno
    self.__lineno__
  end

  def lineno=(counted)
    @lineno = counted
    $. = counted
  end

  def binmode
    @binmode = true
    self
  end

  def binmode?
    @binmode == true
  end

  def closed?
    # Standard input belongs to the program, so ARGF never reports it closed.
    return false if @reading_stdin
    self.file.closed?
  end

  def close
    self.file
    return self if @reading_stdin
    self.file.close
    self
  end

  # Move past whatever is left of the file being read, so the next line comes
  # from the one after it.
  def skip
    @current = nil unless self.__names__.empty?
    self
  end

  # Reading a line records which file it came from and how many have been
  # read, which is what `$FILENAME` and `$.` report.
  def gets(*separator)
    loop do
      self.__open_current__
      if @current.nil?
        $FILENAME = nil
        return nil
      end
      $FILENAME = @current.path
      line = @current.gets(*separator)
      if line.nil?
        if self.__names__.empty?
          @drained = true
          __finish_edit__
          break
        end
        @current = nil
        __finish_edit__
        next
      end
      @lineno = self.__lineno__ + 1
      $. = @lineno
      # Reading in binary hands back bytes rather than text.
      # `$_` names the line last read, which is what a program written
      # without a variable of its own reads back.
      $_ = @binmode ? line.b : __tagged__(line)
      return $_
    end
    nil
  end

  def readline(*separator)
    line = self.gets(*separator)
    raise EOFError, "end of file reached" if line.nil?
    line
  end

  # Every line of every file, read as though the list were one file. A
  # separator stands in for the newline, the way `IO#each_line` takes one.
  def each_line(*separator, &block)
    return to_enum(:each_line, *separator) if block.nil?
    while (line = self.gets(*separator))
      block.call line
    end
    self
  end

  def each(*separator, &block)
    self.each_line(*separator, &block)
  end

  def readlines(*args)
    collected = []
    while (line = self.gets)
      collected.push line
    end
    collected
  end

  def to_a(*args)
    self.readlines
  end

  # `read(length)` stops at the count it was asked for, crossing into the next
  # file only when the one being read runs out first. With no count it drains
  # every remaining file.
  def read(length = nil, buffer = nil)
    collected = ""
    loop do
      self.__open_current__
      break if @current.nil?
      wanted = length.nil? ? nil : length - collected.length
      break if !wanted.nil? && wanted <= 0
      taken = wanted.nil? ? @current.read : @current.read(wanted)
      collected = collected + taken.to_s
      break if !wanted.nil? && collected.length >= length
      if self.__names__.empty?
        @drained = true
        break
      end
      @current = nil
    end
    if collected.empty? && !length.nil? && length > 0
      buffer.replace "" unless buffer.nil?
      return nil
    end
    held = @binmode ? collected.b : __tagged__(collected)
    return buffer.replace held unless buffer.nil?
    held
  end

  # As much as is asked for of the file being read, which is never carried
  # across into the next one. The file running out hands back an empty String
  # and moves on, and running out of the last file is the end.
  def readpartial(length = nil, buffer = nil)
    if length.nil?
      raise ArgumentError, "wrong number of arguments (given 0, expected 1..2)"
    end
    buffer.replace "" unless buffer.nil?
    self.__open_current__
    raise EOFError, "end of file reached" if @current.nil?
    held = @current.read length
    if held.nil? || held.empty?
      if self.__names__.empty?
        @drained = true
        __finish_edit__
        raise EOFError, "end of file reached"
      end
      @current = nil
      __finish_edit__
      held = ""
    end
    held = @binmode ? held.b : __tagged__(held)
    return buffer.replace held unless buffer.nil?
    held
  end

  # As much of the file being read as is there right now, which is as much as
  # `readpartial` hands back for a file and never crosses into the next one.
  def read_nonblock(length = nil, buffer = nil, exception: true)
    if length.nil?
      raise ArgumentError, "wrong number of arguments (given 0, expected 1..2)"
    end
    buffer.replace "" unless buffer.nil?
    self.__open_current__
    raise EOFError, "end of file reached" if @current.nil?
    held = begin
      @current.read_nonblock length, nil, exception: exception
    rescue EOFError
      ""
    end
    return held unless held.is_a? String
    if held.empty?
      if self.__names__.empty?
        @drained = true
        __finish_edit__
        raise EOFError, "end of file reached"
      end
      @current = nil
      __finish_edit__
    end
    held = @binmode ? held.b : __tagged__(held)
    return buffer.replace held unless buffer.nil?
    held
  end

  def getc
    loop do
      self.__open_current__
      return nil if @current.nil?
      character = @current.getc
      return character unless character.nil?
      return nil if self.__names__.empty?
      @current = nil
    end
  end

  def readchar
    character = self.getc
    raise EOFError, "end of file reached" if character.nil?
    character
  end

  def each_byte(&block)
    return to_enum(:each_byte) if block.nil?
    while (byte = self.__next_byte__)
      block.call byte
    end
    self
  end

  def each_char(&block)
    return to_enum(:each_char) if block.nil?
    while (character = self.getc)
      block.call character
    end
    self
  end

  def each_codepoint(&block)
    return to_enum(:each_codepoint) if block.nil?
    while (character = self.getc)
      block.call character.ord
    end
    self
  end

  # The next byte of the list, crossing into the following file when the one
  # being read runs out.
  def __next_byte__
    loop do
      self.__open_current__
      return nil if @current.nil?
      byte = @current.getbyte
      return byte unless byte.nil?
      return nil if self.__names__.empty?
      @current = nil
    end
  end
  private :__next_byte__

  # Whether the file being read has run out, which is asked per file rather
  # than of the whole list. A stream whose last file was drained is closed,
  # and refuses the question.
  def eof?
    raise IOError, "closed stream" if @drained
    self.__open_current__
    return true if @current.nil?
    @current.eof?
  end

  def eof
    self.eof?
  end

  # Where the file being read stands. A stream whose last file has been read
  # to the end is closed, and refuses to report a position at all.
  def pos
    raise ArgumentError, "closed stream" if @drained
    self.file.pos
  end

  def tell
    self.pos
  end

  def pos=(offset)
    self.file.pos = offset
    offset
  end

  # Seeking moves the file now being read. A stream that has not opened one
  # opens the first name first, so the offset lands somewhere.
  def seek(offset, whence = IO::SEEK_SET)
    self.file.seek offset, whence
  end

  # A stream whose last file has been read to the end is closed, and closed
  # streams cannot be put back to the start.
  def rewind
    raise ArgumentError, "closed stream" if @drained
    self.file.rewind
    @lineno = 0
    $. = 0
    0
  end

  # Open the next name when there is no file being read. The name comes off
  # the list as it opens, which is what `argv` reports on.
  def __open_current__
    return if @current
    # A stream over no names at all reads standard input, which is what a
    # program handed nothing on the command line reads from.
    if self.__names__.empty?
      return if @walked
      @walked = true
      @reading_stdin = true
      @current = $stdin
      return @current
    end
    @walked = true
    named = self.__names__.shift
    # A name of `-` stands for standard input, which the program owns rather
    # than ARGF.
    if named == "-"
      @reading_stdin = true
      @current = $stdin
      return @current
    end
    @reading_stdin = false
    # `-i` edits each file as it is read: the file is set aside under the
    # backup name, and what the program writes takes its place.
    extension = $-i
    unless extension.nil?
      @edited_name = named
      @backup_name = named + (extension.empty? ? ".__metorex_edit__" : extension)
      File.rename named, @backup_name
      @current = File.open(@backup_name, "r")
      @written = File.open(named, "w")
      $stdout = @written
      return @current
    end
    @current = File.open(named, "r")
  end
  private :__open_current__

  # Close the file the program was writing in place of the one being read,
  # and put standard output back where it was.
  def __finish_edit__
    return if @written.nil?
    @written.close
    @written = nil
    $stdout = STDOUT
    # A backup asked for under no extension is not kept.
    if !@backup_name.nil? && @backup_name.end_with?(".__metorex_edit__")
      File.unlink @backup_name
    end
    @backup_name = nil
    nil
  end
  private :__finish_edit__

  # The names still to be read. The interpreter builds the global ARGF without
  # running `initialize`, and that one reads ARGV itself, so a name taken off
  # here is taken off ARGV too.
  def __names__
    @names = ARGV if @names.nil?
    @names
  end
  private :__names__

  # How many lines have been read, zero before any have been.
  def __lineno__
    @lineno = 0 if @lineno.nil?
    @lineno
  end
  private :__lineno__
end

# A named set of threads. Ruby starts every thread in the default group and
# moves it when another group takes it, and an enclosed group refuses to give
# its threads up.
class Complex
  # What `Marshal` writes for a Complex: the two parts, in the order
  # `Complex(real, imaginary)` takes them.
  def marshal_dump
    [real, imaginary]
  end
  private :marshal_dump
end

class Float
  # The simplest fraction standing no further away than the tolerance given.
  # Without one the tolerance is half the gap to the next Float, so the
  # answer reads back as this same Float.
  def rationalize(*limits)
    if limits.size > 1
      raise ArgumentError, "wrong number of arguments (given #{limits.size}, expected 0..1)"
    end
    raise FloatDomainError, to_s if nan? || infinite?
    return to_r.rationalize(limits[0]) unless limits.empty?
    to_r.rationalize(Rational(Math.ldexp(1, Math.frexp(self)[1] - 53).to_r, 2))
  end
end
"##;
