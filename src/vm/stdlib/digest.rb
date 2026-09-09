# Message digests. The algorithms themselves are computed by the interpreter;
# what is here is the shape Ruby gives them, which is one class per algorithm
# over a shared instance protocol.

module Digest
  # The methods every digest object answers, whatever algorithm is behind it.
  # A class that includes this and defines no algorithm of its own raises when
  # asked to do the work, which is what the base protocol does in Ruby.
  module Instance
    def update(_string)
      raise RuntimeError, "#{self.class} does not implement update()"
    end

    def <<(string)
      update string
    end

    def finish
      raise RuntimeError, "#{self.class} does not implement finish()"
    end

    def reset
      raise RuntimeError, "#{self.class} does not implement reset()"
    end

    def new
      duplicate = self.clone
      duplicate.reset
      duplicate
    end

    def digest(string = nil)
      return finish if string.nil?
      # A digest taken of a given message leaves the object blank, which is
      # what makes the one-off form free of side effects.
      reset
      update string
      answer = finish
      reset
      answer
    end

    def digest!
      answer = finish
      reset
      answer
    end

    def hexdigest(string = nil)
      Digest.hexencode digest(string)
    end

    def hexdigest!
      Digest.hexencode digest!
    end

    def base64digest(string = nil)
      [digest(string)].pack "m0"
    end

    def base64digest!
      [digest!].pack "m0"
    end

    def to_s
      hexdigest
    end

    def inspect
      "#<#{self.class}: #{hexdigest}>"
    end

    def ==(other)
      if other.is_a? Digest::Instance
        to_s == other.to_s
      elsif other.respond_to? :to_str
        to_s == other.to_str
      else
        false
      end
    end

    def length
      digest_length
    end

    def size
      digest_length
    end

    def digest_length
      digest.length
    end

    def block_length
      raise RuntimeError, "#{self.class} does not implement block_length()"
    end

  end

  # A digest that keeps the message it has been given and computes the answer
  # when it is asked for one. Each algorithm subclasses this and names itself.
  class Class
    include Digest::Instance

    def self.digest(string)
      new.update(string).finish
    end

    def self.hexdigest(string)
      Digest.hexencode digest(string)
    end

    def self.base64digest(string)
      [digest(string)].pack "m0"
    end

    def self.file(name)
      new.file name
    end

    def initialize(string = nil)
      @message = ""
      update string unless string.nil?
    end

    def algorithm
      raise RuntimeError, "#{self.class} does not name an algorithm"
    end

    def update(string)
      @message = @message + Digest.coerce_message(string)
      self
    end

    def <<(string)
      update string
    end

    def reset
      @message = ""
      self
    end

    def finish
      Digest.__digest__ algorithm, @message
    end

    def file(name)
      update File.binread(Digest.coerce_path(name))
      self
    end

    def initialize_copy(other)
      @message = other.instance_variable_get :@message
      self
    end
  end

  # `Digest::Base` is where MRI puts the algorithms it implements in C. The
  # distinction does not survive here, so it names the same class. Each of the
  # library's names loads this one file, so the constant is only set once.
  Base = Class unless defined? Digest::Base

  class MD5 < Digest::Class
    def algorithm
      "MD5"
    end

    def digest_length
      16
    end

    def block_length
      64
    end
  end

  class SHA1 < Digest::Class
    def algorithm
      "SHA1"
    end

    def digest_length
      20
    end

    def block_length
      64
    end
  end

  class SHA256 < Digest::Class
    def algorithm
      "SHA256"
    end

    def digest_length
      32
    end

    def block_length
      64
    end
  end

  class SHA384 < Digest::Class
    def algorithm
      "SHA384"
    end

    def digest_length
      48
    end

    def block_length
      128
    end
  end

  class SHA512 < Digest::Class
    def algorithm
      "SHA512"
    end

    def digest_length
      64
    end

    def block_length
      128
    end
  end

  # `Digest::SHA2` picks one of the three by bit length.
  class SHA2 < Digest::Class
    def initialize(bitlen = 256, string = nil)
      @held = case bitlen
              when 256 then Digest::SHA256.new
              when 384 then Digest::SHA384.new
              when 512 then Digest::SHA512.new
              end
      if @held.nil?
        raise ArgumentError, "unsupported bit length: #{bitlen}"
      end
      @bitlen = bitlen
      super(string)
    end

    def self.digest(string, bitlen = 256)
      new(bitlen).update(string).finish
    end

    def self.hexdigest(string, bitlen = 256)
      Digest.hexencode digest(string, bitlen)
    end

    def algorithm
      @held.algorithm
    end

    def digest_length
      @held.digest_length
    end

    def block_length
      @held.block_length
    end

    def inspect
      "#<#{self.class}:#{@bitlen} #{hexdigest}>"
    end
  end

  # The message a digest was given, which has to be a String or something
  # that says how to read one out of it.
  def self.coerce_message(string)
    return string if string.is_a? ::String
    unless string.respond_to? :to_str
      raise TypeError, "no implicit conversion of #{string.class} into String"
    end
    string.to_str
  end

  # Each character of a digest stands for one byte, so the encoding reads the
  # characters rather than the bytes a text encoding would spell them with.
  def self.hexencode(string)
    Digest.coerce_message(string).each_char.map { |held| "%02x" % held.ord }.join
  end

  # The name of a file to read, which is either a String or something that
  # says how to read one out of it.
  def self.coerce_path(name)
    return name if name.is_a? ::String
    return name.to_path if name.respond_to? :to_path
    unless name.respond_to? :to_str
      raise TypeError, "no implicit conversion of #{name.class} into String"
    end
    name.to_str
  end

  # The bytes a string stands for, one to a character.
  def self.message_bytes(string)
    Digest.coerce_message(string).each_char.map { |held| held.ord }
  end

  # Bubble Babble names a string in syllables, which is easier to read aloud
  # than hex and is what `Digest.bubblebabble` answers.
  def self.bubblebabble(string)
    vowels = "aeiouy"
    consonants = "bcdfghklmnprstvzx"
    bytes = Digest.message_bytes(string)
    seed = 1
    spoken = "x"
    rounds = bytes.length / 2 + 1
    rounds.times do |round|
      if round + 1 < rounds || bytes.length.odd?
        first = bytes[2 * round]
        byte1 = ((first >> 6) & 3) + seed
        spoken = spoken + vowels[byte1 % 6]
        spoken = spoken + consonants[(first >> 2) & 15]
        spoken = spoken + vowels[(((first & 3) + (seed / 6)) % 6)]
        if round + 1 < rounds
          second = bytes[2 * round + 1]
          spoken = spoken + consonants[(second >> 4) & 15]
          spoken = spoken + "-"
          spoken = spoken + consonants[second & 15]
          seed = (seed * 5 + first * 7 + second) % 36
        end
      else
        spoken = spoken + vowels[seed % 6]
        spoken = spoken + "x"
        spoken = spoken + vowels[seed / 6]
      end
    end
    spoken + "x"
  end
end
