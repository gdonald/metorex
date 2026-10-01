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
      # Another digest names the same algorithm, and nothing of the state it
      # has built up comes along with it.
      name = name.name if name.is_a? OpenSSL::Digest
      unless name.is_a?(::String) || name.respond_to?(:to_str)
        raise TypeError, "no implicit conversion of #{name.class} into String"
      end
      held = name.is_a?(::String) ? name : name.to_str
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

    def self.pbkdf2_hmac(pass, salt:, iterations:, length:, hash:)
      unless hash.is_a?(::String) || hash.is_a?(OpenSSL::Digest) || hash.respond_to?(:to_str)
        raise TypeError, "wrong argument type #{hash.class} (expected OpenSSL/Digest)"
      end
      held = OpenSSL::KDF.coerced pass, "pass"
      seasoning = OpenSSL::KDF.coerced salt, "salt"
      rounds = OpenSSL::KDF.counted iterations, "iterations"
      wanted = OpenSSL::KDF.counted length, "length"
      named = hash.respond_to?(:name) ? hash.name : hash.to_s
      raise KDFError, "PKCS5_PBKDF2_HMAC: invalid iteration count" if rounds < 1
      raise KDFError, "invalid length" if wanted < 0
      return "" if wanted == 0
      # The rounds number in the tens of thousands and each is a pair of
      # digests, so the interpreter carries them.
      ::Digest.__pbkdf2__ OpenSSL::Digest.canonical_name(named), held, seasoning, rounds, wanted
    end

    SCRYPT_KEYWORDS = %i[salt N r p length].freeze

    # scrypt takes every one of its parameters by keyword. N, the size of
    # the table each block is mixed through, is a power of two above 1, and
    # r and p are positive.
    def self.scrypt(pass, **options)
      missing = SCRYPT_KEYWORDS - options.keys
      unless missing.empty?
        named = missing.map(&:inspect).join(", ")
        raise ArgumentError, "missing keyword#{"s" if missing.size > 1}: #{named}"
      end
      unknown = options.keys - SCRYPT_KEYWORDS
      unless unknown.empty?
        named = unknown.map(&:inspect).join(", ")
        raise ArgumentError, "unknown keyword#{"s" if unknown.size > 1}: #{named}"
      end
      held = OpenSSL::KDF.coerced pass
      seasoning = OpenSSL::KDF.coerced options[:salt]
      table = OpenSSL::KDF.counted options[:N]
      width = OpenSSL::KDF.counted options[:r]
      blocks = OpenSSL::KDF.counted options[:p]
      wanted = OpenSSL::KDF.counted options[:length]
      raise ArgumentError, "negative string size (or size too big)" if wanted < 0
      if table < 2 || (table & (table - 1)) != 0 || width < 1 || blocks < 1
        raise KDFError, "EVP_PBE_scrypt"
      end
      ::Digest.__scrypt__ held, seasoning, table, width, blocks, wanted
    end

    def self.coerced(value, _what = nil)
      return value if value.is_a? ::String
      unless value.respond_to? :to_str
        named = value.nil? || value == true || value == false ? value.inspect : value.class
        raise TypeError, "no implicit conversion of #{named} into String"
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

