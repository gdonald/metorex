# Reading and writing JSON, the interchange format of RFC 8259.
#
# The parser reads a number by finding where it ends rather than by walking it
# one character at a time, so a document carrying a very long number is read in
# the time a short one takes.
module JSON
  class JSONError < StandardError; end
  class ParserError < JSONError; end
  class GeneratorError < JSONError; end
  class NestingError < ParserError; end

  # How deep a document may nest before the parser refuses it, which is what
  # holds a document built to exhaust the stack.
  MAX_NESTING = 100

  # The characters that can stand after a number.
  NUMBER_ENDS = [",", "]", "}", " ", "\t", "\n", "\r"].freeze

  class << self
    # What a JSON document stands for. `symbolize_names` names each key with a
    # Symbol rather than a String.
    def parse(source, opts = {})
      Parser.new(source, opts).parse
    end

    # The same, reading a document that holds a bare value at its top level.
    def parse!(source, opts = {})
      parse source, opts
    end

    # The JSON text an object is written as.
    def generate(object, opts = {})
      Generator.new(opts).generate object
    end

    alias dump generate

    # The same, laid out over several lines with each nesting indented.
    def pretty_generate(object, opts = {})
      Generator.new({ indent: "  ", space: " ", object_nl: "\n", array_nl: "\n" }.merge(opts))
        .generate object
    end

    # Read a document from anything that answers `read`, or from a String.
    def load(source, proc = nil, opts = {})
      source = source.read if source.respond_to? :read
      held = parse source, opts
      walk_loaded(held, proc) if proc
      held
    end

    private

    def walk_loaded(held, proc)
      case held
      when Array then held.each { |value| walk_loaded value, proc }
      when Hash then held.each { |_key, value| walk_loaded value, proc }
      end
      proc.call held
    end
  end

  # Reads one document.
  class Parser
    def initialize(source, opts = {})
      @source = source.to_s
      @at = 0
      @symbolize = opts[:symbolize_names] ? true : false
      @max_nesting = opts.key?(:max_nesting) ? opts[:max_nesting] : MAX_NESTING
      @depth = 0
    end

    def parse
      skip_space
      held = parse_value
      skip_space
      refuse "unexpected token at #{rest.inspect}" if @at < @source.length
      held
    end

    private

    def rest
      @source[@at, 20].to_s
    end

    def refuse(message)
      raise ParserError, message
    end

    def skip_space
      while @at < @source.length && " \t\n\r".include?(@source[@at])
        @at += 1
      end
    end

    def parse_value
      refuse "unexpected end of input" if @at >= @source.length
      case @source[@at]
      when "{" then parse_object
      when "[" then parse_array
      when '"' then parse_string
      when "t" then parse_word "true", true
      when "f" then parse_word "false", false
      when "n" then parse_word "null", nil
      else parse_number
      end
    end

    def parse_word(spelling, value)
      refuse "unexpected token at #{rest.inspect}" unless @source[@at, spelling.length] == spelling
      @at += spelling.length
      value
    end

    def deeper
      @depth += 1
      if @max_nesting && @max_nesting > 0 && @depth > @max_nesting
        raise NestingError, "nesting of #{@depth} is too deep"
      end
      held = yield
      @depth -= 1
      held
    end

    def parse_array
      deeper do
        @at += 1
        held = []
        skip_space
        if @source[@at] == "]"
          @at += 1
          next held
        end
        loop do
          skip_space
          held.push parse_value
          skip_space
          case @source[@at]
          when "," then @at += 1
          when "]"
            @at += 1
            break
          else refuse "expected ',' or ']' at #{rest.inspect}"
          end
        end
        held
      end
    end

    def parse_object
      deeper do
        @at += 1
        held = {}
        skip_space
        if @source[@at] == "}"
          @at += 1
          next held
        end
        loop do
          skip_space
          refuse "expected a string key at #{rest.inspect}" unless @source[@at] == '"'
          key = parse_string
          key = key.to_sym if @symbolize
          skip_space
          refuse "expected ':' at #{rest.inspect}" unless @source[@at] == ":"
          @at += 1
          skip_space
          held[key] = parse_value
          skip_space
          case @source[@at]
          when "," then @at += 1
          when "}"
            @at += 1
            break
          else refuse "expected ',' or '}' at #{rest.inspect}"
          end
        end
        held
      end
    end

    def parse_string
      @at += 1
      held = +""
      loop do
        stop = @source.index('"', @at)
        refuse "unterminated string" if stop.nil?
        piece = @source[@at...stop]
        # A quote with a backslash in front of it stands in the text rather
        # than closing it, and a backslash before that escapes the backslash.
        slashes = 0
        slashes += 1 while piece[piece.length - 1 - slashes] == "\\"
        held << piece
        @at = stop + 1
        if slashes.odd?
          held << '"'
          next
        end
        break
      end
      unescaped held
    end

    def unescaped(text)
      return text unless text.include? "\\"
      out = +""
      at = 0
      while at < text.length
        letter = text[at]
        if letter != "\\"
          out << letter
          at += 1
          next
        end
        escape = text[at + 1]
        at += 2
        case escape
        when '"' then out << '"'
        when "\\" then out << "\\"
        when "/" then out << "/"
        when "b" then out << "\b"
        when "f" then out << "\f"
        when "n" then out << "\n"
        when "r" then out << "\r"
        when "t" then out << "\t"
        when "u"
          digits = text[at, 4]
          refuse "incomplete unicode escape" if digits.nil? || digits.length < 4
          at += 4
          out << digits.to_i(16).chr("UTF-8")
        else refuse "unknown escape #{escape.inspect}"
        end
      end
      out
    end

    # A number runs to the first character that cannot be part of one. Looking
    # for where it ends reads a long number in one step rather than in as many
    # steps as it has digits.
    def parse_number
      stop = @source.length
      NUMBER_ENDS.each do |letter|
        found = @source.index(letter, @at)
        stop = found if found && found < stop
      end
      spelling = @source[@at...stop]
      refuse "unexpected token at #{rest.inspect}" if spelling.nil? || spelling.empty?
      @at = stop
      if spelling.match?(/\A-?(?:0|[1-9]\d*)\z/)
        spelling.to_i
      elsif spelling.match?(/\A-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][-+]?\d+)?\z/)
        spelling.to_f
      elsif spelling.match?(/\A-?\d/)
        # A number too long for the pattern to walk is still a number, and
        # reading it as a Float is what Ruby does with it.
        spelling.to_f
      else
        refuse "unexpected token at #{spelling.inspect}"
      end
    end
  end

  # Writes one document.
  class Generator
    ESCAPES = {
      '"' => '\\"',
      "\\" => "\\\\",
      "\b" => "\\b",
      "\f" => "\\f",
      "\n" => "\\n",
      "\r" => "\\r",
      "\t" => "\\t",
    }.freeze

    def initialize(opts = {})
      @indent = opts[:indent].to_s
      @space = opts[:space].to_s
      @object_nl = opts[:object_nl].to_s
      @array_nl = opts[:array_nl].to_s
    end

    def generate(object, depth = 0)
      case object
      when nil then "null"
      when true then "true"
      when false then "false"
      when String then quoted(object)
      when Symbol then quoted(object.to_s)
      when Integer then object.to_s
      when Float
        raise GeneratorError, "#{object} not allowed in JSON" if object.nan? || object.infinite?
        object.to_s
      when Array then generate_array(object, depth)
      when Hash then generate_object(object, depth)
      else
        return generate(object.to_json_raw_object, depth) if object.respond_to? :to_json_raw_object
        quoted(object.to_s)
      end
    end

    private

    def margin(depth)
      @indent.empty? ? "" : @indent * depth
    end

    def generate_array(held, depth)
      return "[]" if held.empty?
      inside = held.map { |value| margin(depth + 1) + generate(value, depth + 1) }
      "[#{@array_nl}#{inside.join(",#{@array_nl}")}#{@array_nl}#{margin(depth)}]"
    end

    def generate_object(held, depth)
      return "{}" if held.empty?
      inside = held.map do |key, value|
        "#{margin(depth + 1)}#{quoted(key.to_s)}:#{@space}#{generate(value, depth + 1)}"
      end
      "{#{@object_nl}#{inside.join(",#{@object_nl}")}#{@object_nl}#{margin(depth)}}"
    end

    def quoted(text)
      out = +'"'
      text.each_char do |letter|
        escaped = ESCAPES[letter]
        if escaped
          out << escaped
        elsif letter.ord < 0x20
          out << format("\\u%04x", letter.ord)
        else
          out << letter
        end
      end
      out << '"'
      out
    end
  end
end

# The JSON text an object stands for, which every object answers.
class Object
  def to_json(*)
    JSON.generate self
  end
end
