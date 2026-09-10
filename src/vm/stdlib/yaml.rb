# Reading and writing the subset of YAML that Ruby objects round-trip
# through: scalars, sequences, mappings, and the tags that name a Ruby class,
# module, symbol, or object.

require 'date'

module Psych
  VERSION = "5.1.2"

  class SyntaxError < StandardError
  end

  class DisallowedClass < StandardError
  end

  # The tree a parse answers, which `to_ruby` turns back into objects.
  module Nodes
    class Node
      attr_accessor :children

      def initialize children = []
        @children = children
      end
    end

    class Document < Node
      def initialize root = nil
        super []
        @root = root
      end

      attr_accessor :root

      def to_ruby
        @root
      end

      def value
        @root
      end
    end

    class Stream < Node
      def to_ruby
        @children.map { |held| held.to_ruby }
      end
    end
  end

  def self.to_s
    "Psych"
  end

  # One document read from the text, and nil when the text holds none.
  def self.load source, **options
    documents = parse_documents read_source(source)
    return nil if documents.empty?
    documents.first
  end

  class << self
    alias_method :unsafe_load, :load
    alias_method :safe_load, :load
  end

  def self.load_file path, **options
    load File.read(path)
  end

  def self.unsafe_load_file path, **options
    load_file path
  end

  # Every document in the text, handed to a block one at a time when one is
  # given and answered as an Array otherwise.
  def self.load_stream source, **options
    documents = parse_documents read_source(source)
    return documents unless block_given?
    documents.each { |held| yield held }
    nil
  end

  class << self
    alias_method :each_document, :load_stream
    alias_method :parse_stream, :load_stream
  end

  # The first document as a node tree, and false when the text holds none.
  def self.parse source, **options
    documents = parse_documents read_source(source)
    return false if documents.empty?
    Nodes::Document.new documents.first
  end

  def self.parse_file path, **options
    parse File.read(path)
  end

  # Write an object out, to the stream given or as the text answered.
  def self.dump held, io = nil, **options
    text = "#{document_for held}"
    return text if io.nil?
    io.write text
    io
  end

  def self.dump_stream *objects
    objects.map { |held| document_for held }.join
  end

  # The text an object is written as, with the `---` that opens a document.
  def self.document_for held
    body = emit held, 0
    return "--- #{body}\n" if body.is_a? String
    "---\n#{body.join "\n"}\n"
  end

  def self.read_source source
    return source.read if source.respond_to? :read
    source.to_s
  end

  # Split the text into documents and read each one.
  def self.parse_documents text
    documents = []
    current = []
    started = false
    text.each_line do |line|
      stripped = line.chomp
      if stripped == "---" || stripped.start_with?("--- ")
        documents << current if started
        current = []
        started = true
        rest = stripped == "---" ? "" : stripped[4..-1].to_s
        current << rest unless rest.strip.empty?
        next
      end
      current << stripped
      started = true
    end
    documents << current if started
    documents.map { |lines| read_document lines }
  end

  # The value a document's lines stand for.
  def self.read_document lines
    kept = lines.reject { |line| line.strip.empty? || line.strip.start_with?("#") }
    return nil if kept.empty?
    # A tag on the opening line names what the rest of the document builds.
    first = kept.first.strip
    if first.start_with?("!ruby/object:") && kept.length > 1
      fields = read_block kept[1..-1], indent_of(kept[1])
      return build_object first["!ruby/object:".length..-1].split(" ").first, fields
    end
    read_block kept, 0
  end

  # A block of lines at one indentation, read as a sequence, a mapping, or a
  # scalar of its own.
  def self.read_block lines, indent
    return nil if lines.empty?
    first = lines.first
    if first.lstrip.start_with?("- ") || first.strip == "-"
      return read_sequence lines
    end
    return read_mapping lines if mapping_line?(first) || first.lstrip.start_with?("? ")
    return read_scalar first.strip if lines.length == 1
    # More than one line with no marker of its own is a scalar folded across
    # them, which is what a plain multi-line value amounts to.
    read_scalar lines.map { |line| line.strip }.join(" ")
  end

  def self.mapping_line? line
    held = line.lstrip
    return false if held.start_with?("- ")
    return true if held.end_with?(":")
    !scan_key(held).nil?
  end

  # The key and the rest of a `key: value` line, and nil when the line names
  # no key at all.
  def self.scan_key line
    depth = 0
    index = 0
    quote = nil
    while index < line.length
      character = line[index]
      if quote
        quote = nil if character == quote
      elsif character == "'" || character == '"'
        quote = character
      elsif character == "[" || character == "{"
        depth += 1
      elsif character == "]" || character == "}"
        depth -= 1
      elsif character == ":" && depth == 0 &&
            (index + 1 == line.length || line[index + 1] == " ")
        return [line[0, index], line[(index + 1)..-1].to_s.strip]
      end
      index += 1
    end
    nil
  end

  def self.indent_of line
    line.length - line.lstrip.length
  end

  # Lines at a deeper indentation than the first, which belong to it.
  def self.take_nested lines, at
    taken = []
    while !lines.empty? && indent_of(lines.first) > at
      taken << lines.shift
    end
    taken
  end

  def self.read_sequence lines
    remaining = lines.dup
    items = []
    until remaining.empty?
      line = remaining.shift
      at = indent_of line
      held = line.lstrip
      rest = held == "-" ? "" : held[2..-1].to_s
      nested = take_nested remaining, at
      # `- - one` opens a sequence inside this one on the same line, so the
      # marker's own column is where the nested block starts.
      if rest.lstrip.start_with?("- ") || rest.strip == "-" || mapping_line?(rest)
        inner = [" " * (at + 2) + rest] + nested
        items << read_block(inner, at + 2)
      elsif rest.strip.empty?
        items << (nested.empty? ? nil : read_block(nested, indent_of(nested.first)))
      elsif nested.empty?
        items << read_scalar(rest.strip)
      else
        items << read_block([" " * (at + 2) + rest] + nested, at + 2)
      end
    end
    items
  end

  def self.read_mapping lines
    remaining = lines.dup
    pairs = {}
    until remaining.empty?
      line = remaining.shift
      at = indent_of line
      held = line.lstrip
      # `? key` puts the key in a block of its own, with the value under a
      # `:` line that follows it.
      if held == "?" || held.start_with?("? ")
        rest = held == "?" ? "" : held[2..-1].to_s
        rest = "" if rest.strip.start_with?("#")
        key_lines = rest.strip.empty? ? [] : [" " * (at + 2) + rest]
        key_lines = key_lines + take_nested(remaining, at)
        key = read_block key_lines, at + 2
        value = nil
        if !remaining.empty? && (remaining.first.lstrip == ":" || remaining.first.lstrip.start_with?(": "))
          value_line = remaining.shift
          value_at = indent_of value_line
          value_rest = value_line.lstrip == ":" ? "" : value_line.lstrip[2..-1].to_s
          value_lines = value_rest.strip.empty? ? [] : [" " * (value_at + 2) + value_rest]
          value_lines = value_lines + take_nested(remaining, value_at)
          value = value_lines.empty? ? nil : read_block(value_lines, value_at + 2)
        end
        pairs[key] = value
        next
      end
      found = scan_key held
      raise Psych::SyntaxError, "did not find expected key while parsing a block mapping" if found.nil?
      name, rest = found
      nested = take_nested remaining, at
      key = read_scalar name.strip
      if rest.empty?
        pairs[key] = nested.empty? ? nil : read_block(nested, indent_of(nested.first))
      elsif nested.empty?
        pairs[key] = read_scalar rest
      else
        pairs[key] = read_block([" " * (at + 2) + rest] + nested, at + 2)
      end
    end
    pairs
  end

  # One value written on a single line.
  def self.read_scalar text
    held = text.strip
    held = held[0...held.index(" #")].to_s.strip if held.include?(" #") && !held.start_with?("'", '"')
    return nil if held.empty? || held == "~" || held == "null"
    return read_flow_sequence(held) if held.start_with?("[") && held.end_with?("]")
    return read_flow_mapping(held) if held.start_with?("{") && held.end_with?("}")
    return held[1..-2].to_s.gsub("''", "'") if held.length > 1 && held.start_with?("'") && held.end_with?("'")
    if held.length > 1 && held.start_with?('"') && held.end_with?('"')
      return held[1..-2].to_s.gsub('\\"', '"').gsub("\\n", "\n").gsub("\\t", "\t")
    end
    return true if held == "true"
    return false if held == "false"
    return held[1..-1].to_sym if held.start_with?(":") && held.length > 1
    return tagged_value(held) if held.start_with?("!")
    return held.to_i if /\A-?\d+\z/.match held
    return held.to_f if /\A-?\d+\.\d+\z/.match held
    if (matched = /\A(\d{4})-(\d{2})-(\d{2})\z/.match(held))
      return Date.new matched[1].to_i, matched[2].to_i, matched[3].to_i
    end
    if (matched = /\A(\d{4})-(\d{2})-(\d{2})[tT ](\d{2}):(\d{2}):(\d{2})(\.\d+)?\s*([+-]\d{2}:?\d{2}|Z)?\z/.match(held))
      return read_timestamp matched
    end
    held
  end

  # `[a, b, c]` and `{a: b}` written on one line.
  def self.read_flow_sequence text
    split_flow(text[1..-2].to_s).map { |piece| read_scalar piece }
  end

  def self.read_flow_mapping text
    pairs = {}
    split_flow(text[1..-2].to_s).each do |piece|
      found = scan_key piece.strip
      next if found.nil?
      pairs[read_scalar(found[0].strip)] = read_scalar(found[1])
    end
    pairs
  end

  # The comma-separated parts of a flow collection, keeping nested ones whole.
  def self.split_flow text
    parts = []
    depth = 0
    quote = nil
    held = ""
    text.each_char do |character|
      if quote
        quote = nil if character == quote
        held = held + character
        next
      end
      case character
      when "'", '"' then quote = character
      when "[", "{" then depth += 1
      when "]", "}" then depth -= 1
      when ","
        if depth == 0
          parts << held
          held = ""
          next
        end
      end
      held = held + character
    end
    parts << held unless held.strip.empty?
    parts.map { |piece| piece.strip }.reject { |piece| piece.empty? }
  end

  # A moment written the way a timestamp is, kept to the microsecond.
  def self.read_timestamp matched
    fraction = matched[7].to_s
    micro = fraction.empty? ? 0 : (fraction.to_f * 1_000_000).round
    zone = matched[8].to_s
    offset = 0
    unless zone.empty? || zone == "Z"
      sign = zone.start_with?("-") ? -1 : 1
      digits = zone.gsub(/[^0-9]/, "")
      offset = sign * (digits[0, 2].to_i * 3600 + digits[2, 2].to_i * 60)
    end
    held = Time.utc matched[1].to_i, matched[2].to_i, matched[3].to_i,
                    matched[4].to_i, matched[5].to_i, matched[6].to_i
    Time.at held.to_i - offset, micro
  end

  # `!ruby/class 'Name'` and the tags beside it name something the program
  # already holds.
  def self.tagged_value text
    name, rest = text.split " ", 2
    body = rest.to_s.strip
    body = body[1..-2].to_s if body.start_with?("'") && body.end_with?("'")
    case name
    when "!ruby/class", "!ruby/module" then Object.const_get body
    when "!ruby/symbol", "!ruby/sym" then body.to_sym
    else
      return build_object(name["!ruby/object:".length..-1], {}) if name.start_with?("!ruby/object:") && (body.empty? || body == "{}")
      body.empty? ? name : body
    end
  end

  # An object of the named class, built without running its constructor and
  # given the instance variables the mapping held.
  def self.build_object class_name, fields
    holder = Object.const_get class_name
    made = holder.allocate
    fields.each do |name, value|
      made.instance_variable_set "@#{name.to_s.sub(/\A:/, '')}", value
    end
    made
  end

  # The lines an object is written as: a String for a scalar that fits on the
  # `---` line, and an Array of lines for anything with parts.
  def self.emit held, depth
    pad = "  " * depth
    case held
    when nil then ""
    when true, false, Integer then held.to_s
    when Float
      next_held = held
      return ".nan" if next_held != next_held
      return ".inf" if next_held == Float::INFINITY
      return "-.inf" if next_held == -Float::INFINITY
      next_held.to_s
    when Symbol then ":#{held}"
    when Date then held.strftime("%Y-%m-%d")
    when String then emit_string(held)
    when Array
      return "[]" if held.empty?
      held.flat_map do |item|
        piece = emit item, depth + 1
        next ["#{pad}- #{piece}"] if piece.is_a? String
        next ["#{pad}- #{piece.lines.first}"] + piece.lines[1..-1].to_a if piece.is_a? OpenTagged
        # The first line of a nested block sits beside the marker, which is
        # what keeps `- a: b` on one line.
        ["#{pad}- #{piece.first.lstrip}"] + piece[1..-1].to_a
      end
    when Hash
      return "{}" if held.empty?
      held.flat_map do |key, value|
        name = emit key, 0
        piece = emit value, depth + 1
        piece.is_a?(String) ? ["#{pad}#{name}: #{piece}"] : ["#{pad}#{name}:"] + piece
      end
    when Class then "!ruby/class '#{held.name}'"
    when Module then "!ruby/module '#{held.name}'"
    when Regexp then "!ruby/regexp /#{held.source}/"
    when Time
      moment = held.utc
      nanos = format "%09d", (moment.nsec rescue moment.usec * 1000)
      "#{moment.strftime "%Y-%m-%d %H:%M:%S"}.#{nanos} Z"
    when Range then tagged_lines("!ruby/range", { "begin" => held.begin, "end" => held.end, "excl" => held.exclude_end? }, depth)
    when Exception
      tagged_lines "!ruby/exception:#{held.class.name}",
                   { "message" => held.message, "backtrace" => held.backtrace }, depth
    else
      # A Struct carries its parts under the names it was built with, which
      # is what `!ruby/struct` writes out.
      members = begin
        held.members
      rescue NoMethodError
        nil
      end
      if members
        fields = {}
        values = held.to_a
        members.each_with_index { |name, at| fields[name.to_s] = values[at] }
        named = held.class.name.to_s
        return tagged_lines(named.empty? ? "!ruby/struct" : "!ruby/struct:#{named}", fields, depth)
      end
      emit_object held, depth
    end
  end

  # An object written under its own tag, with the fields it carries listed
  # under it.
  def self.tagged_lines tag, fields, depth
    return OpenTagged.new([tag]) if fields.empty?
    lines = [tag]
    fields.each do |name, value|
      piece = emit value, depth + 1
      lines = lines + (piece.is_a?(String) ? ["#{name}: #{piece}"] : ["#{name}:"] + piece)
    end
    OpenTagged.new lines
  end

  def self.emit_object held, depth
    # The bookkeeping metorex keeps on an object of its own is not state the
    # program set, so it is left out.
    names = held.instance_variables.reject { |name| name.to_s.start_with? "@__" }
    tag = "!ruby/object:#{held.class.name}"
    return "#{tag} {}" if names.empty?
    lines = ["--- #{tag}"]
    fields = {}
    names.each do |name|
      fields[name.to_s.sub("@", "")] = held.instance_variable_get(name)
    end
    lines = [tag]
    fields.each do |name, value|
      piece = emit value, depth + 1
      lines = lines + (piece.is_a?(String) ? ["#{name}: #{piece}"] : ["#{name}:"] + piece)
    end
    OpenTagged.new lines
  end

  # An object's own tag sits on the `---` line with its fields under it, which
  # is neither a plain scalar nor a plain block.
  class OpenTagged
    def initialize(lines)
      @lines = lines
    end

    attr_reader :lines
  end

  def self.emit_string text
    return "''" if text.empty?
    return "'#{text.gsub "'", "''"}'" if quoting_needed? text
    text
  end

  def self.quoting_needed? text
    return true if text != text.strip
    return true if /\A-?\d/.match text
    return true if ["true", "false", "null", "~", "---"].include? text
    return true if text.start_with?(":", "-", "?", "&", "*", "!", "#", "[", "{", "'", '"')
    return true if text.include?(": ") || text.include?("\n")
    false
  end
end

module Psych
  # An object written with its own tag opens the document itself, so the
  # lines it answers are laid out rather than nested under a `---`.
  def self.document_for held
    body = emit held, 0
    if body.is_a? String
      # An empty collection is written on the `---` line and closed by a
      # blank one, which is what keeps it apart from the next document.
      return "--- #{body}\n\n" if body == "[]" || body == "{}"
      return "--- #{body}\n"
    end
    return "--- #{body.lines.first}\n#{body.lines[1..-1].join "\n"}\n" if body.is_a? OpenTagged
    "--- \n#{body.join "\n"}\n"
  end
end

YAML = Psych

class Object
  def to_yaml options = {}
    Psych.dump self
  end

  def psych_to_yaml options = {}
    to_yaml options
  end
end
