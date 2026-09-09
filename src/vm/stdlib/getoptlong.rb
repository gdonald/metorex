# Command line options read the way the GNU getopt_long function reads them.
class GetoptLong
  # What an option does with the word that follows it.
  ORDERINGS = [REQUIRE_ORDER = 0, PERMUTE = 1, RETURN_IN_ORDER = 2].freeze
  ARGUMENT_FLAGS = [NO_ARGUMENT = 0, REQUIRED_ARGUMENT = 1,
                    OPTIONAL_ARGUMENT = 2].freeze

  STATUS_YET = 0
  STATUS_STARTED = 1
  STATUS_TERMINATED = 2

  class Error < StandardError
  end

  class AmbiguousOption < Error
  end

  class NeedlessArgument < Error
  end

  class MissingArgument < Error
  end

  class InvalidOption < Error
  end

  def initialize(*arguments)
    @ordering = ENV.include?("POSIXLY_CORRECT") ? REQUIRE_ORDER : PERMUTE
    @canonical_names = {}
    @argument_flags = {}
    @non_option_arguments = []
    @rest_singles = ""
    @status = STATUS_YET
    @error = nil
    @error_message = nil
    @quiet = false
    self.set_options(*arguments) unless arguments.empty?
  end

  def ordering
    @ordering
  end

  # The order options and plain words are read in. Changing it once reading
  # has begun is refused, and PERMUTE is refused outright where the
  # environment asks for the POSIX order.
  def ordering=(ordering)
    if @status != STATUS_YET
      self.set_error(ArgumentError, "argument error")
    end
    unless ORDERINGS.include?(ordering)
      raise ArgumentError, "invalid ordering `#{ordering}'"
    end
    if ordering == PERMUTE && ENV.include?("POSIXLY_CORRECT")
      @ordering = REQUIRE_ORDER
    else
      @ordering = ordering
    end
    @ordering
  end

  def quiet
    @quiet
  end

  def quiet?
    @quiet
  end

  def quiet=(flag)
    @quiet = flag
  end

  def error
    @error
  end

  def error?
    !@error.nil?
  end

  def error_message
    @error_message
  end

  def set_error(type, message)
    $stderr.print("#{$0}: #{message}\n") unless @quiet
    @error = type
    @error_message = message
    raise type, message
  end
  private :set_error

  # The options to read, each written as the names that stand for it followed
  # by what it does with the word after it.
  def set_options(*arguments)
    if @status != STATUS_YET
      raise RuntimeError, "invoke set_options, but option processing has already started"
    end
    @canonical_names = {}
    @argument_flags = {}
    arguments.each do |argument|
      unless argument.is_a?(Array)
        raise ArgumentError, "the option list contains non-Array argument"
      end
      flag = nil
      argument.each do |part|
        next unless ARGUMENT_FLAGS.include?(part)
        raise ArgumentError, "too many argument-flags" unless flag.nil?
        flag = part
      end
      raise ArgumentError, "no argument-flag" if flag.nil?
      canonical = nil
      argument.each do |name|
        next if name == flag
        begin
          if !name.is_a?(String) || (/\A-([^-]|-.+)\z/ =~ name).nil?
            raise ArgumentError, "an invalid option `#{name}'"
          end
          if @canonical_names.include?(name)
            raise ArgumentError, "option redefined `#{name}'"
          end
        rescue ArgumentError => problem
          @canonical_names = {}
          @argument_flags = {}
          raise problem
        end
        canonical = name if canonical.nil?
        @canonical_names[name] = canonical
        @argument_flags[name] = flag
      end
    end
    self
  end

  def terminate
    return nil if @status == STATUS_TERMINATED
    raise RuntimeError, "an error has occurred" unless @error.nil?
    @status = STATUS_TERMINATED
    @non_option_arguments.reverse.each { |argument| ARGV.unshift(argument) }
    @canonical_names = nil
    @argument_flags = nil
    @rest_singles = nil
    @non_option_arguments = nil
    self
  end

  def terminated?
    @status == STATUS_TERMINATED
  end

  # The next option and the word it carries, or nil once there are no more.
  def get
    option_name = nil
    option_argument = ""
    return nil unless @error.nil?
    if @status == STATUS_YET
      @non_option_arguments = []
      @status = STATUS_STARTED
    elsif @status == STATUS_TERMINATED
      return nil
    end
    argument = self.next_word
    return nil if argument.nil?
    if argument == "--" && @rest_singles.length == 0
      self.terminate
      return nil
    end
    if @rest_singles.length == 0 && !(/\A(--[^=]*)(?:=(.*))?\z/m =~ argument).nil?
      option_name, option_argument = self.read_long(argument, $1, $2)
    elsif !(/\A(-(.))(.*)\z/m =~ argument).nil?
      option_name, option_argument = self.read_short($1, $2, $3)
    else
      return ["", argument]
    end
    [@canonical_names[option_name], option_argument]
  end

  # The word the next option is read from, taking the ordering into account.
  def next_word
    return "-" + @rest_singles if @rest_singles.length > 0
    if ARGV.length == 0
      self.terminate
      return nil
    end
    if @ordering == PERMUTE
      while ARGV.length > 0 && (/\A-./m =~ ARGV[0]).nil?
        @non_option_arguments.push(ARGV.shift)
      end
      if ARGV.length == 0
        self.terminate
        return nil
      end
      return ARGV.shift
    end
    if @ordering == REQUIRE_ORDER && (/\A-./m =~ ARGV[0]).nil?
      self.terminate
      return nil
    end
    ARGV.shift
  end
  private :next_word

  # An option written in full, which may carry its word after an `=` sign or
  # in the word that follows it. A shortened name is taken when only one
  # option starts with it.
  def read_long(argument, pattern, written)
    option_name = pattern
    unless @canonical_names.include?(pattern)
      matches = @canonical_names.keys.select { |key| key.index(pattern) == 0 }
      option_name = matches[0]
      if matches.length >= 2
        self.set_error(AmbiguousOption,
                       "option `#{argument}' is ambiguous between #{matches.join(', ')}")
      elsif matches.length == 0
        self.set_error(InvalidOption, "unrecognized option `#{argument}'")
      end
    end
    flag = @argument_flags[option_name]
    if flag == REQUIRED_ARGUMENT
      if written.nil?
        if ARGV.length > 0
          written = ARGV.shift
        else
          self.set_error(MissingArgument, "option `#{argument}' requires an argument")
        end
      end
    elsif flag == OPTIONAL_ARGUMENT
      if written.nil?
        if ARGV.length > 0 && (/\A-./m =~ ARGV[0]).nil?
          written = ARGV.shift
        else
          written = ""
        end
      end
    elsif !written.nil?
      self.set_error(NeedlessArgument, "option `#{option_name}' doesn't allow an argument")
    end
    [option_name, written.nil? ? "" : written]
  end
  private :read_long

  # An option written with one letter, which may be run together with the
  # ones after it and may carry its word in the same run.
  def read_short(option_name, letter, rest)
    @rest_singles = rest
    written = ""
    unless @canonical_names.include?(option_name)
      self.set_error(InvalidOption, "invalid option -- #{letter}")
    end
    flag = @argument_flags[option_name]
    if flag == REQUIRED_ARGUMENT
      if @rest_singles.length > 0
        written = @rest_singles
        @rest_singles = ""
      elsif ARGV.length > 0
        written = ARGV.shift
      else
        self.set_error(MissingArgument, "option requires an argument -- #{letter}")
      end
    elsif flag == OPTIONAL_ARGUMENT
      if @rest_singles.length > 0
        written = @rest_singles
        @rest_singles = ""
      elsif ARGV.length > 0 && (/\A-./m =~ ARGV[0]).nil?
        written = ARGV.shift
      end
    end
    [option_name, written]
  end
  private :read_short

  def get_option
    self.get
  end

  def each(&block)
    walked = self.get
    while !walked.nil?
      block.call(walked[0], walked[1])
      walked = self.get
    end
    self
  end

  def each_option(&block)
    self.each(&block)
  end
end
