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

  # Every name an encoding answers to, its own and the aliases pointing at
  # it, in the order Ruby registered them, then the names that follow the
  # settings.
  def self.name_list
    named = __name_list__
    aliases.each_key { |held| named.push(held) unless named.include?(held) }
    named.push("internal")
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

  # The encodings Ruby carries no converter to or from.
  def self.__without_converter__ name
    ["Emacs-Mule", "EUC-TW", "Windows-1258", "GB1988", "macCentEuro", "macThai",
     "ISO-2022-JP-2", "MacJapanese", "UTF-7"].include? name
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
      @newline_decorator = Encoding::Converter.newline_decorator options
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
      refused = from.name == "ASCII-8BIT" && to.name != "ASCII-8BIT"
      refused ||= from != to && (Encoding.__without_converter__(from.name) ||
                                 Encoding.__without_converter__(to.name))
      if refused
        raise Encoding::ConverterNotFoundError,
              "code converter not found (#{from.name} to #{to.name})"
      end
      path = if from == to
        [[from, to]]
      else
        names = Encoding::Converter.shortest_steps from.name, to.name
        names.each_cons(2).map { |pair| [Encoding.find(pair[0]), Encoding.find(pair[1])] }
      end
      decorator = Encoding::Converter.newline_decorator options
      return path if decorator.nil?
      # A decorator works on ASCII, so it goes before a last step into an
      # encoding that is not written in ASCII.
      last = path.last
      return path + [decorator] if from == to || last[1].ascii_compatible?
      path[0..-2] + [decorator, last]
    end

    # The steps between the Japanese encodings that do not pass through
    # UTF-8, by the name of the encoding each one starts from.
    JAPANESE_STEPS = {
      "Shift_JIS" => ["EUC-JP"],
      "EUC-JP" => ["Shift_JIS", "stateless-ISO-2022-JP"],
      "stateless-ISO-2022-JP" => ["EUC-JP", "ISO-2022-JP"],
      "ISO-2022-JP" => ["stateless-ISO-2022-JP"]
    }

    # The encodings only another Japanese encoding steps to or from.
    REACHED_THROUGH_EUC_JP = ["stateless-ISO-2022-JP", "ISO-2022-JP"]

    # The encodings one step from `name` on the way to `destination`. Every
    # encoding steps to and from UTF-8 except the ones reached through EUC-JP.
    def self.next_steps name, destination
      found = JAPANESE_STEPS.fetch name, []
      return found if REACHED_THROUGH_EUC_JP.include? name
      return found + ["UTF-8"] unless name == "UTF-8"
      return ["EUC-JP"] if REACHED_THROUGH_EUC_JP.include? destination
      [destination, "EUC-JP"]
    end

    # The names of the encodings the fewest steps from `from` to `to` pass
    # through, both ends included.
    def self.shortest_steps from, to
      came_from = { from => nil }
      waiting = [from]
      until waiting.empty?
        name = waiting.shift
        Encoding::Converter.next_steps(name, to).each do |step|
          next if came_from.key? step
          came_from[step] = name
          waiting.push step
          next unless step == to
          names = [to]
          names.unshift came_from[names.first] until came_from[names.first].nil?
          return names
        end
      end
    end

    # The decorator that rewrites line endings, by the name the convpath
    # gives it, or nil when the options ask for none.
    def self.newline_decorator options
      if options.is_a? Integer
        return "universal_newline" if options & UNIVERSAL_NEWLINE_DECORATOR != 0
        return "crlf_newline" if options & CRLF_NEWLINE_DECORATOR != 0
        return "cr_newline" if options & CR_NEWLINE_DECORATOR != 0
        return nil
      end
      return nil unless options.is_a? Hash
      named = options[:newline]
      return "#{named}_newline" if [:universal, :crlf, :cr].include? named
      return "universal_newline" if options[:universal_newline]
      return "crlf_newline" if options[:crlf_newline]
      return "cr_newline" if options[:cr_newline]
      nil
    end

    # Carry text from the source encoding to the destination through each
    # step of the convpath, refusing what a step cannot read or spell. A
    # character the text stops part-way through is held for the next piece.
    def convert text
      raise ArgumentError, "converter already finished" if @finished
      held = text.to_s.b
      held = @pending.b + held unless @pending.nil? || @pending.empty?
      @pending = nil
      converted = "".b
      read_from_source(held).each_char do |character|
        newline_decorated(character).each do |piece|
          converted = converted + written_out(piece).b
        end
      end
      @errinfo = [:source_buffer_empty, nil, nil, nil, nil]
      @last_error = nil
      converted.force_encoding @destination.name
    end

    # A character with the line ending the decorator asks for written in its
    # place. A carriage return read with universal newlines is held until
    # the character after it shows whether it ends the line by itself.
    def newline_decorated character
      return [character] if @newline_decorator.nil?
      code = character.ord
      if @newline_decorator == "universal_newline"
        held = @carriage_return_held
        @carriage_return_held = code == 13
        return held ? ["\n"] : [] if code == 13
        return ["\n"] if code == 10
        return held ? ["\n", character] : [character]
      end
      return [character] unless code == 10
      @newline_decorator == "crlf_newline" ? ["\r", "\n"] : ["\r"]
    end
    private :newline_decorated

    # The text the first step reads, as characters of the source encoding,
    # or of EUC-JP when the source is ISO-2022-JP.
    def read_from_source held
      return jis_decoded held if @source.name == "ISO-2022-JP"
      refuse_invalid held.dup.force_encoding(@source.name)
      unless @pending.nil? || @pending.empty?
        held = held.byteslice(0, held.bytesize - @pending.bytesize)
      end
      held.force_encoding @source.name
    end
    private :read_from_source

    # A character carried through the rest of the convpath and written the
    # way the destination spells it.
    def written_out character
      return carried_into_jis(character) if @destination.name == "ISO-2022-JP"
      return character.encode(@destination.name) if character.encoding.name == "EUC-JP" && @source.name == "ISO-2022-JP"
      read = character.encoding == Encoding::UTF_8 ? character : character.encode("UTF-8")
      return read if @destination == Encoding::UTF_8
      unless spellable? read
        return @replacement if replacing_undefined?
        refuse_in_step Encoding::UTF_8, @destination, read
      end
      carried read
    end
    private :written_out

    # A character carried into ISO-2022-JP, which reaches it through EUC-JP
    # and the stateless form of ISO-2022-JP that holds only two-byte
    # characters.
    def carried_into_jis character
      euc = character
      unless euc.encoding.name == "EUC-JP"
        begin
          euc = character.encode "EUC-JP"
        rescue Encoding::UndefinedConversionError
          return replaced_in_jis if replacing_undefined?
          refuse_in_step character.encoding, Encoding.find("EUC-JP"), character
        end
      end
      if euc.bytesize > 2 || (euc.bytesize == 2 && euc.getbyte(0) == 0x8e)
        return replaced_in_jis if replacing_undefined?
        refuse_in_step Encoding.find("EUC-JP"), Encoding.find("stateless-ISO-2022-JP"), euc
      end
      jis_carried euc
    end
    private :carried_into_jis

    # The replacement written into ISO-2022-JP, which switches back to ASCII
    # first when a run of two-byte characters is open.
    def replaced_in_jis
      written = ""
      @replacement.each_char { |character| written = written + jis_carried(character) }
      written
    end
    private :replaced_in_jis

    # Refuse a character the step from `from` to `to` cannot spell, naming
    # the step the way Ruby does: by itself when it is the whole conversion,
    # and as part of the convpath otherwise.
    def refuse_in_step from, to, character
      spelled = if from == Encoding::UTF_8
        "U+" + character.ord.to_s(16).upcase.rjust(4, "0")
      else
        character.b.dump
      end
      message = if from == @source && to == @destination
        "#{spelled} from #{from.name} to #{to.name}"
      else
        steps = @convpath.select { |step| step.is_a? Array }
        names = [steps.first[0].name] + steps.map { |step| step[1].name }
        "#{spelled} to #{to.name} in conversion from #{names.join(" to ")}"
      end
      @errinfo = [:undefined_conversion, from.name, to.name, character.b, ""]
      trouble = Encoding::UndefinedConversionError.new message, from, to, character
      @last_error = trouble
      raise trouble
    end
    private :refuse_in_step

    # The escapes ISO-2022-JP switches sets with: the first two into JIS X
    # 0208, the last two back into ASCII.
    JIS_ESCAPES = ["\e$B", "\e$@", "\e(B", "\e(J"].map(&:b)

    # ISO-2022-JP read back into EUC-JP, following the escapes that switch
    # between ASCII and JIS X 0208. An escape or a character the text stops
    # part-way through is held back for the next piece.
    def jis_decoded held
      bytes = held.bytes
      written = []
      at = 0
      while at < bytes.length
        byte = bytes[at]
        if byte == 0x1b
          rest = held.byteslice(at, 3)
          found = JIS_ESCAPES.index rest
          unless found.nil?
            @reading_jis = found < 2
            at = at + 3
            next
          end
          if JIS_ESCAPES.any? { |escape| escape.start_with? rest }
            @pending = rest
            break
          end
          refuse_jis_byte held.byteslice(at, 1), held.byteslice(at + 1, 1) || ""
        end
        refuse_jis_byte held.byteslice(at, 1), "" if byte >= 0x80
        if @reading_jis && byte >= 0x21 && byte <= 0x7e
          if at + 1 >= bytes.length
            @pending = held.byteslice(at, 1)
            break
          end
          following = bytes[at + 1]
          unless following >= 0x21 && following <= 0x7e
            refuse_jis_byte held.byteslice(at, 2), ""
          end
          written.push byte + 0x80, following + 0x80
          at = at + 2
          next
        end
        written.push byte
        at = at + 1
      end
      written.pack("C*").force_encoding "EUC-JP"
    end
    private :jis_decoded

    # Refuse bytes ISO-2022-JP cannot read, which stops the step into its
    # stateless form.
    def refuse_jis_byte wrong, rest
      to = Encoding.find "stateless-ISO-2022-JP"
      @errinfo = [:invalid_byte_sequence, @source.name, to.name, wrong, rest]
      @held_back = rest
      trouble = Encoding::InvalidByteSequenceError.new(
        "#{wrong.inspect} on #{@source.name}", @source, to, wrong, rest, false
      )
      @last_error = trouble
      raise trouble
    end
    private :refuse_jis_byte

    # What went wrong the last time text was carried over, or nil when the
    # last attempt made it through.
    def last_error
      @last_error
    end

    # Carry what it can from `source` into `destination`, reporting how it
    # stopped rather than raising. The source is left holding whatever was
    # not read. Output that does not fit in `destination_bytesize` is kept and
    # written first the next time, and so are the bytes an invalid run asks
    # to be read again and a character the input stopped part-way through.
    def primitive_convert source, destination, destination_byteoffset = nil, destination_bytesize = nil, options = 0
      destination_byteoffset = destination_byteoffset.to_int unless destination_byteoffset.nil?
      destination_bytesize = destination_bytesize.to_int unless destination_bytesize.nil?
      if destination_byteoffset.nil?
        destination_byteoffset = destination.bytesize
      elsif destination_byteoffset > destination.bytesize
        raise ArgumentError, "output_byteoffset too big"
      end
      kept = destination.byteslice(0, destination_byteoffset)
      # A conversion that finished takes nothing more.
      if @primitive_finished
        destination.replace kept
        return :finished
      end
      given = source.nil? ? "" : source
      carried_over = (@held_back || "") + (@partial || "")
      @held_back = nil
      @partial = nil
      held = carried_over.b + given.b
      trouble = invalid_run held
      readable = trouble.nil? ? held : held.byteslice(0, trouble[0])
      written = "".b
      consumed = 0
      status = nil
      readable.dup.force_encoding(@source.name).each_char do |character|
        unless spellable? character
          consumed = consumed + character.bytesize
          if replacing_undefined?
            written = written + @replacement.b
            next
          end
          from, to = stage_for :undefined
          bytes = character.b
          spelled = "U+" + character.ord.to_s(16).upcase.rjust(4, "0")
          @errinfo = [:undefined_conversion, from.name, to.name, bytes, ""]
          @last_error = Encoding::UndefinedConversionError.new(
            "#{spelled} from #{from.name} to #{to.name}", from, to, character
          )
          status = :undefined_conversion
          break
        end
        written = written + carried(character).b
        consumed = consumed + character.bytesize
      end
      if status.nil? && !trouble.nil?
        wrong = trouble[1]
        rest = trouble[2]
        truncated = trouble[3]
        consumed = trouble[0] + wrong.bytesize + rest.bytesize
        if truncated && partial_input_wanted(options)
          @partial = wrong
        else
          from, to = stage_for :invalid
          status = truncated ? :incomplete_input : :invalid_byte_sequence
          @errinfo = [status, from.name, to.name, wrong, rest]
          @held_back = rest
          @last_error = Encoding::InvalidByteSequenceError.new(
            "#{wrong.inspect} on #{from.name}", from, to, wrong, rest, truncated
          )
        end
      end
      if status.nil?
        status = partial_input_wanted(options) ? :source_buffer_empty : :finished
        @errinfo = [status, nil, nil, nil, nil]
        @last_error = nil
      end
      # A finished conversion into ISO-2022-JP ends back in ASCII.
      if status == :finished && @shifted
        @shifted = false
        written = written + "\e(B".b
      end
      output = (@pending_output || "".b) + written
      @pending_output = nil
      if !destination_bytesize.nil? && output.bytesize > destination_bytesize
        @pending_output = output.byteslice(destination_bytesize, output.bytesize - destination_bytesize)
        output = output.byteslice(0, destination_bytesize)
        status = :destination_buffer_full
        @errinfo = [status, nil, nil, nil, nil]
        @last_error = nil
      end
      @primitive_finished = status == :finished
      destination.replace (kept.b + output).force_encoding(@destination.name)
      unless source.nil?
        left = held.byteslice(consumed, held.bytesize - consumed)
        source.replace left.force_encoding(source.encoding)
      end
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
      written = "".b
      if @carriage_return_held
        @carriage_return_held = false
        written = written_out("\n").b
      end
      if @shifted
        @shifted = false
        return (written + "\e(B".b).force_encoding(@destination.name)
      end
      unless pending.nil? || pending.empty?
        from, to = stage_for :invalid
        pending = pending.b
        @errinfo = [:incomplete_input, from.name, to.name, pending, ""]
        trouble = Encoding::InvalidByteSequenceError.new(
          "incomplete #{pending.inspect} on #{from.name}", from, to, pending, "", true
        )
        @last_error = trouble
        raise trouble
      end
      written.force_encoding @destination.name
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
      when "UTF-8", "UTF8-MAC", "UTF-16", "UTF-16BE", "UTF-16LE", "UTF-32", "UTF-32BE", "UTF-32LE", "CESU-8", "GB18030"
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
      return character if character.ord < 128 && @destination.ascii_compatible?
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
      # A byte that opens no character is wrong by itself, and nothing after
      # it is read again.
      if @source.name != "EUC-JP" && bytes[start] < 0xc0
        return [start, [bytes[start]].pack("C").force_encoding("ASCII-8BIT"), "", false]
      end
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
