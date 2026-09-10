# The readable shapes a run of random bytes is handed back in.

require 'securerandom'

class Random
  module Formatter
    ALPHANUMERIC = [*"A".."Z", *"a".."z", *"0".."9"]

    # Each character comes from one random byte, so the alphabet is read one
    # element at a time rather than converted up front.
    def alphanumeric size = nil, chars: ALPHANUMERIC
      size = 16 if size.nil?
      unless size.is_a? Integer
        raise ArgumentError, "invalid argument - #{size}"
      end
      held = bytes size
      (0...size).map { |at| chars[held[at].ord % chars.length].to_s }.join
    end

    def hex size = nil
      bytes(size.nil? ? 16 : size).unpack1 "H*"
    end

    def base64 size = nil
      [bytes(size.nil? ? 16 : size)].pack "m0"
    end

    def urlsafe_base64 size = nil, padding = false
      held = base64(size).tr "+/", "-_"
      padding ? held : held.delete("=")
    end

    def uuid
      held = bytes(16).unpack "C*"
      held[6] = (held[6] & 0x0f) | 0x40
      held[8] = (held[8] & 0x3f) | 0x80
      shaped = held.map { |byte| format "%02x", byte }.join
      "#{shaped[0, 8]}-#{shaped[8, 4]}-#{shaped[12, 4]}-#{shaped[16, 4]}-#{shaped[20, 12]}"
    end

    def random_number limit = nil
      limit.nil? || limit == 0 ? rand : rand(limit)
    end
  end
end
