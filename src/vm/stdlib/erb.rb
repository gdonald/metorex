# Templates with Ruby in them. A template is compiled into Ruby source that
# builds the answer a piece at a time, and running that source against a
# binding gives the text the template stands for.

class ERB
  VERSION = "4.0.2"

  # Escaping the two things a template most often puts into a page.
  module Util
    def html_escape(text)
      held = text.to_s
      held = held.gsub "&", "&amp;"
      held = held.gsub "<", "&lt;"
      held = held.gsub ">", "&gt;"
      held = held.gsub "\"", "&quot;"
      held.gsub "'", "&#39;"
    end

    alias_method :h, :html_escape

    def url_encode(text)
      text.to_s.each_char.map { |held| ERB::Util.encoded_character(held) }.join
    end

    alias_method :u, :url_encode

    # A character stands for itself when a URL may carry it, and for its
    # bytes written in hex when it may not.
    def self.encoded_character(held)
      return held if held =~ /\A[a-zA-Z0-9_\-.~]\z/
      held.each_byte.map { |byte| "%%%02X" % byte }.join
    end

    module_function :html_escape, :h, :url_encode, :u
  end

  include Util

  attr_accessor :filename
  attr_accessor :lineno
  attr_reader :src
  attr_reader :encoding

  def initialize(template, safe_level = nil, legacy_trim_mode = nil,
                 legacy_eoutvar = nil, trim_mode: nil, eoutvar: "_erbout")
    @filename = nil
    @lineno = 0
    @encoding = "UTF-8"
    unless safe_level.nil? && legacy_trim_mode.nil? && legacy_eoutvar.nil?
      warn "warning: Passing safe_level with the 2nd argument of ERB.new is " \
           "deprecated. Do not use it, and specify other arguments as keyword " \
           "arguments."
    end
    mode = trim_mode.nil? ? legacy_trim_mode : trim_mode
    known = ["0", "1", "2", "%", "<>", ">", "-", "%<>", "%>", "%-"]
    unless mode.nil? || known.include?(mode.to_s)
      warn "Invalid ERB trim mode: #{mode.inspect} (trim_mode: nil, %, <>, >, -, %<>, %>, %-)"
      mode = nil
    end
    name = legacy_eoutvar.nil? ? eoutvar : legacy_eoutvar
    @src = ERB.compile template.to_s, mode, name
  end

  # The text the template stands for, read against the given binding. The
  # file the template came from is named so an error in it points there.
  def result(held = nil)
    eval @src, held.nil? ? ERB.fresh_binding : held, named_source, 0
  end

  # A scope of its own for each rendering, so a local a template names is
  # gone by the time the next one runs.
  def self.fresh_binding
    TOPLEVEL_BINDING.dup
  end

  # What a backtrace calls the compiled source.
  def named_source
    @filename.nil? ? "(erb)" : @filename
  end

  private :named_source

  # The same, with the names in the Hash bound for the template to see.
  def result_with_hash(names)
    assignments = names.map { |key, _| "#{key} = __erb_names__[:#{key}];" }.join
    __erb_names__ = names
    eval "#{assignments}#{@src}", binding
  end

  def run(held = nil)
    print result(held)
  end

  # Write the template as a method on the given module, so rendering it is a
  # call rather than a compile each time.
  def def_method(target, name, filename = nil)
    named = filename.nil? ? named_source : filename
    target.module_eval "def #{name}\n#{@src}\nend\n", named
  end

  # The same, on a module of its own.
  def def_module(name = "erb")
    built = Module.new
    def_method built, name
    built
  end

  # The same, on a subclass of the class given.
  def def_class(superclass = Object, name = "result")
    built = Class.new superclass
    def_method built, name
    built
  end

  # Writing a template out as a method of the module extending this one, so
  # rendering it reads as an ordinary call.
  module DefMethod
    # `def_erb_method("render()", file_or_erb)` writes the template as a
    # method of this module. A String names a file holding the template, and
    # an ERB stands for one already read.
    def def_erb_method(name, held)
      if held.is_a? ERB
        held.def_method(self, name, held.filename)
        return
      end
      compiled = ERB.new(File.read(held.to_s))
      compiled.filename = held.to_s
      compiled.def_method(self, name, held.to_s)
    end
  end

  # Compile a template into Ruby source. The answer is built with `+` rather
  # than in place, so the source runs wherever a String does.
  def self.compile(template, trim_mode, name)
    trims = trim_mode.to_s
    template = ERB.expand_percent_lines(template) if trims.start_with? "%"
    # The compiled source opens with the encoding it is written in, the
    # way MRI's does, so a line in the template and a line in the source
    # carry the same number.
    pieces = ["#coding:UTF-8\n#{name} = +'';"]
    at = 0
    while at < template.length
      opening = template.index "<%", at
      if opening.nil?
        pieces.push ERB.literal_piece(template[at..-1], name)
        break
      end
      literal = template[at, opening - at]
      closing = ERB.closing_index template, opening
      if closing.nil?
        pieces.push ERB.literal_piece(template[at..-1], name)
        break
      end
      body = template[opening + 2, closing - opening - 2]
      literal = ERB.without_line_indent(literal) if ERB.trims_indent?(body, trims)
      pieces.push ERB.literal_piece(literal, name)
      at = closing + 2
      if ERB.trims_newline?(body, trims, template, opening) && template[at] == "\n"
        at = at + 1
      end
      pieces.push ERB.tag_piece(ERB.tag_body(body, trims), name)
    end
    pieces.push "\n#{name}"
    pieces.join
  end

  # Where the tag opened at `opening` closes, skipping a `%>` written inside a
  # string in the tag's own code.
  def self.closing_index(template, opening)
    template.index "%>", opening
  end

  # A line opening with `%` is code, and one opening with `%%` is a line that
  # starts with a single `%`.
  def self.expand_percent_lines(template)
    ending = template.end_with? "\n"
    lines = template.split "\n", -1
    lines.pop if ending
    built = +""
    lines.each_with_index do |line, index|
      last = index == lines.length - 1
      if line.start_with? "%%"
        built << line[1..-1]
        built << "\n" unless last && !ending
      elsif line.start_with? "%"
        # A line of code is the whole line, so the ending belongs to the tag
        # rather than to the answer.
        built << "<%#{line[1..-1]}%>"
      else
        built << line
        built << "\n" unless last && !ending
      end
    end
    built
  end

  # The code a tag holds, with the `-` markers the explicit trim mode reads
  # taken off.
  def self.tag_body(body, trims)
    held = body
    held = held[1..-1] if ERB.opens_with_trim?(held, trims)
    held = held[0..-2] if trims.include?("-") && held.end_with?("-")
    held
  end

  # Whether a tag opens with the `-` that trims the spaces before it. A `-`
  # followed by `=` is code rather than a marker, which is why `<%-=` is not
  # a tag Ruby reads.
  def self.opens_with_trim?(body, trims)
    trims.include?("-") && body.start_with?("-") && body[1] != "="
  end

  # Whether the spaces between the start of the line and the tag are dropped.
  def self.trims_indent?(body, trims)
    ERB.opens_with_trim? body, trims
  end

  # A run of plain text, written as a piece to add.
  def self.literal_piece(text, name)
    return "" if text.nil? || text.empty?
    "#{name} = #{name} + #{text.dump};"
  end

  # The same text with the spaces at the start of its last line taken off.
  def self.without_line_indent(text)
    at = text.rindex "\n"
    head = at.nil? ? "" : text[0..at]
    tail = at.nil? ? text : text[(at + 1)..-1]
    tail.strip.empty? ? head : text
  end

  # One `<% %>` tag: `=` prints what it names, `#` is a comment, and anything
  # else is code that runs for what it does.
  def self.tag_piece(body, name)
    if body.start_with? "="
      "#{name} = #{name} + (#{body[1..-1]}).to_s;"
    elsif body.start_with? "#"
      "\n"
    else
      "#{body}\n"
    end
  end

  # Whether the newline after a tag is dropped, which only a trim mode asks
  # for. Left alone, a tag on a line of its own leaves that line's ending
  # behind in the answer.
  def self.trims_newline?(body, trims, template, opening)
    return true if trims.include?("-") && body.end_with?("-")
    return false if trims.include? "-"
    return ERB.opens_a_line?(template, opening) if trims.include? "<>"
    ["1", "2", ">", "%>"].include? trims
  end

  # Whether the tag sits at the start of its line, which is what the `<>`
  # trim mode asks about before dropping the newline after it.
  def self.opens_a_line?(template, opening)
    opening == 0 || template[opening - 1] == "\n"
  end
end
