pub(super) const SOURCE: &str = r##"
class MatchData
  def initialize(string, regexp, begins, ends, names)
    # The subject is kept as a frozen copy, so a later change to the string
    # that was matched leaves what the match reports alone.
    @string = string.dup.freeze
    @regexp = regexp
    @begins = begins
    @ends = ends
    @names = names
  end

  def string
    @string
  end

  def regexp
    @regexp
  end

  def size
    @begins.size
  end

  def length
    @begins.size
  end

  # The index a subscript names: a number counts from the front, and a Symbol
  # or String names one of the pattern's groups.
  def group_index(key)
    if key.is_a?(Symbol) || key.is_a?(String)
      name = key.to_s
      index = @names[name]
      raise IndexError, "undefined group name reference: #{name}" if index.nil?
      return index
    end
    unless key.is_a?(Integer)
      unless key.respond_to?(:to_int)
        raise TypeError, "no implicit conversion of #{key.class} into Integer"
      end
      key = key.to_int
      unless key.is_a?(Integer)
        raise TypeError, "can't convert #{key.class} to Integer"
      end
    end
    index = key < 0 ? key + @begins.size : key
    if index < 0 || index >= @begins.size
      raise IndexError, "index #{key} out of matches"
    end
    index
  end
  private :group_index

  def group_text(index)
    index = index + @begins.size if index < 0
    return nil if index < 0 || index >= @begins.size
    return nil if @begins[index].nil?
    @string[@begins[index], @ends[index] - @begins[index]]
  end
  private :group_text

  def [](*keys)
    if keys.size == 2 && keys[0].is_a?(Integer) && keys[1].is_a?(Integer)
      return to_a[keys[0], keys[1]]
    end
    key = keys[0]
    return to_a[key] if key.is_a?(Range)
    if key.is_a?(Integer)
      # A negative index counts back over the groups the pattern named, so it
      # never reaches the whole match at index 0.
      if key < 0
        index = key + @begins.size
        return nil if index < 1
      else
        index = key
      end
      return nil if index >= @begins.size
      return group_text(index)
    end
    group_text(group_index(key))
  end

  def to_a
    collected = []
    index = 0
    while index < @begins.size
      collected.push(group_text(index))
      index += 1
    end
    collected
  end

  def captures
    to_a[1, @begins.size - 1]
  end

  def named_captures(symbolize_names: false)
    collected = {}
    @names.each do |name, index|
      key = symbolize_names ? name.to_sym : name
      collected[key] = group_text(index)
    end
    collected
  end

  def names
    @names.keys
  end

  # A Range subscript picks a run of groups, the way it does on an Array.
  def values_at(*keys)
    collected = []
    keys.each do |key|
      if key.is_a?(Range)
        first = key.begin.nil? ? 0 : key.begin
        first += @begins.size if first < 0
        raise RangeError, "#{key} out of range" if first < 0 || first > @begins.size
        last = key.end.nil? ? @begins.size - 1 : key.end
        last += @begins.size if key.end && key.end < 0
        last -= 1 if key.exclude_end?
        index = first
        while index <= last
          collected.push(self[index])
          index += 1
        end
      elsif key.is_a?(Integer)
        collected.push(self[key])
      else
        collected.push(group_text(group_index(key)))
      end
    end
    collected
  end

  # `begin`, `end`, and `offset` count only forward, so a negative subscript
  # is out of bounds rather than a count from the end.
  def positive_group_index(key)
    if key.is_a?(Integer) && key < 0
      raise IndexError, "index #{key} out of matches"
    end
    group_index(key)
  end
  private :positive_group_index

  def begin(key)
    @begins[positive_group_index(key)]
  end

  def end(key)
    @ends[positive_group_index(key)]
  end

  def offset(key)
    index = positive_group_index(key)
    [@begins[index], @ends[index]]
  end

  # Where a group sat counted in bytes rather than in characters, which is
  # what a program reading the subject byte by byte needs.
  def byteoffset(key)
    index = positive_group_index(key)
    [__bytes_before__(@begins[index]), __bytes_before__(@ends[index])]
  end

  def bytebegin(key)
    __bytes_before__ @begins[positive_group_index(key)]
  end

  def byteend(key)
    __bytes_before__ @ends[positive_group_index(key)]
  end

  # How many bytes of the subject sit before a place counted in characters.
  def __bytes_before__(counted)
    return nil if counted.nil?
    @string[0, counted].bytesize
  end
  private :__bytes_before__

  # `match` and `match_length` answer one group's text and its length.
  def match(key)
    group_text(positive_group_index(key))
  end

  def match_length(key)
    found = match(key)
    found.nil? ? nil : found.size
  end

  def pre_match
    @string[0, @begins[0]]
  end

  def post_match
    @string[@ends[0], @string.size - @ends[0]]
  end

  def to_s
    group_text(0)
  end

  def deconstruct
    captures
  end

  # Only an Array names which groups to read, and each name in it must be a
  # Symbol. A key the pattern does not name ends the reading, and more keys
  # than there are named groups reads none at all.
  def deconstruct_keys(keys)
    return named_captures.transform_keys { |name| name.to_sym } if keys.nil?
    unless keys.is_a?(Array)
      raise TypeError, "wrong argument type #{keys.class} (expected Array)"
    end
    return {} if keys.size > @names.size
    collected = {}
    keys.each do |key|
      unless key.is_a?(Symbol)
        raise TypeError, "wrong argument type #{key.class} (expected Symbol)"
      end
      index = @names[key.to_s]
      return collected if index.nil?
      collected[key] = group_text(index)
    end
    collected
  end

  def ==(other)
    return false unless other.is_a?(MatchData)
    string == other.string && regexp == other.regexp && to_a == other.to_a
  end

  def eql?(other)
    self == other
  end

  def hash
    [string, to_a].hash
  end

  def inspect
    parts = ["#<MatchData"]
    parts.push(group_text(0).inspect)
    index = 1
    reversed = {}
    @names.each { |name, at| reversed[at] = name }
    while index < @begins.size
      label = reversed.has_key?(index) ? reversed[index] : index.to_s
      parts.push("#{label}:#{group_text(index).inspect}")
      index += 1
    end
    parts.join(" ") + ">"
  end
