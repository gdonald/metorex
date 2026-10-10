# What io/console adds to IO: the terminal's modes, its size, the cursor and
# the screen, written the way console.c of io-console 0.8.2, the version MRI
# 4.0 carries, writes them for a POSIX terminal. The settings themselves are
# read and changed natively.

class IO
  # A terminal's settings, held as the bytes the system keeps them in.
  class ConsoleMode
    def initialize_copy(other)
      super
      @settings = other.__settings__.dup
    end

    def echo=(flag)
      @settings = __console_mode_change__(@settings, flag ? :echo : :noecho, -1, -1, nil)
    end

    def raw!(min: nil, time: nil, intr: nil)
      vmin, vtime, interrupts = IO.__raw_options__(min, time, intr)
      @settings = __console_mode_change__(@settings, :raw, vmin, vtime, interrupts)
      self
    end

    def raw(**options)
      dup.raw!(**options)
    end

    def __settings__
      @settings
    end

    def __settings__=(settings)
      @settings = settings
    end
  end

  # The `min:`, `time:` and `intr:` options of `raw`, as the bytes the
  # settings hold. `time:` is in seconds, kept in tenths.
  def self.__raw_options__(min, time, intr)
    unless intr.nil? || intr == true || intr == false
      raise ArgumentError, "true or false expected as intr: #{intr.inspect}"
    end
    vmin = min.nil? ? -1 : min.to_int & 0xff
    vtime = time.nil? ? -1 : (time * 10).to_int & 0xff
    [vmin, vtime, intr]
  end

  def __console_path__
    respond_to?(:path) && path ? path : inspect
  end
  private :__console_path__

  def __console_fail__(errno)
    raise SystemCallError.new(__console_path__, errno)
  end
  private :__console_fail__

  def __console_settings__
    settings = __console_mode_get__(fileno)
    __console_fail__(settings) if settings.is_a?(Integer)
    settings
  end
  private :__console_settings__

  def __console_apply__(settings)
    errno = __console_mode_set__(fileno, settings)
    __console_fail__(errno) if errno
  end
  private :__console_apply__

  # Run the block with the settings changed, putting back what was there
  # however the block ends.
  def __console_ttymode__(change, vmin = -1, vtime = -1, intr = nil)
    saved = __console_settings__
    __console_apply__(__console_mode_change__(saved, change, vmin, vtime, intr))
    begin
      yield self
    ensure
      __console_apply__(saved)
    end
  end
  private :__console_ttymode__

  def raw(min: nil, time: nil, intr: nil, &block)
    vmin, vtime, interrupts = IO.__raw_options__(min, time, intr)
    __console_ttymode__(:raw, vmin, vtime, interrupts, &block)
  end

  def raw!(min: nil, time: nil, intr: nil)
    vmin, vtime, interrupts = IO.__raw_options__(min, time, intr)
    __console_apply__(__console_mode_change__(__console_settings__, :raw, vmin, vtime, interrupts))
    self
  end

  def cooked(&block)
    __console_ttymode__(:cooked, &block)
  end

  def cooked!
    __console_apply__(__console_mode_change__(__console_settings__, :cooked, -1, -1, nil))
    self
  end

  def getch(min: nil, time: nil, intr: nil)
    raw(min: min, time: time, intr: intr) { getc }
  end

  def noecho(&block)
    __console_ttymode__(:noecho, &block)
  end

  def echo=(flag)
    __console_apply__(__console_mode_change__(__console_settings__, flag ? :echo : :noecho, -1, -1, nil))
    flag
  end

  def echo?
    __console_mode_query__(__console_settings__, :echo)
  end

  def console_mode
    mode = ConsoleMode.allocate
    mode.__settings__ = __console_settings__
    mode
  end

  def console_mode=(mode)
    __console_apply__(mode.__settings__)
    mode
  end

  def winsize
    size = __console_winsize__(fileno)
    __console_fail__(size) if size.is_a?(Integer)
    size
  end

  def winsize=(size)
    size = Array(size)
    unless size.length == 2 || size.length == 4
      raise ArgumentError, "wrong number of arguments (given #{size.length}, expected 2 or 4)"
    end
    row, column, xpixel, ypixel = size.map { |held| held.nil? ? 0 : held.to_int }
    errno = __console_set_winsize__(fileno, row, column, xpixel || 0, ypixel || 0)
    __console_fail__(errno) if errno
    size
  end

  def iflush
    errno = __console_flush__(fileno, 0)
    __console_fail__(errno) if errno
    self
  end

  def oflush
    errno = __console_flush__(fileno, 1)
    __console_fail__(errno) if errno
    self
  end

  def ioflush
    errno = __console_flush__(fileno, 2)
    __console_fail__(errno) if errno
    self
  end

  def beep
    errno = __console_beep__(fileno)
    __console_fail__(errno) if errno
    self
  end

  # Where the cursor stands, asked of the terminal, as a zero-based row and
  # column, or nil when the terminal answers something else.
  def cursor
    response = raw do
      write("\e[6n")
      flush
      next nil unless getbyte == 0x1b && getbyte == "[".ord
      numbers = []
      number = 0
      last = nil
      while (byte = getbyte)
        if byte == ";".ord
          numbers << number
          number = 0
        elsif byte.between?("0".ord, "9".ord)
          number = number * 10 + byte - "0".ord
        else
          numbers << number
          last = byte.chr
          break
        end
      end
      numbers << last
    end
    return nil unless response.is_a?(Array) && response.length == 3 && response[2] == "R"
    [response[0] - 1, response[1] - 1]
  end

  def cursor=(position)
    position = position.to_ary
    raise ArgumentError, "expected 2D coordinate" unless position.length == 2
    goto(position[0], position[1])
  end

  def goto(row, column)
    write(format("\e[%d;%dH", row.to_int + 1, column.to_int + 1))
    self
  end

  def __console_move__(rows, columns)
    if rows != 0 || columns != 0
      moved = +""
      moved << format("\e[%d%s", rows.abs, rows < 0 ? "A" : "B") if rows != 0
      moved << format("\e[%d%s", columns.abs, columns < 0 ? "D" : "C") if columns != 0
      write(moved)
      flush
    end
    self
  end
  private :__console_move__

  def cursor_up(count) = __console_move__(-count.to_int, 0)
  def cursor_down(count) = __console_move__(count.to_int, 0)
  def cursor_left(count) = __console_move__(0, -count.to_int)
  def cursor_right(count) = __console_move__(0, count.to_int)

  def goto_column(column)
    write(format("\e[%dG", column.to_int + 1))
    self
  end

  def __console_erase_mode__(mode, highest, name)
    return 0 if mode.nil?
    unless mode.is_a?(Integer) && mode.between?(0, highest)
      raise ArgumentError, "wrong #{name} mode: #{mode}"
    end
    mode
  end
  private :__console_erase_mode__

  def erase_line(mode)
    write(format("\e[%dK", __console_erase_mode__(mode, 2, "line erase")))
    self
  end

  def erase_screen(mode)
    write(format("\e[%dJ", __console_erase_mode__(mode, 3, "screen erase")))
    self
  end

  def __console_scroll__(lines)
    write(format("\e[%d%s", lines.abs, lines < 0 ? "T" : "S")) if lines != 0
    self
  end
  private :__console_scroll__

  def scroll_forward(lines) = __console_scroll__(lines.to_int)
  def scroll_backward(lines) = __console_scroll__(-lines.to_int)

  def clear_screen
    erase_screen(2)
    goto(0, 0)
  end

  # Read a line with echo off, after writing the prompt, and end the line
  # the reader did not see echoed. A prompt for standard input goes to
  # standard error.
  def getpass(prompt = nil)
    out = equal?($stdin) ? $stderr : self
    out.write(prompt) unless prompt.nil?
    out.flush
    begin
      line = noecho { gets }
    ensure
      out.write($/)
    end
    line&.chomp!
    line
  end

  def ttyname
    __console_ttyname__(fileno)
  end

  # The terminal the process belongs to, opened once and kept. `:close`
  # closes it, and any other symbol is sent to it.
  def self.console(*args)
    held = @__console__
    @__console__ = held = nil if held && held.closed?
    unless args.empty?
      raise TypeError, "wrong argument type #{args[0].class} (expected Symbol)" unless args[0].is_a?(Symbol)
      if args[0] == :close && args.length == 1
        held&.close
        @__console__ = nil
        return nil
      end
    end
    unless held
      begin
        held = File.open("/dev/tty", File::RDWR)
      rescue SystemCallError
        return nil
      end
      held.sync = true
      @__console__ = held
    end
    return held.__send__(*args) unless args.empty?
    held
  end
end

# A stream that is not a terminal reads a key and a password the plain way.
class StringIO
  def getch(*args)
    getc(*args)
  end

  def getpass(prompt = nil)
    write(prompt) unless prompt.nil?
    begin
      line = gets($/)
    ensure
      write($/)
    end
    line&.chomp!
    line
  end
end
