# Reading command-line options against a description of what they mean.

class OptionParser
  class ParseError < StandardError
  end

  class InvalidOption < ParseError
  end

  class MissingArgument < ParseError
  end

  class InvalidArgument < ParseError
  end

  class AmbiguousOption < ParseError
  end

  # One option: the names it answers to, whether it takes a value, and what
  # to do when it is seen.
  class Switch
    attr_reader :short
    attr_reader :long
    attr_reader :negatable
    attr_reader :argument
    attr_reader :description
    attr_reader :handler

    def initialize short, long, negatable, argument, description, handler
      @short = short
      @long = long
      @negatable = negatable
      @argument = argument
      @description = description
      @handler = handler
    end

    # The name a value is stored under, which is the long form when there is
    # one and the short form otherwise.
    def key
      (@long || @short).to_sym
    end
  end

  attr_accessor :banner
  attr_accessor :program_name
  attr_accessor :version
  attr_accessor :summary_width
  attr_accessor :summary_indent

  def initialize banner = nil, width = 32, indent = " " * 4
    @banner = banner
    @summary_width = width
    @summary_indent = indent
    @program_name = File.basename($PROGRAM_NAME.to_s)
    @switches = []
    yield self if block_given?
  end

  # `on` describes one option. A name in `--[no-]name` form also answers to
  # `--no-name`, and a name followed by a word takes a value.
  def on *specs, &handler
    short = nil
    long = nil
    negatable = false
    argument = nil
    description = nil
    specs.each do |spec|
      held = spec.to_s
      if held.start_with? "--"
        name, wanted = held[2..-1].to_s.split(/[ =]/, 2)
        argument = wanted unless wanted.nil? || wanted.empty?
        if name.start_with? "[no-]"
          negatable = true
          name = name["[no-]".length..-1]
        end
        long = name.gsub "-", "_"
      elsif held.start_with?("-") && held.length > 1
        name, wanted = held[1..-1].to_s.split(/[ =]/, 2)
        # A space after the letter says the option takes a value, whether or
        # not a name for the value follows it.
        argument = wanted unless wanted.nil?
        short = name
      else
        description = held
      end
    end
    @switches << Switch.new(short, long, negatable, argument, description, handler)
    self
  end

  alias_method :on_tail, :on
  alias_method :on_head, :on

  def separator text
    @switches << Switch.new(nil, nil, false, nil, text, nil)
    self
  end

  # Read the options out of `argv`, leaving what is not an option behind.
  # The options read in order. An argument that is not an option is handed
  # to the block, and without a block it ends the reading, leaving it and
  # everything after it in `argv`.
  def order! argv = ARGV, into: nil, &block
    until argv.empty?
      held = argv.first
      if held == "--"
        argv.shift
        break
      end
      unless held.start_with?("-") && held != "-"
        break unless block
        block.call(argv.shift)
        next
      end
      take_option argv.shift, argv, into
    end
    argv
  end

  def order argv = ARGV, into: nil, &block
    order! argv.dup, into: into, &block
  end

  # Every option read wherever it stands, with the arguments that are not
  # options left in `argv` in the order they came.
  def permute! argv = ARGV, into: nil
    rest = []
    order!(argv, into: into) { |held| rest << held }
    argv.replace rest + argv
    argv
  end

  def permute argv = ARGV, into: nil
    permute! argv.dup, into: into
  end

  alias_method :parse!, :permute!
  alias_method :parse, :permute

  def to_s
    lines = [@banner || "Usage: #{@program_name} [options]"]
    @switches.each do |switch|
      next lines << "#{@summary_indent}#{switch.description}" if switch.long.nil? && switch.short.nil?
      names = []
      names << "-#{switch.short}" if switch.short
      names << "--#{switch.long.to_s.gsub "_", "-"}#{switch.argument ? " #{switch.argument}" : ""}" if switch.long
      lines << "#{@summary_indent}#{names.join ", "}  #{switch.description}"
    end
    "#{lines.join "\n"}\n"
  end

  alias_method :help, :to_s

  def take_option held, argv, into
    return take_long_option(held, argv, into) if held.start_with?("--")

    letters = held[1..-1].to_s
    until letters.empty?
      letter = letters[0]
      letters = letters[1..-1].to_s
      switch = @switches.find { |held_switch| held_switch.short == letter }
      raise InvalidOption, "invalid option: -#{letter}" if switch.nil?
      if switch.argument.nil?
        record_option switch, true, into
        next
      end
      value = letters.empty? ? argv.shift : letters
      raise MissingArgument, "missing argument: -#{letter}" if value.nil?
      record_option switch, value, into
      break
    end
  end
  private :take_option

  def take_long_option held, argv, into
    name, given = held[2..-1].to_s.split("=", 2)
    negated = false
    if name.start_with? "no-"
      negated = true
      name = name["no-".length..-1]
    end
    wanted = name.gsub "-", "_"
    switch = @switches.find { |held_switch| held_switch.long == wanted }
    raise InvalidOption, "invalid option: #{held}" if switch.nil?
    value = if switch.argument.nil?
              !negated
            else
              given = argv.shift if given.nil?
              raise MissingArgument, "missing argument: #{held}" if given.nil?
              given
            end
    record_option switch, value, into
  end
  private :take_long_option

  def record_option switch, value, into
    into[switch.key] = value unless into.nil?
    switch.handler.call value if switch.handler
    value
  end
  private :record_option
end