end

class Complex
  # The imaginary unit, which every other Complex is measured against.
  I = Complex(0, 1)

  # A Complex names no point on the number line, so it answers neither of the
  # questions a real number does.
  undef_method :positive?
  undef_method :negative?
end

class IO
  module WaitReadable
  end

  module WaitWritable
  end

  class EAGAINWaitReadable < Errno::EAGAIN
    include WaitReadable
  end

  class EAGAINWaitWritable < Errno::EAGAIN
    include WaitWritable
  end

  EWOULDBLOCKWaitReadable = EAGAINWaitReadable
  EWOULDBLOCKWaitWritable = EAGAINWaitWritable

  class EINPROGRESSWaitReadable < Errno::EINPROGRESS
    include WaitReadable
  end

  class EINPROGRESSWaitWritable < Errno::EINPROGRESS
    include WaitWritable
  end

  # What a read or a write raises when the stream's own limit on how long it
  # may take runs out.
  class TimeoutError < IOError
  end
end

class StopIteration
  attr_accessor :result
end

# The object a program runs against at the top level. Ruby calls it `main`,
# and that is what it says of itself.
class << __main__
  def to_s
    "main"
  end

  def inspect
    "main"
  end

  private

  # A module included at the top level is included into Object.
  def include(*modules)
    Object.include(*modules)
  end
end

class Thread
  class Backtrace
    # How many frames under the top one a report writes out, which
    # `--backtrace-limit` settles and which is -1 when it was not written.
    def self.limit
      $__backtrace_limit__.nil? ? -1 : $__backtrace_limit__
    end

    class Location
      attr_reader :path, :lineno, :label, :absolute_path

      # The label without what it was reached through: a block's label names
      # the method holding it, and a method's label names the method alone
      # rather than the class or module it was found on.
      def base_label
        return @label if @label.nil?
        held = @label.start_with?("block ") ? @label.split(" in ", 2).last : @label
        return held if held.start_with?("<")
        held.split(/[.#]/).last
      end

      def to_s
        return "#{@path}:#{@lineno}" if @label.nil? || @label.empty?
        "#{@path}:#{@lineno}:in '#{@label}'"
      end

      def inspect
        to_s.inspect
      end
    end
  end
end

# An in-memory IO. `StringIO.new` starts from the string it is given, and
# everything written is appended to it.
# A string read and written the way a file is: it holds a position, a line
# count, and the two sides of a stream that may be closed apart."##;