module OpenSSL
  # The universal ASN.1 tags, by the names OpenSSL gives them.
  module ASN1
    UNIVERSAL_TAG_NAME = [
      "EOC", "BOOLEAN", "INTEGER", "BIT_STRING", "OCTET_STRING", "NULL", "OBJECT",
      "OBJECT_DESCRIPTOR", "EXTERNAL", "REAL", "ENUMERATED", "EMBEDDED_PDV", "UTF8STRING",
      "RELATIVE_OID", nil, nil, "SEQUENCE", "SET", "NUMERICSTRING", "PRINTABLESTRING",
      "T61STRING", "VIDEOTEXSTRING", "IA5STRING", "UTCTIME", "GENERALIZEDTIME", "GRAPHICSTRING",
      "ISO64STRING", "GENERALSTRING", "UNIVERSALSTRING", "CHARACTER_STRING", "BMPSTRING"
    ].freeze
    UNIVERSAL_TAG_NAME.each_with_index do |named, tag|
      const_set named, tag unless named.nil?
    end

    # The tag and content of the DER value at the start of `bytes`, and the
    # bytes after it, or nil when they do not hold one.
    def self.__read_der__(bytes)
      return nil if bytes.bytesize < 2
      tag = bytes.getbyte(0)
      length = bytes.getbyte(1)
      offset = 2
      if length >= 0x80
        count = length & 0x7f
        return nil if count.zero? || bytes.bytesize < 2 + count
        length = bytes.byteslice(2, count).unpack1("H*").to_i(16)
        offset += count
      end
      return nil if bytes.bytesize < offset + length
      [tag, bytes.byteslice(offset, length), bytes.byteslice(offset + length..)]
    end

    # Every DER value one after another in `bytes`.
    def self.__read_all__(bytes)
      found = []
      rest = bytes
      until rest.empty?
        held = __read_der__(rest)
        return nil if held.nil?
        found << held[0, 2]
        rest = held[2]
      end
      found
    end

    def self.__integer_value__(content)
      content.unpack1("H*").to_i(16)
    end

    # DER: a tag, the length of the content, and the content.
    def self.__encoded__(tag, content)
      bytes = content.b
      size = bytes.bytesize
      length = if size < 0x80
                 size.chr
               else
                 digits = []
                 while size > 0
                   digits.unshift(size & 0xff)
                   size >>= 8
                 end
                 (0x80 | digits.size).chr + digits.pack("C*")
               end
      (tag.chr + length).b + bytes
    end
  end

  module X509
    class NameError < OpenSSL::OpenSSLError; end

    # A distinguished name: a list of attributes, each an object with a
    # value and the ASN.1 string type the value is written in.
    class Name
      include Comparable

      COMPAT = 0
      RFC2253 = 17_892_119
      ONELINE = 8_520_479
      MULTILINE = 44_302_342
      DEFAULT_OBJECT_TYPE = OpenSSL::ASN1::UTF8STRING
      OBJECT_TYPE_TEMPLATE = Hash.new(OpenSSL::ASN1::UTF8STRING).merge(
        "C" => OpenSSL::ASN1::PRINTABLESTRING,
        "countryName" => OpenSSL::ASN1::PRINTABLESTRING,
        "serialNumber" => OpenSSL::ASN1::PRINTABLESTRING,
        "dnQualifier" => OpenSSL::ASN1::PRINTABLESTRING,
        "DC" => OpenSSL::ASN1::IA5STRING,
        "domainComponent" => OpenSSL::ASN1::IA5STRING,
        "emailAddress" => OpenSSL::ASN1::IA5STRING
      ).freeze

      # The ASN.1 string types OpenSSL takes a name's value in. Two others
      # fail in its ASN.1 library, and the rest are refused as the entry's
      # value.
      ACCEPTED_VALUE_TYPES = [3, 7, 8, 9, 11, 12, 13, 14, 15, 18, 19, 20, 22, 29].freeze
      LIBRARY_REFUSED_TYPES = [1, 6].freeze

      # "/A=B/C=D" or "A=B, C=D".
      def self.parse_openssl(text, template = OBJECT_TYPE_TEMPLATE)
        pairs = if text.start_with?("/")
                  text[1..].split("/").map { |part| part.split("=", 2) }
                else
                  text.split(",").map { |part| part.strip.split("=", 2) }
                end
        new(pairs, template)
      end

      def self.parse(text, template = OBJECT_TYPE_TEMPLATE)
        parse_openssl(text, template)
      end

      def initialize(entries = nil, template = OBJECT_TYPE_TEMPLATE)
        @entries = []
        return if entries.nil?
        entries.each do |oid, value, type|
          type ||= template[oid]
          add_entry(oid, value, type)
        end
      end

      def add_entry(oid, value, type = nil, loc: -1, set: 0)
        named = oid.to_s
        text = OpenSSL::KDF.coerced(value)
        found = OpenSSL.__object__(:find, named)
        if found.nil? || found[2].nil?
          raise NameError, "X509_NAME_add_entry_by_txt: invalid field name (name=#{named})"
        end
        type = OBJECT_TYPE_TEMPLATE[named] if type.nil?
        if LIBRARY_REFUSED_TYPES.include?(type)
          raise NameError, "X509_NAME_add_entry_by_txt: ASN1 lib"
        end
        unless ACCEPTED_VALUE_TYPES.include?(type)
          raise NameError, "X509_NAME_add_entry_by_txt: nested asn1 error (Field=value, Type=X509_NAME_ENTRY)"
        end
        stored = type == OpenSSL::ASN1::UTF8STRING ? text.dup.force_encoding("UTF-8") : text.b
        entry = { short: found[0], long: found[1], oid: found[2], value: stored, type: type }
        at = loc.negative? ? @entries.size : [loc, @entries.size].min
        @entries.insert(at, entry)
        self
      end

      def to_a
        @entries.map { |entry| [entry[:short], entry[:value], entry[:type]] }
      end

      def to_s(format = nil)
        case format
        when nil then @entries.map { |entry| "/#{entry[:short]}=#{oneline_value(entry[:value])}" }.join
        when RFC2253 then @entries.reverse.map { |entry| "#{entry[:short]}=#{rfc2253_value(entry[:value], false)}" }.join(",")
        when ONELINE then @entries.map { |entry| "#{entry[:short]} = #{quoted_value(entry[:value])}" }.join(", ")
        when MULTILINE
          @entries.map { |entry| "#{entry[:long].ljust(25)} = #{control_escaped(entry[:value])}" }.join("\n")
        else @entries.map { |entry| "#{entry[:short]}=#{oneline_value(entry[:value])}" }.join(", ")
        end
      end

      def to_utf8
        @entries.reverse.map { |entry| "#{entry[:short]}=#{rfc2253_value(entry[:value], true)}" }.join(",")
          .force_encoding("UTF-8")
      end

      def inspect
        "#<#{self.class.name} #{to_utf8}>"
      end

      def pretty_print(printer)
        printer.text(inspect)
      end

      def to_der
        sets = @entries.map do |entry|
          pair = OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::OBJECT, OpenSSL.__object__(:encode, entry[:oid])) +
                 OpenSSL::ASN1.__encoded__(entry[:type], entry[:value])
          OpenSSL::ASN1.__encoded__(0x31, OpenSSL::ASN1.__encoded__(0x30, pair))
        end
        OpenSSL::ASN1.__encoded__(0x30, sets.join.b)
      end

      # The order OpenSSL compares names in: by the length of their canonical
      # encoding, then by its bytes. The canonical encoding writes each value
      # as UTF-8, lowercased, with spaces trimmed and runs of them collapsed.
      def cmp(other)
        return nil unless other.is_a?(Name)
        mine = canonical_encoding
        theirs = other.canonical_encoding
        held = mine.bytesize <=> theirs.bytesize
        held = mine <=> theirs if held.zero?
        held
      end

      def <=>(other)
        cmp(other)
      end

      def eql?(other)
        other.is_a?(Name) && cmp(other).zero?
      end

      def hash
        canonical_encoding.hash
      end

      alias hash_old hash

      def canonical_encoding
        @entries.map do |entry|
          value = entry[:value].b.downcase.strip.gsub(/\s+/, " ")
          pair = OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::OBJECT, OpenSSL.__object__(:encode, entry[:oid])) +
                 OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::UTF8STRING, value)
          OpenSSL::ASN1.__encoded__(0x31, OpenSSL::ASN1.__encoded__(0x30, pair))
        end.join.b
      end
      protected :canonical_encoding

      private

      # X509_NAME_oneline: '/' and '+' escaped, and bytes outside printable
      # ASCII written as \xHH.
      def oneline_value(value)
        value.b.each_byte.map do |byte|
          if byte == 0x2f || byte == 0x2b then "\\#{byte.chr}"
          elsif byte < 0x20 || byte > 0x7e then format("\\x%02X", byte)
          else byte.chr
          end
        end.join
      end

      # RFC 2253: the separators and the quoting characters escaped, a
      # leading '#' or space and a trailing space escaped, and control bytes
      # written as \HH. Bytes above ASCII are \HH too, unless the name is
      # written as UTF-8.
      def rfc2253_value(value, utf8)
        bytes = value.b.bytes
        last = bytes.size - 1
        written = +""
        index = 0
        while index < bytes.size
          byte = bytes[index]
          if byte >= 0x80 && utf8
            width = byte >= 0xf0 ? 4 : byte >= 0xe0 ? 3 : byte >= 0xc0 ? 2 : 1
            written << bytes[index, width].pack("C*").force_encoding("UTF-8")
            index += width
            next
          end
          written << if ",+\"\\<>;".include?(byte.chr) then "\\#{byte.chr}"
                     elsif index.zero? && (byte == 0x23 || byte == 0x20) then "\\#{byte.chr}"
                     elsif index == last && byte == 0x20 then "\\ "
                     elsif byte < 0x20 || byte >= 0x7f then format("\\%02X", byte)
                     else byte.chr
                     end
          index += 1
        end
        written
      end

      # ONELINE: a value holding a separator, or with a leading '#' or space
      # or a trailing space, is quoted. Quotes and backslashes are escaped
      # either way.
      def quoted_value(value)
        bytes = value.b
        escaped = control_escaped(bytes.gsub(/["\\]/) { |held| "\\#{held}" })
        needs_quotes = bytes.match?(/[,+<>;]/) || bytes.start_with?("#", " ") || bytes.end_with?(" ")
        needs_quotes ? "\"#{escaped}\"" : escaped
      end

      def control_escaped(value)
        value.b.each_byte.map { |byte| byte < 0x20 || byte >= 0x7f ? format("\\%02X", byte) : byte.chr }.join
      end
    end
  end
end

module OpenSSL
  class BNError < OpenSSLError; end

  # An arbitrary-precision integer, as OpenSSL hands numbers to Ruby.
  class BN
    include Comparable

    def initialize(value = 0, base = 10)
      @value = case value
               when BN then value.to_i
               when Integer then value
               when String then base == 2 ? value.b.unpack1("H*").to_i(16) : value.to_i(base)
               else raise TypeError, "Cannot convert #{value.class} into OpenSSL::BN"
               end
    end

    def to_i
      @value
    end
    alias to_int to_i

    def to_s(base = 10)
      return @value.to_s(16).upcase.then { |text| text.length.odd? ? "0#{text}" : text } if base == 16
      return OpenSSL::ASN1.__integer_bytes__(@value) if base == 2
      @value.to_s(base)
    end

    # The number is held in C, so the inspection names only the object.
    def inspect
      ::Kernel.instance_method(:to_s).bind_call(self)
    end

    def num_bits
      @value.bit_length
    end

    def num_bytes
      (num_bits + 7) / 8
    end

    def <=>(other)
      @value <=> (other.is_a?(BN) ? other.to_i : other)
    end

    def ==(other)
      other.respond_to?(:to_int) && @value == other.to_int
    end
    alias eql? ==

    def hash
      @value.hash
    end

    def coerce(other)
      [other, @value]
    end
  end

  module ASN1
    # The big-endian bytes of a non-negative integer, with no leading zeros.
    def self.__integer_bytes__(value)
      return "\x00".b if value.zero?
      hex = value.to_s(16)
      hex = "0#{hex}" if hex.length.odd?
      [hex].pack("H*")
    end

    def self.__integer__(value)
      bytes = __integer_bytes__(value)
      bytes = "\x00".b + bytes if bytes.getbyte(0) >= 0x80
      __encoded__(INTEGER, bytes)
    end

    def self.__sequence__(*parts)
      __encoded__(0x30, parts.join.b)
    end

    def self.__object__(oid)
      __encoded__(OBJECT, OpenSSL.__object__(:encode, oid))
    end

    # A time as an X.509 validity bound: UTCTime through 2049, and
    # GeneralizedTime after.
    def self.__time__(time)
      held = time.utc
      if held.year < 2050
        __encoded__(UTCTIME, held.strftime("%y%m%d%H%M%SZ"))
      else
        __encoded__(GENERALIZEDTIME, held.strftime("%Y%m%d%H%M%SZ"))
      end
    end
  end

  module PKey
    class PKeyError < OpenSSL::OpenSSLError; end

    class PKey; end

    # An RSA key pair, or the public half of one.
    class RSA < PKey
      class RSAError < PKeyError; end

      RSA_OID = "1.2.840.113549.1.1.1"

      # The DigestInfo prefix PKCS #1 v1.5 puts ahead of each digest.
      DIGEST_INFO = {
        "SHA1" => ["3021300906052b0e03021a05000414"].pack("H*"),
        "SHA256" => ["3031300d060960864801650304020105000420"].pack("H*"),
        "SHA384" => ["3041300d060960864801650304020205000430"].pack("H*"),
        "SHA512" => ["3051300d060960864801650304020305000440"].pack("H*")
      }.freeze

      def self.generate(bits, exponent = 65_537)
        new(bits, exponent)
      end

      # A key generated at the size given, or read from the PEM or DER text
      # of one.
      def initialize(bits = nil, exponent = 65_537)
        return if bits.nil?
        if bits.is_a?(String)
          parts = RSA.__read__(bits)
          raise PKeyError, "Neither PUB key nor PRIV key" if parts.nil?
          @n, @e, @d, @p, @q, @dmp1, @dmq1, @iqmp = parts
          return
        end
        raise TypeError, "no implicit conversion of #{bits.class} into Integer" unless bits.is_a?(Integer)
        if bits < 512
          raise PKeyError, "EVP_PKEY_CTX_ctrl_str(ctx, \"rsa_keygen_bits\", \"#{bits}\"): key size too small"
        end
        if exponent.negative?
          raise PKeyError, "EVP_PKEY_CTX_ctrl_str(ctx, \"rsa_keygen_pubexp\", \"#{exponent}\"): invalid negative value"
        end
        raise PKeyError, "EVP_PKEY_keygen: pub exponent out of range" if exponent < 3 || exponent.even?
        @n, @e, @d, @p, @q, @dmp1, @dmq1, @iqmp = OpenSSL.__rsa_generate__(bits, exponent)
      end

      PEM_LABELS = ["RSA PRIVATE KEY", "PUBLIC KEY", "RSA PUBLIC KEY"].freeze

      # The parts of a key written as PKCS #1 RSAPrivateKey, as a
      # SubjectPublicKeyInfo, or as PKCS #1 RSAPublicKey, in DER or in PEM.
      def self.__read__(text)
        der = text.b
        if text.include?("-----BEGIN ")
          matched = text.match(/-----BEGIN (#{PEM_LABELS.join("|")})-----(.*?)-----END \1-----/m)
          return nil if matched.nil?
          der = matched[2].unpack1("m")
        end
        top = OpenSSL::ASN1.__read_der__(der)
        return nil unless top && top[0] == 0x30
        fields = OpenSSL::ASN1.__read_all__(top[1])
        return nil if fields.nil?
        if fields.size == 9 && fields.all? { |tag, _| tag == OpenSSL::ASN1::INTEGER }
          return fields.drop(1).map { |_, content| OpenSSL::ASN1.__integer_value__(content) }
        end
        if fields.size == 2 && fields.all? { |tag, _| tag == OpenSSL::ASN1::INTEGER }
          return fields.map { |_, content| OpenSSL::ASN1.__integer_value__(content) }
        end
        if fields.size == 2 && fields[0][0] == 0x30 && fields[1][0] == OpenSSL::ASN1::BIT_STRING
          inner = OpenSSL::ASN1.__read_der__(fields[1][1].byteslice(1..))
          numbers = inner && OpenSSL::ASN1.__read_all__(inner[1])
          return nil unless numbers && numbers.size == 2
          return numbers.map { |_, content| OpenSSL::ASN1.__integer_value__(content) }
        end
        nil
      end

      def to_pem
        label = private? ? "RSA PRIVATE KEY" : "PUBLIC KEY"
        "-----BEGIN #{label}-----\n#{[to_der].pack("m48")}-----END #{label}-----\n"
      end
      alias to_s to_pem
      alias export to_pem

      def public_to_pem
        "-----BEGIN PUBLIC KEY-----\n#{[public_to_der].pack("m48")}-----END PUBLIC KEY-----\n"
      end

      def self.__from_parts__(parts)
        made = allocate
        made.instance_variable_set(:@n, parts[0])
        made.instance_variable_set(:@e, parts[1])
        made
      end

      %i[n e d p q dmp1 dmq1 iqmp].each do |part|
        define_method(part) do
          held = instance_variable_get(:"@#{part}")
          held.nil? ? nil : OpenSSL::BN.new(held)
        end
      end

      def private?
        !@d.nil?
      end

      def public?
        !@n.nil?
      end

      def public_key
        RSA.__from_parts__([@n, @e])
      end

      # SubjectPublicKeyInfo: the algorithm, then the key itself as a
      # SEQUENCE of the modulus and the public exponent.
      def public_to_der
        key = OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__integer__(@n), OpenSSL::ASN1.__integer__(@e))
        algorithm = OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__object__(RSA_OID), OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::NULL, ""))
        OpenSSL::ASN1.__sequence__(algorithm, OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::BIT_STRING, "\x00".b + key))
      end

      # The bytes of the public key a key identifier hashes: the key's own
      # SEQUENCE of modulus and exponent.
      def __public_key_bits__
        OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__integer__(@n), OpenSSL::ASN1.__integer__(@e))
      end

      def to_der
        return public_to_der unless private?
        OpenSSL::ASN1.__sequence__(*[0, @n, @e, @d, @p, @q, @dmp1, @dmq1, @iqmp].map { |part| OpenSSL::ASN1.__integer__(part) })
      end

      def sign(digest, data)
        raise ArgumentError, "private key is needed" unless private?
        hash, prefix = __digested__(digest, data)
        width = (@n.bit_length + 7) / 8
        padding = width - prefix.bytesize - hash.bytesize - 3
        raise PKeyError, "EVP_DigestSign: RSA lib" if padding < 8
        encoded = "\x00\x01".b + ("\xFF".b * padding) + "\x00".b + prefix + hash
        message = encoded.unpack1("H*").to_i(16)
        # The two halves of the private operation, joined by the Chinese
        # remainder theorem.
        first = message.pow(@dmp1, @p)
        second = message.pow(@dmq1, @q)
        joined = second + ((@iqmp * (first - second)) % @p) * @q
        OpenSSL::ASN1.__integer_bytes__(joined).rjust(width, "\x00".b)
      end

      def verify(digest, signature, data)
        hash, prefix = __digested__(digest, data)
        width = (@n.bit_length + 7) / 8
        return false unless signature.bytesize == width
        recovered = signature.b.unpack1("H*").to_i(16).pow(@e, @n)
        bytes = OpenSSL::ASN1.__integer_bytes__(recovered).rjust(width, "\x00".b)
        padding = width - prefix.bytesize - hash.bytesize - 3
        padding >= 8 && bytes == "\x00\x01".b + ("\xFF".b * padding) + "\x00".b + prefix + hash
      end

      def __digested__(digest, data)
        named = OpenSSL::Digest.canonical_name(digest.respond_to?(:name) ? digest.name : digest.to_s)
        prefix = DIGEST_INFO[named]
        raise PKeyError, "unsupported digest #{named}" if prefix.nil?
        [OpenSSL::Digest.digest(named, data).b, prefix]
      end
      private :__digested__
    end
  end
