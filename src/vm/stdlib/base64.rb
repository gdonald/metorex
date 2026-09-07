# Base64 encoding, in the four spellings Ruby ships: the one that wraps every
# sixty characters, the strict one that does not, and their URL-safe forms.
module Base64
  ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
  SLOTS = {}
  ALPHABET.chars.each_with_index { |letter, slot| SLOTS[letter] = slot }

  def self.encode64(text)
    wrapped(strict_encode64(text))
  end

  def self.strict_encode64(text)
    bytes = text.bytes
    written = ""
    index = 0
    while index < bytes.size
      first = bytes[index]
      second = bytes[index + 1]
      third = bytes[index + 2]
      written += ALPHABET[first >> 2]
      written += ALPHABET[((first & 0x03) << 4) | (second.nil? ? 0 : second >> 4)]
      if second.nil?
        written += "=="
      else
        written += ALPHABET[((second & 0x0f) << 2) | (third.nil? ? 0 : third >> 6)]
        written += third.nil? ? "=" : ALPHABET[third & 0x3f]
      end
      index += 3
    end
    written
  end

  def self.decode64(text)
    strict_decode64(text.gsub(/[^A-Za-z0-9+\/=]/, ""))
  end

  def self.strict_decode64(text)
    unless text.length % 4 == 0
      raise ArgumentError, "invalid base64"
    end
    bytes = []
    index = 0
    while index < text.length
      quad = text[index, 4]
      raise ArgumentError, "invalid base64" if quad.length < 4
      slots = quad.chars.map { |letter| letter == "=" ? nil : SLOTS[letter] }
      raise ArgumentError, "invalid base64" if slots[0].nil? || slots[1].nil?
      bytes.push(((slots[0] << 2) | (slots[1] >> 4)) & 0xff)
      unless slots[2].nil?
        bytes.push((((slots[1] & 0x0f) << 4) | (slots[2] >> 2)) & 0xff)
        unless slots[3].nil?
          bytes.push((((slots[2] & 0x03) << 6) | slots[3]) & 0xff)
        end
      end
      index += 4
    end
    bytes.map { |byte| byte.chr }.join
  end

  def self.urlsafe_encode64(text, padding: true)
    written = strict_encode64(text).gsub("+", "-").gsub("/", "_")
    padding ? written : written.gsub("=", "")
  end

  def self.urlsafe_decode64(text)
    restored = text.gsub("-", "+").gsub("_", "/")
    unless restored.length % 4 == 0
      restored += "=" * (4 - restored.length % 4)
    end
    strict_decode64(restored)
  end

  # Base64 written sixty characters to a line, which is what `encode64` and
  # the mail formats it came from use.
  def self.wrapped(text)
    return "" if text.empty?
    lines = []
    index = 0
    while index < text.length
      lines.push(text[index, 60])
      index += 60
    end
    lines.join("\n") + "\n"
  end
  private_class_method :wrapped
end
