# Comma separated values, read into rows of fields and written back out.
class CSV
  include Enumerable

  class MalformedCSVError < RuntimeError
  end

  DEFAULT_COLUMN_SEPARATOR = ",".freeze
  DEFAULT_ROW_SEPARATOR = "\n".freeze
  QUOTE = "\"".freeze

  def self.parse(text, **options)
    Reader.new(options).read(text.to_s)
  end

  def self.parse_line(text, **options)
    rows = parse(text, **options)
    rows.empty? ? nil : rows[0]
  end

  def self.generate_line(row, **options)
    separator = options[:col_sep] || DEFAULT_COLUMN_SEPARATOR
    written = row.map { |field| quote_field(field, separator) }.join(separator)
    written + (options[:row_sep] || DEFAULT_ROW_SEPARATOR)
  end

  # A field as it is written into a row. One carrying the separator, a quote,
  # or a line ending is wrapped in quotes, with its own quotes doubled.
  def self.quote_field(field, separator)
    return "" if field.nil?
    written = field.to_s
    unwritable = written.include?(separator) || written.include?(QUOTE) ||
      written.include?("\n") || written.include?("\r")
    return written unless unwritable
    QUOTE + written.gsub(QUOTE, QUOTE + QUOTE) + QUOTE
  end

  def self.generate(text = "", **options, &block)
    writer = CSV.new(text.to_s, **options)
    block.call(writer)
    writer.string
  end

  def self.read(path, **options)
    parse(File.read(path), **options)
  end

  def self.readlines(path, **options)
    read(path, **options)
  end

  def self.foreach(path, **options, &block)
    rows = read(path, **options)
    return rows.each if block.nil?
    rows.each { |row| block.call(row) }
    nil
  end

  def self.open(path, mode = "r", **options, &block)
    made = CSV.new(File.read(path), **options)
    return made if block.nil?
    block.call(made)
  end

  def self.instance(text = "", **options)
    CSV.new(text, **options)
  end

  def initialize(data = "", **options)
    @options = options
    @col_sep = options[:col_sep] || DEFAULT_COLUMN_SEPARATOR
    @row_sep = options[:row_sep] || DEFAULT_ROW_SEPARATOR
    @liberal_parsing = options[:liberal_parsing] ? true : false
    @string = data.to_s
    @rows = nil
  end

  def col_sep
    @col_sep
  end

  def row_sep
    @row_sep
  end

  def liberal_parsing?
    @liberal_parsing
  end

  def string
    @string
  end

  def to_s
    @string
  end

  def <<(row)
    @string = @string + CSV.generate_line(row, **@options)
    @rows = nil
    self
  end

  def add_row(row)
    self << row
  end

  def puts(row)
    self << row
  end

  def read
    @rows = Reader.new(@options).read(@string) if @rows.nil?
    @rows
  end

  def readlines
    self.read
  end

  def shift
    rows = self.read
    return nil if rows.empty?
    rows.shift
  end

  def each(&block)
    return self.read.each if block.nil?
    self.read.each { |row| block.call(row) }
    self
  end

  def rewind
    @rows = nil
    self
  end

  def close
    self
  end

  # Text read one character at a time into the rows and fields it names.
  class Reader
    def initialize(options)
      @col_sep = options[:col_sep] || DEFAULT_COLUMN_SEPARATOR
      @liberal_parsing = options[:liberal_parsing] ? true : false
    end

    def read(text)
      @rows = []
      @row = []
      @field = nil
      @quoted = false
      @closed = false
      @line = 1
      index = 0
      while index < text.length
        index = self.take(text, index)
      end
      self.close_quoted_field
      self.end_row unless @field.nil? && @row.empty?
      @rows
    end

    # One character, read against what the field before it left open.
    def take(text, index)
      character = text[index]
      return self.take_quoted(text, index) if @quoted
      return self.open_quotes(index) if character == QUOTE && @field.nil?
      return self.take_stray_quote(character, index) if character == QUOTE
      if @closed && character != @col_sep && character != "\n" && character != "\r"
        return self.take_after_quotes(character, index)
      end
      return self.end_field(index) if character == @col_sep
      return index + 1 if character == "\r"
      return self.end_row(index) if character == "\n"
      @field = @field.nil? ? character : @field + character
      index + 1
    end

    def take_quoted(text, index)
      character = text[index]
      unless character == QUOTE
        @field = @field + character
        @line += 1 if character == "\n"
        return index + 1
      end
      if text[index + 1] == QUOTE
        @field = @field + QUOTE
        return index + 2
      end
      @quoted = false
      @closed = true
      index + 1
    end

    def open_quotes(index)
      @quoted = true
      @field = ""
      index + 1
    end

    # A quote inside a field that did not open with one, which is malformed
    # unless the reader was asked to take what it is given.
    def take_stray_quote(character, index)
      unless @liberal_parsing
        raise MalformedCSVError, "Illegal quoting in line #{@line}."
      end
      @field = @field + character
      index + 1
    end

    def take_after_quotes(character, index)
      unless @liberal_parsing
        raise MalformedCSVError,
              "Any value after quoted field isn't allowed in line #{@line}."
      end
      @field = @field + character
      index + 1
    end

    def close_quoted_field
      return unless @quoted
      raise MalformedCSVError, "Unclosed quoted field in line #{@line}."
    end

    def end_field(index)
      @row.push(@field)
      @field = nil
      @closed = false
      index + 1
    end

    def end_row(index = nil)
      @row.push(@field) unless @field.nil? && @row.empty?
      @rows.push(@row)
      @row = []
      @field = nil
      @closed = false
      @line += 1
      index.nil? ? nil : index + 1
    end
  end
end
