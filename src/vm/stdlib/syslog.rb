# The system log. A program opens the log under a name, writes messages at a
# severity, and closes it. Metorex writes through the C library, and with
# LOG_PERROR set it also writes the line to the error stream, which is where
# a program watching its own output sees it.

module Syslog
  # The names the C library gives the severities, the facilities, and the
  # options a log may be opened with.
  module Constants
    LOG_EMERG = 0
    LOG_ALERT = 1
    LOG_CRIT = 2
    LOG_ERR = 3
    LOG_WARNING = 4
    LOG_NOTICE = 5
    LOG_INFO = 6
    LOG_DEBUG = 7

    LOG_KERN = 0 << 3
    LOG_USER = 1 << 3
    LOG_MAIL = 2 << 3
    LOG_DAEMON = 3 << 3
    LOG_AUTH = 4 << 3
    LOG_SYSLOG = 5 << 3
    LOG_LPR = 6 << 3
    LOG_NEWS = 7 << 3
    LOG_UUCP = 8 << 3
    LOG_CRON = 9 << 3
    LOG_AUTHPRIV = 10 << 3
    LOG_FTP = 11 << 3
    LOG_LOCAL0 = 16 << 3
    LOG_LOCAL1 = 17 << 3
    LOG_LOCAL2 = 18 << 3
    LOG_LOCAL3 = 19 << 3
    LOG_LOCAL4 = 20 << 3
    LOG_LOCAL5 = 21 << 3
    LOG_LOCAL6 = 22 << 3
    LOG_LOCAL7 = 23 << 3

    LOG_PID = 0x01
    LOG_CONS = 0x02
    LOG_ODELAY = 0x04
    LOG_NDELAY = 0x08
    LOG_NOWAIT = 0x10
    LOG_PERROR = 0x20

    # The mask bit one severity stands for.
    def LOG_MASK(priority)
      1 << priority
    end

    # The mask bit for every severity down to the one named.
    def LOG_UPTO(priority)
      (1 << (priority + 1)) - 1
    end

    module_function :LOG_MASK, :LOG_UPTO
  end

  include Constants
  extend Constants

  @opened = false
  @ident = nil
  @options = nil
  @facility = nil
  @mask = nil
  @inside_block = false

  class << self
    def opened?
      @opened
    end

    def instance
      self
    end

    def ident
      @opened ? @ident : nil
    end

    def options
      @opened ? @options : nil
    end

    def facility
      @opened ? @facility : nil
    end

    def mask
      @opened ? @mask : nil
    end

    # A mask is a number, so a Float loses its fraction and anything that is
    # not a number at all is refused.
    def mask=(held)
      raise RuntimeError, "must open syslog before setting log mask" unless @opened
      unless held.is_a?(Integer) || held.is_a?(Float)
        raise TypeError, "no implicit conversion of #{held.class} into Integer"
      end
      @mask = held.to_i
    end

    # Open the log under a name. With a block the log is handed over and
    # closed afterwards.
    def open(ident = nil, options = nil, facility = nil)
      raise RuntimeError, "syslog already open" if @opened
      @ident = ident.nil? ? $0 : ident.to_s
      @options = options.nil? ? Constants::LOG_PID | Constants::LOG_CONS : options
      @facility = facility.nil? ? Constants::LOG_USER : facility
      @mask = 255
      @opened = true
      return self unless block_given?
      @inside_block = true
      begin
        yield self
      ensure
        @inside_block = false
        Syslog.close if @opened
      end
    end

    # Close and open again under a new name, keeping the mask in force.
    def reopen(ident = nil, options = nil, facility = nil, &block)
      raise RuntimeError, "syslog not opened" unless @opened
      held = @mask
      close
      answered = Syslog.open ident, options, facility, &block
      @mask = held if @opened
      answered
    end

    alias_method :open!, :reopen

    def close
      raise RuntimeError, "syslog not opened" unless @opened
      # The block an `open` handed the log to owns it until the block ends,
      # so nothing inside may close it early.
      raise RuntimeError, "syslog opened with a block" if @inside_block
      @opened = false
      @ident = nil
      @options = nil
      @facility = nil
      @mask = nil
      nil
    end

    def inspect
      return "#<Syslog: opened=false>" unless @opened
      first = "#<Syslog: opened=true, ident=\"#{@ident}\", options=#{@options}"
      first + ", facility=#{@facility}, mask=#{@mask}>"
    end

    # Write one message at the given severity. Anything the mask has switched
    # off is dropped before it goes anywhere.
    def log(priority, format = nil, *pieces)
      raise RuntimeError, "must open syslog before write" unless @opened
      return self if format.nil?
      return self if @mask & (1 << (priority & 7)) == 0
      written = pieces.empty? ? format.to_s : format.to_s % pieces
      Syslog.__write__ @ident, @options, @facility | (priority & 7), written
      self
    end

    def emerg(format = nil, *pieces)
      log Constants::LOG_EMERG, format, *pieces
    end

    def alert(format = nil, *pieces)
      log Constants::LOG_ALERT, format, *pieces
    end

    def crit(format = nil, *pieces)
      log Constants::LOG_CRIT, format, *pieces
    end

    def err(format = nil, *pieces)
      log Constants::LOG_ERR, format, *pieces
    end

    def warning(format = nil, *pieces)
      log Constants::LOG_WARNING, format, *pieces
    end

    def notice(format = nil, *pieces)
      log Constants::LOG_NOTICE, format, *pieces
    end

    def info(format = nil, *pieces)
      log Constants::LOG_INFO, format, *pieces
    end

    def debug(format = nil, *pieces)
      log Constants::LOG_DEBUG, format, *pieces
    end
  end
end
