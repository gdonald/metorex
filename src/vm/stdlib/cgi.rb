# Escaping text for a URL and for a page. A URL carries only a few characters
# as themselves, and a page reads five of them as markup, so both need what
# is written into them spelled out.

module CGI
  # `escape` writes a form value: everything but the unreserved characters is
  # written as its bytes in hex, and a space is written as a plus.
  def self.escape(text)
    CGI.coerced(text).each_char.map { |held| CGI.escaped_for_form(held) }.join
  end

  def self.escaped_for_form(held)
    return held if held =~ /\A[a-zA-Z0-9_\-.~]\z/
    return "+" if held == " "
    CGI.hex_bytes held
  end

  def self.unescape(text, _encoding = nil)
    CGI.read_percent CGI.coerced(text).gsub("+", " ")
  end

  # `escapeURIComponent` writes a piece of a URL, where a space is written in
  # hex like anything else and the tilde stands for itself.
  def self.escapeURIComponent(text)
    CGI.coerced(text).each_char.map { |held| CGI.escaped_for_uri(held) }.join
  end

  def self.escaped_for_uri(held)
    return held if held =~ /\A[a-zA-Z0-9_\-.~]\z/
    CGI.hex_bytes held
  end

  def self.unescapeURIComponent(text, _encoding = nil)
    CGI.read_percent CGI.coerced(text)
  end

  # The five characters a page reads as markup.
  def self.escapeHTML(text)
    held = CGI.coerced text
    held = held.gsub "&", "&amp;"
    held = held.gsub "<", "&lt;"
    held = held.gsub ">", "&gt;"
    held = held.gsub "\"", "&quot;"
    held.gsub "'", "&#39;"
  end

  # The same five read back, along with a character named by its number.
  def self.unescapeHTML(text, _encoding = nil)
    held = CGI.coerced text
    return held unless held.include? "&"
    answer = ""
    at = 0
    while at < held.length
      piece = held[at]
      if piece != "&"
        answer = answer + piece
        at = at + 1
        next
      end
      finish = held.index ";", at
      named = finish.nil? ? nil : held[at + 1, finish - at - 1]
      read = CGI.named_character named
      if read.nil?
        answer = answer + piece
        at = at + 1
      else
        answer = answer + read
        at = finish + 1
      end
    end
    answer
  end

  # What one `&...;` stands for, or nil when it names nothing.
  def self.named_character(named)
    return nil if named.nil? || named.empty?
    case named
    when "amp" then "&"
    when "lt" then "<"
    when "gt" then ">"
    when "quot" then "\""
    when "apos" then "'"
    else
      return CGI.numbered_character named[1..-1] if named.start_with? "#"
      nil
    end
  end

  def self.numbered_character(digits)
    return nil if digits.nil? || digits.empty?
    if digits.start_with?("x") || digits.start_with?("X")
      rest = digits[1..-1]
      return nil unless rest =~ /\A[0-9a-fA-F]+\z/
      return rest.to_i(16).chr
    end
    return nil unless digits =~ /\A[0-9]+\z/
    digits.to_i.chr
  end

  # `escapeElement` escapes only the tags of the elements it is given, so the
  # rest of the markup is left standing.
  def self.escapeElement(text, *elements)
    named = elements.flatten.map { |held| held.to_s.downcase }
    CGI.walk_tags(CGI.coerced(text)) do |tag, name|
      named.include?(name) ? CGI.escapeHTML(tag) : tag
    end
  end

  def self.unescapeElement(text, *elements)
    named = elements.flatten.map { |held| held.to_s.downcase }
    held = CGI.coerced text
    answer = ""
    at = 0
    while at < held.length
      if held[at, 4] == "&lt;"
        finish = held.index "&gt;", at
        if finish.nil?
          answer = answer + held[at]
          at = at + 1
          next
        end
        piece = held[at, finish - at + 4]
        read = CGI.unescapeHTML piece
        name = CGI.tag_name read
        answer = answer + (named.include?(name) ? read : piece)
        at = finish + 4
      else
        answer = answer + held[at]
        at = at + 1
      end
    end
    answer
  end

  # Walk the tags in a piece of markup, handing each to the block along with
  # the name of the element it opens or closes.
  def self.walk_tags(held)
    answer = ""
    at = 0
    while at < held.length
      if held[at] == "<"
        finish = held.index ">", at
        if finish.nil?
          answer = answer + held[at..-1]
          break
        end
        tag = held[at, finish - at + 1]
        answer = answer + yield(tag, CGI.tag_name(tag))
        at = finish + 1
      else
        answer = answer + held[at]
        at = at + 1
      end
    end
    answer
  end

  # The element a tag opens or closes, named in lower case.
  def self.tag_name(tag)
    named = tag.sub("<", "").sub(">", "")
    named = named[1..-1] if named.start_with? "/"
    named.split(/[\s\/]/).first.to_s.downcase
  end

  # Each byte a character stands for, written as `%` and two hex figures.
  def self.hex_bytes(held)
    held.each_byte.map { |byte| "%%%02X" % byte }.join
  end

  # Read every `%NN` back into the byte it names.
  def self.read_percent(held)
    answer = ""
    bytes = []
    at = 0
    while at < held.length
      if held[at] == "%" && held[at + 1, 2] =~ /\A[0-9a-fA-F]{2}\z/
        bytes.push held[at + 1, 2].to_i(16)
        at = at + 3
      else
        answer = answer + CGI.text_of(bytes) unless bytes.empty?
        bytes = []
        answer = answer + held[at]
        at = at + 1
      end
    end
    answer = answer + CGI.text_of(bytes) unless bytes.empty?
    answer
  end

  # The text a run of bytes stands for.
  # The text a run of bytes stands for. Bytes that spell characters in UTF-8
  # read back as those characters, and bytes that spell nothing stand for
  # themselves.
  def self.text_of(bytes)
    held = [bytes.map { |byte| "%02x" % byte }.join].pack "H*"
    points = held.unpack "U*"
    return held if points.empty?
    rebuilt = points.map { |point| "%c" % point }.join
    rebuilt.bytes == bytes ? rebuilt : held
  end

  def self.coerced(text)
    return text if text.is_a? ::String
    unless text.respond_to? :to_str
      raise TypeError, "no implicit conversion of #{text.class} into String"
    end
    text.to_str
  end
end
