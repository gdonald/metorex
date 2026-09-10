# A log with a severity on each message. The messages go to an IO or to a
# file named by path, and the log device rotates that file when it is asked
# to keep more than one.

class Logger
  # The severities, from the one that is always shown to the one that is
  # shown only when everything is.
  module Severity
    DEBUG = 0
    INFO = 1
    WARN = 2
    ERROR = 3
    FATAL = 4
    UNKNOWN = 5
  end

  include Severity

  DEBUG = Severity::DEBUG
  INFO = Severity::INFO
  WARN = Severity::WARN
  ERROR = Severity::ERROR
  FATAL = Severity::FATAL
  UNKNOWN = Severity::UNKNOWN

  SEV_LABEL = ["DEBUG", "INFO", "WARN", "ERROR", "FATAL", "ANY"]

  class Error < RuntimeError; end
  class ShiftingError < Error; end

  attr_accessor :level
  attr_accessor :progname
  attr_accessor :formatter
  attr_reader :datetime_format

  # Where the messages go, and how the file behind them is rotated.
  class LogDevice
    attr_reader :dev
    attr_reader :filename

    def initialize(log = nil, shift_age: 0, shift_size: 1048576, **_rest)
      @shift_age = shift_age
      @shift_size = shift_size
      @filename = nil
      # A stream is written to as it stands. Only a name opens a file, and
      # anything that is neither is refused: `to_s` on an object that is not
      # a name describes it, and opening that would make a file called
      # `#<File:0x...>` wherever the program happened to be running.
      if log.respond_to?(:write) && log.respond_to?(:close)
        @dev = log
      elsif log.is_a? ::String
        @filename = log
        @dev = create_logfile @filename
      elsif log.respond_to? :to_path
        @filename = log.to_path
        @dev = create_logfile @filename
      else
        raise ArgumentError, "#{log.inspect} is not a supported output target"
      end
    end

    def write(message)
      begin
        check_shift_log
      rescue => problem
        warn "log shifting failed. #{problem.message}"
        return nil
      end
      begin
        raise IOError, "closed stream" if @dev.nil? || @dev.closed?
        @dev.write message
        message.to_s.length
      rescue => problem
        warn "log writing failed. #{problem.message}"
        nil
      end
    end

    def close
      @dev.close unless @dev.nil? || @dev.closed?
      nil
    end

    def reopen(log = nil)
      close
      @filename = log.to_s unless log.nil?
      @dev = create_logfile @filename
      self
    end

    private

    # A log file metorex makes itself opens with a line saying when it was
    # made. One that was already there is written to as it stands, so the
    # header never lands in the middle of a log.
    def create_logfile(path)
      fresh = !File.exist?(path)
      handle = File.open path, "a"
      handle.write "# Logfile created on #{Time.now}\n" if fresh
      handle
    end

    # Whether the log has grown past the size it may reach. A device made
    # from a stream that was handed in has to ask the stream about itself,
    # which a closed one refuses.
    def check_shift_log
      if @filename.nil?
        raise IOError, "closed stream" if @dev.nil? || @dev.closed?
        return nil
      end
      shift_log if shifting_needed?
      nil
    end

    def shifting_needed?
      return false if @shift_age.to_i < 1
      File.size(@filename) > @shift_size
    end

    # Rotation moves the file aside under a numbered name and starts a new
    # one, keeping as many as the shift age asks for.
    def shift_log
      kept = @shift_age.to_i - 1
      @dev.close unless @dev.closed?
      (kept - 1).downto(0) do |slot|
        older = "#{@filename}.#{slot}"
        File.rename older, "#{@filename}.#{slot + 1}" if File.exist? older
      end
      File.rename @filename, "#{@filename}.0"
      @dev = create_logfile @filename
    end
  end

  # The default line: severity, time, program name, and the message.
  class Formatter
    FORMAT = "%s, [%s] %5s -- %s: %s\n"

    attr_accessor :datetime_format

    def call(severity, time, progname, message)
      FORMAT % [
        severity[0, 1],
        format_datetime(time),
        severity,
        progname,
        message_text(message)
      ]
    end

    private

    def format_datetime(time)
      return time.strftime(@datetime_format) unless @datetime_format.nil?
      time.strftime("%Y-%m-%dT%H:%M:%S.%6N")
    end

    def message_text(message)
      case message
      when ::String then message
      when ::Exception then "#{message.message} (#{message.class})"
      else message.inspect
      end
    end
  end

  def initialize(logdev, shift_age = 0, shift_size = 1048576, level: DEBUG,
                 progname: nil, formatter: nil, datetime_format: nil, **_rest)
    @level = coerce_level level
    @progname = progname
    @formatter = formatter
    @default_formatter = Formatter.new
    self.datetime_format = datetime_format
    @logdev = nil
    return if logdev.nil?
    @logdev = LogDevice.new logdev, shift_age: shift_age, shift_size: shift_size
  end

  def datetime_format=(format)
    @datetime_format = format
    @default_formatter.datetime_format = format
  end

  def add(severity, message = nil, progname = nil)
    severity = UNKNOWN if severity.nil?
    return true if @logdev.nil? || severity < @level
    if message.nil?
      if block_given?
        message = yield
      else
        message = progname
        progname = @progname
      end
    end
    progname = @progname if progname.nil?
    @logdev.write format_message(format_severity(severity), Time.now, progname, message)
    true
  end

  alias_method :log, :add

  def <<(message)
    return nil if @logdev.nil?
    @logdev.write message
  end

  def debug(progname = nil, &block)
    add DEBUG, nil, progname, &block
  end

  def info(progname = nil, &block)
    add INFO, nil, progname, &block
  end

  def warn(progname = nil, &block)
    add WARN, nil, progname, &block
  end

  def error(progname = nil, &block)
    add ERROR, nil, progname, &block
  end

  def fatal(progname = nil, &block)
    add FATAL, nil, progname, &block
  end

  def unknown(progname = nil, &block)
    add UNKNOWN, nil, progname, &block
  end

  def debug?
    @level <= DEBUG
  end

  def info?
    @level <= INFO
  end

  def warn?
    @level <= WARN
  end

  def error?
    @level <= ERROR
  end

  def fatal?
    @level <= FATAL
  end

  def level=(severity)
    @level = coerce_level severity
  end

  def close
    @logdev.close unless @logdev.nil?
  end

  def reopen(logdev = nil)
    @logdev.reopen logdev unless @logdev.nil?
    self
  end

  private

  # A severity named by symbol or string reads back as the number it stands
  # for, so `level: :info` and `level: Logger::INFO` mean the same thing.
  def coerce_level(severity)
    return severity if severity.is_a? Integer
    named = severity.to_s.upcase
    place = SEV_LABEL.index named
    place = UNKNOWN if named == "UNKNOWN"
    if place.nil?
      raise ArgumentError, "invalid log level: #{severity}"
    end
    place
  end

  def format_severity(severity)
    SEV_LABEL[severity] || "ANY"
  end

  def format_message(severity, time, progname, message)
    if @formatter.nil?
      @default_formatter.call severity, time, progname, message
    else
      @formatter.call severity, time, progname, message
    end
  end
end
