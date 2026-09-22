pub(super) const SOURCE: &str = r##"
class Encoding
  # The encoding the machine's locale names. It is read once, so a program
  # writing to the environment afterwards does not change it.
  def self.locale_charmap
    return @locale_charmap unless @locale_charmap.nil?
    # The C library names it, since one locale is called different things on
    # different platforms.
    held = Encoding.__charmap__.to_s
    return @locale_charmap = held unless held.empty?
    named = ENV["LC_ALL"] || ENV["LC_CTYPE"] || ENV["LANG"] || ""
    @locale_charmap = if named.empty? || named == "C" || named == "POSIX"
      "US-ASCII"
    elsif named.include?(".")
      named.split(".", 2)[1]
    else
      "UTF-8"
    end
  end

  # Every name an encoding answers to, its own and the aliases pointing at it.
  def self.name_list
    named = list.map { |held| held.name }
    aliases.each_key { |held| named.push(held) unless named.include?(held) }
    named
  end

  # The names this encoding answers to, its own first.
  def names
    held = [name]
    Encoding.aliases.each do |spelled, stands_for|
      held.push(spelled) if stands_for == name && !held.include?(spelled)
    end
    held
  end

  # The encoding the source of a program is read as. `-K` names it, and
  # without that flag it is UTF-8.
  def self.__source__
    @__source__ || __running_source__
  end

  def self.__source__= named
    @__source__ = named.is_a?(Encoding) ? named : Encoding.find(named)
  end

  # A conversion from one encoding to another, along with the flags saying
  # what to do with what the destination cannot spell.
  # What a conversion reports when the destination cannot spell a character.
  class UndefinedConversionError
    attr_reader :source_encoding
    attr_reader :destination_encoding
    attr_reader :error_char

    def initialize message = nil, source = nil, destination = nil, character = nil
      super message
      @source_encoding = source
      @destination_encoding = destination
      @error_char = character
    end

    def source_encoding_name
      @source_encoding.nil? ? nil : @source_encoding.name
    end

    def destination_encoding_name
      @destination_encoding.nil? ? nil : @destination_encoding.name
    end
  end

  # What a conversion reports when the source cannot read its own bytes.
  class InvalidByteSequenceError
    attr_reader :source_encoding
    attr_reader :destination_encoding
    attr_reader :error_bytes
    attr_reader :readagain_bytes

    def initialize message = nil, source = nil, destination = nil, wrong = nil, rest = nil, truncated = nil
      super message
      @source_encoding = source
      @destination_encoding = destination
      @error_bytes = wrong.nil? ? "" : wrong
      @readagain_bytes = rest.nil? ? "" : rest
      @truncated = truncated
    end

    def source_encoding_name
      @source_encoding.nil? ? nil : @source_encoding.name
    end

    def destination_encoding_name
      @destination_encoding.nil? ? nil : @destination_encoding.name
    end

    # A run cut off at the end of the text is incomplete rather than wrong.
    # An error built by hand says nothing either way.
    def incomplete_input?
      @truncated
    end
  end

  class Converter
    INVALID_MASK = 0x0f
    INVALID_REPLACE = 0x02
    UNDEF_MASK = 0xf0
    UNDEF_REPLACE = 0x20
    UNDEF_HEX_CHARREF = 0x30
    PARTIAL_INPUT = 0x10000
    AFTER_OUTPUT = 0x20000
    UNIVERSAL_NEWLINE_DECORATOR = 0x100
    CRLF_NEWLINE_DECORATOR = 0x1000
    CR_NEWLINE_DECORATOR = 0x2000
    XML_TEXT_DECORATOR = 0x4000
    XML_ATTR_CONTENT_DECORATOR = 0x8000
    XML_ATTR_QUOTE_DECORATOR = 0x10000

    def initialize source, destination, options = 0
      @source = Encoding::Converter.named source
      @destination = Encoding::Converter.named destination
      if @source == @destination
        raise Encoding::ConverterNotFoundError,
              "code converter not found (#{@source.name} to #{@destination.name})"
      end
      @options = options
      @convpath = Encoding::Converter.search_convpath @source, @destination, options
      @replacement = Encoding::Converter.replacement_for @destination, options
    end

    # What stands in for a character the destination cannot spell. UTF-8 has
    # a character of its own for that, and everything else uses a question
    # mark.
    def self.replacement_for destination, options
      standard = if destination == Encoding::UTF_8
        "\u{fffd}"
      else
        "?".force_encoding Encoding::US_ASCII
      end
      return standard unless options.is_a? Hash
      return standard unless options.key? :replace
      held = options[:replace]
      return standard if held.nil?
      return held if held.is_a? String
      unless held.respond_to? :to_str
        raise TypeError, "no implicit conversion of #{held.class} into String"
      end
      held = held.to_str
      unless held.is_a? String
        raise TypeError, "can't convert #{held.class} to String"
      end
      held
    end

    def source_encoding
      @source
    end

    def destination_encoding
      @destination
    end

    def convpath
      @convpath
    end

    def options
      @options.is_a?(Integer) ? @options : 0
    end

    def replacement
      @replacement
    end

    def replacement= held
      unless held.is_a? String
        raise TypeError, "no implicit conversion of #{held.class} into String"
      end
      # A replacement the destination cannot spell is refused, and the one
      # already in use stays.
      held.each_char do |character|
        next if spellable? character
        spelled = "U+" + character.ord.to_s(16).upcase.rjust(4, "0")
        from, to = stage_for :undefined
        raise Encoding::UndefinedConversionError.new(
          "#{spelled} #{undefined_path}", from, to, character
        )
      end
      @replacement = held
    end

    # Whether the conversion was told to stand a replacement in for what the
    # destination cannot spell.
    def replacing_undefined?
      return (@options & UNDEF_MASK) == UNDEF_REPLACE if @options.is_a? Integer
      return false unless @options.is_a? Hash
      @options[:undef] == :replace
    end
    private :replacing_undefined?

    def inspect
      "#<Encoding::Converter: #{@source.name} to #{@destination.name}>"
    end

    def to_s
      inspect
    end

    # The encoding a name stands for, which may be written as an Encoding, a
    # String, or anything that reads as one.
    def self.named held
      name = if held.is_a? String
        held
      elsif held.respond_to? :to_str
        held.to_str
      elsif held.respond_to? :name
        held.name
      else
        raise TypeError, "no implicit conversion of #{held.class} into String"
      end
      found = Encoding.find name
      if found.nil?
        raise Encoding::ConverterNotFoundError, "code converter not found (#{name})"
      end
      found
    end

    # The steps a conversion takes. Anything that is not already UTF-8 on one
    # side passes through UTF-8 on the way.
    def self.search_convpath source, destination, options = 0
      from = Encoding::Converter.named source
      to = Encoding::Converter.named destination
      if from.name == "ASCII-8BIT" && to.name != "ASCII-8BIT"
        raise Encoding::ConverterNotFoundError,
              "code converter not found (#{from.name} to #{to.name})"
      end
      path = if from == to || from == Encoding::UTF_8 || to == Encoding::UTF_8
        [[from, to]]
      else
        [[from, Encoding::UTF_8], [Encoding::UTF_8, to]]
      end
      path = path + ["crlf_newline"] if Encoding::Converter.crlf_wanted options
      path
    end

    def self.crlf_wanted options
      return false unless options.is_a? Hash
      options[:crlf_newline] ? true : false
    end

    # Carry text from the source encoding to the destination, refusing what
    # neither one can spell.
    def convert text
      raise ArgumentError, "converter already finished" if @finished
      held = text.to_s
      refuse_invalid held
      unless @pending.nil? || @pending.empty?
        held = held.byteslice(0, held.bytesize - @pending.bytesize)
      end
      converted = ""
      held.dup.force_encoding(@source.name).each_char do |character|
        refuse_undefined character unless spellable? character
        converted = converted + carried(character)
      end
      @errinfo = [:source_buffer_empty, nil, nil, nil, nil]
      @last_error = nil
      converted.dup.force_encoding @destination.name
    end

    # What went wrong the last time text was carried over, or nil when the
    # last attempt made it through.
    def last_error
      @last_error
    end

    # Carry what it can from `source` into `destination`, reporting how it
    # stopped rather than raising. The source is left holding whatever was
    # not read.
    def primitive_convert source, destination, destination_byteoffset = nil, destination_bytesize = nil, options = 0
      unless destination_byteoffset.nil?
        destination.replace destination.byteslice(0, destination_byteoffset)
      end
      held = source.dup.force_encoding "ASCII-8BIT"
      trouble = invalid_run held
      readable = trouble.nil? ? held : held.byteslice(0, trouble[0])
      written = ""
      consumed = 0
      status = nil
      readable.dup.force_encoding(@source.name).each_char do |character|
        if !destination_bytesize.nil? && written.bytesize + character.bytesize > destination_bytesize
          @errinfo = [:destination_buffer_full, nil, nil, nil, nil]
          @last_error = nil
          status = :destination_buffer_full
          break
        end
        unless spellable? character
          if replacing_undefined?
            written = written + @replacement
            consumed = consumed + character.bytesize
            next
          end
          from, to = stage_for :undefined
          bytes = character.dup.force_encoding("ASCII-8BIT")
          spelled = "U+" + character.ord.to_s(16).upcase.rjust(4, "0")
          @errinfo = [:undefined_conversion, from.name, to.name, bytes, ""]
          @last_error = Encoding::UndefinedConversionError.new(
            "#{spelled} from #{from.name} to #{to.name}", from, to, character
          )
          status = :undefined_conversion
          break
        end
        written = written + character
        consumed = consumed + character.bytesize
      end
      destination.replace destination + written.dup.force_encoding(@destination.name)
      if status.nil? && !trouble.nil?
        from, to = stage_for :invalid
        wrong = trouble[1]
        rest = trouble[2]
        truncated = trouble[3]
        status = truncated ? :incomplete_input : :invalid_byte_sequence
        @errinfo = [status, from.name, to.name, wrong, rest]
        @held_back = rest
        @last_error = Encoding::InvalidByteSequenceError.new(
          "#{wrong.inspect} on #{from.name}", from, to, wrong, rest, truncated
        )
        # The bytes that could not carry on are read too, and held for a
        # `putback` to hand to the next piece of text.
        consumed = trouble[0] + wrong.bytesize + rest.bytesize
      end
      if status.nil?
        status = partial_input_wanted(options) ? :source_buffer_empty : :finished
        @errinfo = [status, nil, nil, nil, nil]
        @last_error = nil
      end
      source.replace held.byteslice(consumed, held.bytesize - consumed)
      status
    end

    # Whether the caller said more text is still to come, which leaves the
    # conversion open rather than finishing it.
    def partial_input_wanted options
      return false unless options.is_a? Hash
      options[:partial_input] ? true : false
    end
    private :partial_input_wanted


    # Close the conversion, reporting a character the text stopped part-way
    # through. There is nothing more to carry over once it has been called.
    def finish
      pending = @pending
      @pending = nil
      @finished = true
      if @shifted
        @shifted = false
        return "\e(B".dup.force_encoding(@destination.name)
      end
      unless pending.nil? || pending.empty?
        from, to = stage_for :invalid
        @errinfo = [:incomplete_input, from.name, to.name, pending, ""]
        trouble = Encoding::InvalidByteSequenceError.new(
          "#{pending.inspect} on #{from.name}", from, to, pending, "", true
        )
        @last_error = trouble
        raise trouble
      end
      "".dup.force_encoding @destination.name
    end

    # The bytes held back after a run the source encoding could not read,
    # which the caller may put in front of the next piece of text. Reading
    # them takes them, so a second call answers nothing.
    def putback count = nil
      held = @held_back.nil? ? "" : @held_back
      wanted = count.nil? ? held.bytesize : count
      taken = held.byteslice(held.bytesize - wanted, wanted)
      taken = "" if taken.nil?
      @held_back = held.byteslice(0, held.bytesize - taken.bytesize)
      taken.dup.force_encoding @source.name
    end

    # What the last conversion ran into, as the tuple Ruby reports.
    def primitive_errinfo
      @errinfo.nil? ? [:source_buffer_empty, nil, nil, nil, nil] : @errinfo
    end

    # Whether the destination can spell a character at all. An encoding that
    # covers only the ASCII letters spells nothing above them, and one that
    # covers a single byte spells nothing wider.
    def spellable? character
      code = character.ord
      case @destination.name
      when "UTF-8", "UTF-16", "UTF-16BE", "UTF-16LE", "UTF-32", "UTF-32BE", "UTF-32LE", "CESU-8", "GB18030"
        true
      when "ISO-8859-1"
        code < 256
      when "EUC-JP"
        begin
          character.encode "EUC-JP"
          true
        rescue Encoding::UndefinedConversionError
          false
        end
      when "ISO-2022-JP"
        code < 128 || !jis_bytes(character).nil?
      else
        code < 128
      end
    end
    private :spellable?

    # The character written the way the destination spells it, which is the
    # character itself where the two encodings spell it the same way.
    def carried character
      return jis_carried(character) if @destination.name == "ISO-2022-JP"
      return character if character.ord < 128
      character.encode @destination.name
    rescue StandardError
      character
    end
    private :carried

    # The two bytes JIS X 0208 spells a character with, or nil where it has
    # none. They are the EUC-JP bytes with the high bit taken off.
    def jis_bytes character
      return nil if character.ord < 128
      begin
        spelled = character.encode("EUC-JP").bytes
      rescue StandardError
        return nil
      end
      return nil unless spelled.length == 2
      spelled.map { |byte| byte - 0x80 }
    end
    private :jis_bytes

    # ISO-2022-JP writes an escape before a run of two-byte characters and
    # another one before going back to ASCII, so the run is carried over with
    # whichever escape the switch calls for.
    def jis_carried character
      if character.ord < 128
        return character unless @shifted
        @shifted = false
        return "\e(B" + character
      end
      written = jis_bytes(character).pack("C*")
      return written if @shifted
      @shifted = true
      "\e$B" + written
    end
    private :jis_carried

    # The run of bytes the source encoding cannot read, as
    # `[offset, wrong, rest, truncated]`, or nil when every byte reads.
    def invalid_run held
      return nil if @source.name == "ASCII-8BIT"
      return nil if held.dup.force_encoding(@source.name).valid_encoding?
      # The bytes are cut apart rather than joined onto text, since joining
      # would read each one as the character it spells.
      bytes = held.bytes
      start = 0
      # The walk steps a whole character at a time, so a run cut inside one
      # is not mistaken for bytes the encoding cannot read at all.
      while start < bytes.length
        width = whole_character_width bytes, start
        break if width.nil?
        start = start + width
      end
      start = bytes.length - 1 if start >= bytes.length
      # An encoding written in units wider than a byte is cut at unit
      # boundaries, so the run that could not be read is the whole unit and
      # what follows it is the whole unit after that.
      unit = unit_width
      if unit > 1
        stop = start + unit
        stop = bytes.length if stop > bytes.length
        wrong = bytes[start..(stop - 1)].pack("C*").force_encoding "ASCII-8BIT"
        after = bytes[stop, unit]
        rest = after.nil? || after.empty? ? "" : after.pack("C*").force_encoding("ASCII-8BIT")
        return [start, wrong, rest, stop >= bytes.length]
      end
      # The run that opened the character, and the one byte after it that
      # could not carry on.
      stop = start + 1
      stop = stop + 1 while stop < bytes.length && carries_on?(bytes[stop])
      wrong = bytes[start..(stop - 1)].pack("C*").force_encoding "ASCII-8BIT"
      rest = stop < bytes.length ? [bytes[stop]].pack("C").force_encoding("ASCII-8BIT") : ""
      [start, wrong, rest, stop >= bytes.length]
    end
    private :invalid_run

    # How many bytes one unit of the source encoding takes. Most encodings
    # are written a byte at a time, while the wide ones are written in pairs
    # or in fours.
    def unit_width
      case @source.name
      when "UTF-16", "UTF-16BE", "UTF-16LE"
        2
      when "UTF-32", "UTF-32BE", "UTF-32LE"
        4
      else
        1
      end
    end
    private :unit_width

    # How many bytes the character opening at `start` takes, or nil when the
    # bytes there open no whole character the source encoding reads.
    def whole_character_width bytes, start
      width = 1
      while start + width <= bytes.length && width <= 6
        piece = bytes[start, width].pack("C*").force_encoding(@source.name)
        return width if piece.valid_encoding?
        width = width + 1
      end
      nil
    end
    private :whole_character_width

    # A run of bytes the source encoding cannot read is refused before any
    # of it is carried over.
    def refuse_invalid held
      found = invalid_run held
      return if found.nil?
      # A character the text stops part-way through is held back, since more
      # of it may still arrive. `finish` is where that is reported.
      if found[3]
        @pending = found[1]
        return
      end
      wrong = found[1]
      rest = found[2]
      truncated = found[3]
      from, to = stage_for :invalid
      @errinfo = [:invalid_byte_sequence, from.name, to.name, wrong, rest]
      @held_back = rest
      trouble = Encoding::InvalidByteSequenceError.new(
        "#{wrong.inspect} on #{from.name}", from, to, wrong, rest, truncated
      )
      @last_error = trouble
      raise trouble
    end
    private :refuse_invalid

    def refuse_undefined character
      spelled = "U+" + character.ord.to_s(16).upcase.rjust(4, "0")
      from, to = stage_for :undefined
      # A conversion that passes through another encoding refuses the
      # character at that step, so the character is reported as that step
      # reads it rather than as the text was read.
      refused = character.dup.force_encoding from.name
      @errinfo = [:undefined_conversion, from.name, to.name, refused, ""]
      trouble = Encoding::UndefinedConversionError.new(
        "#{spelled} #{undefined_path}", from, to, refused
      )
      @last_error = trouble
      raise trouble
    end

    # The steps the message names, which is every encoding the conversion
    # passes through rather than only the step that refused the character.
    def undefined_path
      steps = @convpath.select { |step| step.is_a? Array }
      return "from #{@source.name} to #{@destination.name}" if steps.empty?
      names = [steps.first[0].name]
      steps.each { |step| names.push step[1].name }
      "from " + names.join(" to ")
    end
    private :undefined_path
    private :refuse_undefined

    # Whether a byte carries on the character the one before it opened.
    def carries_on? byte
      return byte >= 0xa1 && byte <= 0xfe if @source.name == "EUC-JP"
      byte >= 0x80 && byte <= 0xbf
    end
    private :carries_on?

    # The step of the conversion the trouble belongs to. Bytes the source
    # cannot read stop the first step, while a character the destination
    # cannot spell stops the last one.
    def stage_for kind
      steps = @convpath.select { |step| step.is_a? Array }
      return [@source, @destination] if steps.empty?
      kind == :invalid ? steps.first : steps.last
    end
    private :stage_for

    # The ASCII-compatible encoding that stands in for one that is not, and
    # nil for one that already is.
    def self.asciicompat_encoding held
      found = begin
        Encoding::Converter.named held
      rescue Encoding::ConverterNotFoundError, ArgumentError
        nil
      end
      return nil if found.nil?
      return nil if found.ascii_compatible?
      return Encoding.find "stateless-ISO-2022-JP" if found.name.start_with? "ISO-2022-JP"
      Encoding::UTF_8
    end
  end
end
"##;
