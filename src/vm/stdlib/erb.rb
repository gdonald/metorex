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
    eval @src, held.nil? ? TOPLEVEL_BINDING : held, named_source, 0
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

  # Compile a template into Ruby source. The answer is built with `+` rather
  # than in place, so the source runs wherever a String does.
  def self.compile(template, trim_mode, name)
    pieces = ["#{name} = +'';"]
    at = 0
    trims = trim_mode.to_s
    while at < template.length
      opening = template.index "<%", at
      if opening.nil?
        pieces.push ERB.literal_piece(template[at..-1], name)
        break
      end
      pieces.push ERB.literal_piece(template[at, opening - at], name)
      closing = template.index "%>", opening
      if closing.nil?
        pieces.push ERB.literal_piece(template[opening..-1], name)
        break
      end
      body = template[opening + 2, closing - opening - 2]
      at = closing + 2
      at = at + 1 if ERB.trims_newline?(body, trims) && template[at] == "\n"
      pieces.push ERB.tag_piece(body, name)
    end
    pieces.push "\n#{name}"
    pieces.join
  end

  # A run of plain text, written as a piece to add.
  def self.literal_piece(text, name)
    return "" if text.nil? || text.empty?
    "#{name} = #{name} + #{text.dump};"
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
  def self.trims_newline?(body, trims)
    return true if body.end_with? "-"
    return false if body.start_with? "="
    ["1", ">", "<>", "%>", "%<>", "-", "%-"].include? trims
  end
end
