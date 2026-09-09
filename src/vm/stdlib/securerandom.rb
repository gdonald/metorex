# Random values in the shapes a program usually wants them: hex digits, raw
# bytes, base64 text, and numbers within a range.
require 'base64'

# The draw itself, written outside the module so `rand` names the one the
# language gives rather than the module's own.
SECURE_RANDOM_DRAW = lambda { |limit| limit.nil? ? rand : rand(limit) }

module SecureRandom
  DEFAULT_LENGTH = 16
  DRAW = SECURE_RANDOM_DRAW
  HEX_DIGITS = "0123456789abcdef"

  def self.bytes(count)
    random_bytes(count)
  end

  # A run of bytes with no character meaning, which is what BINARY says.
  def self.random_bytes(count = nil)
    wanted = counted_length(count)
    written = ""
    wanted.times { written += DRAW.call(256).chr }
    written.force_encoding(Encoding::BINARY)
  end

  def self.hex(count = nil)
    wanted = counted_length(count)
    written = ""
    (wanted * 2).times { written += HEX_DIGITS[DRAW.call(16)] }
    written
  end

  def self.base64(count = nil)
    Base64.strict_encode64(random_bytes(count))
  end

  def self.urlsafe_base64(count = nil, padding = false)
    written = Base64.urlsafe_encode64(random_bytes(count))
    padding ? written : written.gsub("=", "")
  end

  def self.uuid
    parts = [8, 4, 4, 4, 12].map do |width|
      written = ""
      width.times { written += HEX_DIGITS[DRAW.call(16)] }
      written
    end
    parts.join("-")
  end

  def self.alphanumeric(count = nil)
    letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
    wanted = counted_length(count)
    written = ""
    wanted.times { written += letters[DRAW.call(letters.length)] }
    written
  end

  def self.random_number(limit = nil)
    return DRAW.call(nil) if limit.nil?
    return DRAW.call(limit) if limit.is_a?(Range)
    return DRAW.call(nil) if limit.is_a?(Numeric) && limit <= 0
    DRAW.call(limit)
  end

  def self.rand(limit = nil)
    random_number(limit)
  end

  # How many bytes a call asked for. A missing length means the default, and
  # a length that is not an Integer is asked to become one.
  def self.counted_length(count)
    return DEFAULT_LENGTH if count.nil?
    wanted = if count.is_a?(Integer)
      count
    elsif count.is_a?(Float)
      count.to_i
    else
      count.to_int
    end
    raise ArgumentError, "negative string size (or size too big)" if wanted < 0
    wanted
  end
  private_class_method :counted_length
end