end

module OpenSSL
  module ASN1
    class ASN1Error < OpenSSL::OpenSSLError; end
  end

  module X509
    class ExtensionError < OpenSSL::OpenSSLError; end
    class CertificateError < OpenSSL::OpenSSLError; end
    class StoreError < OpenSSL::OpenSSLError; end

    V_OK = 0
    V_ERR_CERT_SIGNATURE_FAILURE = 7
    V_ERR_CERT_NOT_YET_VALID = 9
    V_ERR_CERT_HAS_EXPIRED = 10
    V_ERR_DEPTH_ZERO_SELF_SIGNED_CERT = 18
    V_ERR_SELF_SIGNED_CERT_IN_CHAIN = 19
    V_ERR_UNABLE_TO_GET_ISSUER_CERT_LOCALLY = 20

    VERIFY_ERROR_STRINGS = {
      0 => "ok",
      7 => "certificate signature failure",
      9 => "certificate is not yet valid or the system clock is incorrect",
      10 => "certificate has expired",
      18 => "self-signed certificate",
      19 => "self-signed certificate in certificate chain",
      20 => "unable to get local issuer certificate"
    }.freeze

    # One extension: the object it names, the DER of its value, and whether
    # a reader that does not know it must refuse the certificate.
    class Extension
      attr_reader :oid

      def initialize(oid, value, critical = false, shown = nil)
        @oid = oid
        @der_value = value.b
        @critical = critical
        @shown = shown
      end

      def critical?
        @critical
      end

      def value
        @shown || @der_value.unpack1("H*").scan(/../).map(&:upcase).join(":")
      end

      def value_der
        @der_value
      end

      def to_der
        named = OpenSSL.__object__(:find, @oid)[2]
        parts = [OpenSSL::ASN1.__object__(named)]
        parts << OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::BOOLEAN, "\xFF".b) if @critical
        parts << OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::OCTET_STRING, @der_value)
        OpenSSL::ASN1.__sequence__(*parts)
      end

      def to_s
        "#{@oid} = #{"critical, " if @critical}#{value}"
      end

      def to_a
        [@oid, value, @critical]
      end
    end

    # Builds extensions from the text OpenSSL's configuration files write
    # them in.
    class ExtensionFactory
      attr_accessor :subject_certificate, :issuer_certificate, :subject_request, :crl, :config

      KEY_USAGE_BITS = {
        "digitalSignature" => [0, "Digital Signature"],
        "nonRepudiation" => [1, "Non Repudiation"],
        "keyEncipherment" => [2, "Key Encipherment"],
        "dataEncipherment" => [3, "Data Encipherment"],
        "keyAgreement" => [4, "Key Agreement"],
        "keyCertSign" => [5, "Certificate Sign"],
        "cRLSign" => [6, "CRL Sign"],
        "encipherOnly" => [7, "Encipher Only"],
        "decipherOnly" => [8, "Decipher Only"]
      }.freeze

      def initialize(issuer = nil, subject = nil, request = nil, crl = nil)
        @issuer_certificate = issuer
        @subject_certificate = subject
        @subject_request = request
        @crl = crl
      end

      def create_extension(oid, value, critical = false)
        found = OpenSSL.__object__(:find, oid.to_s)
        named = found && found[1]
        der, shown = case named
                     when "X509v3 Basic Constraints" then basic_constraints(value)
                     when "X509v3 Key Usage" then key_usage(value)
                     when "X509v3 Subject Key Identifier" then subject_key_identifier(value)
                     when "X509v3 Authority Key Identifier" then authority_key_identifier(value)
                     end
        if der.nil?
          raise ExtensionError, "#{oid} = #{value}: error in extension (name=#{oid}, value=#{value})"
        end
        Extension.new(found[0], der, critical, shown)
      end

      private

      def basic_constraints(value)
        parts = value.split(",").map(&:strip)
        authority = parts.include?("CA:TRUE")
        length = parts.find { |part| part.start_with?("pathlen:") }&.split(":", 2)&.last
        body = []
        body << OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::BOOLEAN, "\xFF".b) if authority
        body << OpenSSL::ASN1.__integer__(length.to_i) if length
        [OpenSSL::ASN1.__sequence__(*body), value]
      end

      # A BIT STRING of the uses named, trimmed to the last bit set.
      def key_usage(value)
        names = value.split(",").map(&:strip)
        return nil unless names.all? { |name| KEY_USAGE_BITS.key?(name) }
        bits = names.map { |name| KEY_USAGE_BITS[name][0] }
        width = bits.max + 1
        bytes = Array.new((width + 7) / 8, 0)
        bits.each { |bit| bytes[bit / 8] |= 0x80 >> (bit % 8) }
        unused = bytes.size * 8 - width
        shown = names.map { |name| KEY_USAGE_BITS[name][1] }.join(", ")
        [OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::BIT_STRING, unused.chr + bytes.pack("C*")), shown]
      end

      def subject_key_identifier(value)
        return nil unless value == "hash" && @subject_certificate&.public_key
        identifier = OpenSSL::X509.__key_identifier__(@subject_certificate.public_key)
        [OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::OCTET_STRING, identifier), hex(identifier)]
      end

      def authority_key_identifier(value)
        return nil unless value.start_with?("keyid") && @issuer_certificate&.public_key
        identifier = OpenSSL::X509.__key_identifier__(@issuer_certificate.public_key)
        [OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__encoded__(0x80, identifier)), hex(identifier)]
      end

      def hex(bytes)
        bytes.unpack1("H*").scan(/../).map(&:upcase).join(":")
      end
    end

    # The identifier a key is known by: the SHA-1 of its public key bits.
    def self.__key_identifier__(key)
      OpenSSL::Digest.digest("SHA1", key.__public_key_bits__).b
    end

    SIGNATURE_ALGORITHMS = {
      "SHA1" => ["sha1WithRSAEncryption", "1.2.840.113549.1.1.5"],
      "SHA256" => ["sha256WithRSAEncryption", "1.2.840.113549.1.1.11"],
      "SHA384" => ["sha384WithRSAEncryption", "1.2.840.113549.1.1.12"],
      "SHA512" => ["sha512WithRSAEncryption", "1.2.840.113549.1.1.13"]
    }.freeze

    class Certificate
      attr_reader :version, :subject, :issuer, :extensions

      def initialize
        @version = 0
        @serial = 0
        @subject = Name.new
        @issuer = Name.new
        @extensions = []
      end

      def version=(value)
        @version = value
      end

      def serial
        OpenSSL::BN.new(@serial)
      end

      def serial=(value)
        @serial = value.to_i
      end

      def subject=(name)
        @subject = name
      end

      def issuer=(name)
        @issuer = name
      end

      def public_key
        raise CertificateError, "decode error" if @public_key.nil?
        @public_key
      end

      def signature_algorithm
        raise OpenSSL::OpenSSLError, "OBJ_obj2txt" if @signature.nil?
        @signature_algorithm
      end

      def public_key=(key)
        @public_key = key.public_key
      end

      def not_before
        raise OpenSSL::ASN1::ASN1Error, "ASN1_TIME_to_tm" if @not_before.nil?
        @not_before
      end

      def not_before=(time)
        @not_before = Time.at(time.to_i).utc
      end

      def not_after
        raise OpenSSL::ASN1::ASN1Error, "ASN1_TIME_to_tm" if @not_after.nil?
        @not_after
      end

      def not_after=(time)
        @not_after = Time.at(time.to_i).utc
      end

      def add_extension(extension)
        @extensions << extension
        extension
      end

      def extensions=(list)
        @extensions = list.to_a
      end

      def sign(key, digest)
        named = OpenSSL::Digest.canonical_name(digest.respond_to?(:name) ? digest.name : digest.to_s)
        @signature_algorithm, @signature_oid = SIGNATURE_ALGORITHMS.fetch(named)
        @digest_name = named
        @signature = key.sign(named, tbs_der)
        self
      end

      def verify(key)
        raise CertificateError, "unknown signature algorithm" if @signature.nil?
        key.verify(@digest_name, @signature, tbs_der)
      end

      def to_der
        raise CertificateError, "illegal zero content" if @signature.nil?
        OpenSSL::ASN1.__sequence__(
          tbs_der,
          algorithm_der,
          OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::BIT_STRING, "\x00".b + @signature)
        )
      end

      # Issued by itself: its subject names its issuer.
      def __self_issued__
        @subject.cmp(@issuer).zero?
      end

      def __valid_at__(time)
        return :not_yet if !@not_before.nil? && time < @not_before
        return :expired if !@not_after.nil? && time > @not_after
        :valid
      end

      private

      def algorithm_der
        OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__object__(@signature_oid), OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::NULL, ""))
      end

      # TBSCertificate: what the signature covers.
      def tbs_der
        parts = []
        parts << OpenSSL::ASN1.__encoded__(0xa0, OpenSSL::ASN1.__integer__(@version)) unless @version.zero?
        parts << OpenSSL::ASN1.__integer__(@serial)
        parts << algorithm_der
        parts << @issuer.to_der
        parts << OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__time__(not_before), OpenSSL::ASN1.__time__(not_after))
        parts << @subject.to_der
        parts << @public_key.public_to_der
        unless @extensions.empty?
          listed = OpenSSL::ASN1.__sequence__(*@extensions.map(&:to_der))
          parts << OpenSSL::ASN1.__encoded__(0xa3, listed)
        end
        OpenSSL::ASN1.__sequence__(*parts)
      end
    end

    # The certificates trusted to anchor a chain, and the result of the last
    # verification made against them.
    class Store
      attr_reader :error, :chain
      attr_accessor :time, :flags, :purpose, :trust, :verify_callback

      def initialize
        @trusted = []
        @error = 0
        @chain = nil
      end

      def add_cert(certificate)
        @trusted << certificate
        self
      end

      def set_default_paths
        self
      end

      def error_string
        VERIFY_ERROR_STRINGS.fetch(@error, "certificate verify error")
      end

      # Build the chain from the certificate up to one the store trusts,
      # finding each issuer by name and signature, and then check every
      # certificate's validity period, the root's first.
      def verify(certificate, untrusted = nil)
        candidates = @trusted + untrusted.to_a
        chain = [certificate]
        current = certificate
        loop do
          if current.__self_issued__ && current.verify(current.public_key)
            unless @trusted.include?(current)
              return failed(chain, chain.size == 1 ? V_ERR_DEPTH_ZERO_SELF_SIGNED_CERT : V_ERR_SELF_SIGNED_CERT_IN_CHAIN)
            end
            break
          end
          issuer = candidates.find do |held|
            !held.equal?(current) && held.subject.cmp(current.issuer).zero? && current.verify(held.public_key)
          end
          return failed(chain, V_ERR_UNABLE_TO_GET_ISSUER_CERT_LOCALLY) if issuer.nil?
          chain << issuer
          current = issuer
          break if @trusted.include?(issuer) && !issuer.__self_issued__
        end
        now = @time || Time.now
        chain.reverse_each do |held|
          case held.__valid_at__(now)
          when :not_yet then return failed(chain, V_ERR_CERT_NOT_YET_VALID)
          when :expired then return failed(chain, V_ERR_CERT_HAS_EXPIRED)
          end
        end
        @chain = chain
        @error = V_OK
        true
      end

      private

      def failed(chain, code)
        @chain = chain
        @error = code
        false
      end
    end
  end
end
