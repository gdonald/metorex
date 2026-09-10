# The parts of OpenSSL that are arithmetic over bytes: the message digests,
# the keyed digest built on them, the key derivation built on that, and the
# comparison that takes the same time however two strings differ.

require 'digest'
require 'securerandom'

module OpenSSL
  OPENSSL_VERSION = "OpenSSL 3.0.0"
  OPENSSL_VERSION_NUMBER = 0x30000000
  OPENSSL_LIBRARY_VERSION = OPENSSL_VERSION
  VERSION = "3.2.0"

  class OpenSSLError < StandardError; end

  # A message digest named the way OpenSSL names one.
  class Digest < ::Digest::Class
    class DigestError < OpenSSL::OpenSSLError; end

    # What each name stands for: how wide the digest is and how wide the
    # block it is computed over is.
    SHAPES = {
      "MD5" => [16, 64],
      "SHA1" => [20, 64],
      "SHA224" => [28, 64],
      "SHA256" => [32, 64],
      "SHA384" => [48, 128],
      "SHA512" => [64, 128]
    }

    attr_reader :name

    def initialize(name, data = nil)
      unless name.is_a?(::String) || name.is_a?(::Symbol) || name.respond_to?(:to_str)
        raise TypeError, "no implicit conversion of #{name.class} into String"
      end
      held = name.is_a?(::String) ? name : name.to_s
      @name = OpenSSL::Digest.canonical_name held
      unless SHAPES.key? @name
        raise OpenSSL::Digest::DigestError, "Unsupported digest algorithm (#{held}).: unsupported"
      end
      super(data)
    end

    def self.canonical_name(held)
      named = held.to_s.upcase.gsub("-", "")
      named == "SHA" ? "SHA1" : named
    end

    def self.digest(name, data)
      new(name).digest(data)
    end

    def self.hexdigest(name, data)
      new(name).hexdigest(data)
    end

    def self.base64digest(name, data)
      new(name).base64digest(data)
    end

    def algorithm
      @name
    end

    def digest_length
      SHAPES[@name][0]
    end

    def block_length
      SHAPES[@name][1]
    end

    # Each name is also a class of its own, so `OpenSSL::Digest::SHA1.new`
    # names the same algorithm `OpenSSL::Digest.new("SHA1")` does.
    SHAPES.each_key do |named|
      built = ::Class.new(OpenSSL::Digest) do
        define_method :initialize do |data = nil|
          super(named, data)
        end
      end
      const_set named, built
    end
  end

  # A digest keyed with a secret, which says both what the message is and
  # that whoever wrote it held the key.
  class HMAC
    def self.digest(digest, key, data)
      OpenSSL::HMAC.computed digest, key, data
    end

    def self.hexdigest(digest, key, data)
      ::Digest.hexencode computed(digest, key, data)
    end

    def self.base64digest(digest, key, data)
      [computed(digest, key, data)].pack "m0"
    end

    # The keyed digest itself: the key is brought to one block, and the
    # message is digested once inside a padded block and once outside it.
    def self.computed(digest, key, data)
      named = digest.respond_to?(:name) ? digest.name : digest.to_s
      held = OpenSSL::Digest.new named
      block = held.block_length
      shortened = key.to_s
      shortened = OpenSSL::Digest.new(named).digest(shortened) if shortened.length > block
      shortened = shortened + ("\x00" * (block - shortened.length))
      inner = OpenSSL::HMAC.masked shortened, 0x36
      outer = OpenSSL::HMAC.masked shortened, 0x5c
      once = OpenSSL::Digest.new(named).digest(inner + data.to_s)
      OpenSSL::Digest.new(named).digest(outer + once)
    end

    # The key with every byte of it masked by the same value.
    def self.masked(key, mask)
      key.each_char.map { |held| (held.ord ^ mask).chr }.join
    end
  end

  # Turning a password into a key, which takes long enough to do that
  # guessing the password one try at a time is not worth the wait.
  module KDF
    class KDFError < OpenSSL::OpenSSLError; end

    def self.pbkdf2_hmac(pass, salt: nil, iterations: nil, length: nil, hash: nil)
      held = OpenSSL::KDF.coerced pass, "pass"
      seasoning = OpenSSL::KDF.coerced salt, "salt"
      rounds = OpenSSL::KDF.counted iterations, "iterations"
      wanted = OpenSSL::KDF.counted length, "length"
      named = hash.respond_to?(:name) ? hash.name : hash.to_s
      raise KDFError, "invalid iteration count" if rounds < 1
      raise KDFError, "invalid length" if wanted < 0
      return "" if wanted == 0
      # The rounds number in the tens of thousands and each is a pair of
      # digests, so the interpreter carries them.
      ::Digest.__pbkdf2__ OpenSSL::Digest.canonical_name(named), held, seasoning, rounds, wanted
    end

    def self.coerced(value, _what = nil)
      return value if value.is_a? ::String
      unless value.respond_to? :to_str
        raise TypeError, "no implicit conversion of #{value.class} into String"
      end
      value.to_str
    end

    def self.counted(value, _what = nil)
      return value if value.is_a? Integer
      unless value.respond_to? :to_int
        raise TypeError, "no implicit conversion of #{value.class} into Integer"
      end
      value.to_int
    end
  end

  # Bytes nobody can guess, which is what a key or a salt is made of.
  module Random
    def self.random_bytes(count = nil)
      SecureRandom.bytes OpenSSL::KDF.counted(count.nil? ? 16 : count, "length")
    end

    def self.pseudo_bytes(count = nil)
      random_bytes count
    end

    def self.seed(_text)
      nil
    end
  end

  # A cipher named the way OpenSSL names one. Metorex carries no cipher
  # implementations, so the class exists to be named and to refuse.
  class Cipher
    class CipherError < OpenSSLError; end

    def initialize name
      @name = name.to_s
      raise CipherError, "unsupported cipher algorithm (#{@name})"
    end

    def self.ciphers
      []
    end
  end

  module SSL
    # The settings a TLS connection is made under. Metorex carries the
    # settings themselves, and has no TLS transport to hand them to.
    class SSLContext
      DEFAULT_PARAMS = {}

      def initialize
        @params = {}
      end

      def set_params params = {}
        @params = params
        params.each do |name, value|
          instance_variable_set "@#{name}", value
        end
        @params
      end

      def params
        @params
      end

      def verify_mode
        @verify_mode
      end

      def verify_mode= mode
        @verify_mode = mode
      end

      def ca_file
        @ca_file
      end

      def ca_file= path
        @ca_file = path
      end
    end
  end

  # Comparing two strings without letting how long it takes say where they
  # first differ.
  def self.secure_compare(left, right)
    OpenSSL.fixed_length_secure_compare(
      ::Digest::SHA256.digest(OpenSSL::KDF.coerced(left, "a")),
      ::Digest::SHA256.digest(OpenSSL::KDF.coerced(right, "b"))
    ) && left == right
  end

  def self.fixed_length_secure_compare(left, right)
    held = OpenSSL::KDF.coerced left, "a"
    other = OpenSSL::KDF.coerced right, "b"
    unless held.length == other.length
      raise ArgumentError, "inputs must be of equal length"
    end
    differing = 0
    held.each_char.each_with_index do |piece, at|
      differing = differing | (piece.ord ^ other[at].ord)
    end
    differing == 0
  end
end
