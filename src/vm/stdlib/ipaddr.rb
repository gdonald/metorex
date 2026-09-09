# An IPv4 or IPv6 address, held as the number it stands for together with the
# mask that says how much of it names the network.

# The address families IPAddr reads and reports. Ruby takes these from the
# socket library, which is where a program that has one will find them.
unless defined?(Socket)
  module Socket
    AF_UNSPEC = 0
    AF_INET = 2
    AF_INET6 = 30
  end
end

class IPAddr
  include Comparable

  IN4MASK = 0xffffffff
  IN6MASK = 0xffffffffffffffffffffffffffffffff

  class Error < ArgumentError
  end

  class AddressFamilyError < Error
  end

  class InvalidAddressError < Error
  end

  class InvalidPrefixError < InvalidAddressError
  end

  def self.new_ntoh(written)
    IPAddr.new(IPAddr.ntop(written))
  end

  # A packed address read back as the text it stands for.
  def self.ntop(written)
    bytes = written.bytes
    return bytes.map { |one| one.to_s }.join(".") if bytes.length == 4
    unless bytes.length == 16
      raise ArgumentError, "invalid address length"
    end
    groups = []
    index = 0
    while index < 16
      groups.push("%04x" % (bytes[index] * 256 + bytes[index + 1]))
      index += 2
    end
    groups.join(":")
  end

  def initialize(addr = "::", family = Socket::AF_UNSPEC)
    unless addr.is_a?(String)
      unless [Socket::AF_INET, Socket::AF_INET6].include?(family)
        raise AddressFamilyError, "address family must be specified"
      end
      @family = family
      @addr = addr.to_i
      @mask_addr = family == Socket::AF_INET ? IN4MASK : IN6MASK
      return
    end
    prefix, prefixlen = addr.split("/", 2)
    if (/\A\[(.*)\]\z/ =~ prefix).nil? == false
      prefix = $1
      family = Socket::AF_INET6
    end
    @addr = nil
    @family = nil
    if family == Socket::AF_UNSPEC || family == Socket::AF_INET
      @addr = IPAddr.read_ipv4(prefix, false)
      @family = Socket::AF_INET unless @addr.nil?
    end
    if @addr.nil? && (family == Socket::AF_UNSPEC || family == Socket::AF_INET6)
      @addr = IPAddr.read_ipv6(prefix)
      @family = Socket::AF_INET6
    end
    if family != Socket::AF_UNSPEC && @family != family
      raise AddressFamilyError, "address family mismatch"
    end
    if prefixlen.nil?
      @mask_addr = @family == Socket::AF_INET ? IN4MASK : IN6MASK
    else
      self.mask!(prefixlen)
    end
  end

  # ── Reading text into a number ───────────────────────────────────────────

  def self.read_ipv4(text, strict = true)
    parts = text.split(".", -1)
    unless parts.length == 4
      raise InvalidAddressError, "invalid address: #{text}" if strict
      return nil
    end
    found = 0
    parts.each do |part|
      if (/\A\d+\z/ =~ part).nil? || part.to_i > 255
        raise InvalidAddressError, "invalid address: #{text}" if strict
        return nil
      end
      found = found * 256 + part.to_i
    end
    found
  end

  def self.read_ipv6(text)
    unless (/\A::ffff:(\d+\.\d+\.\d+\.\d+)\z/ =~ text).nil?
      return read_ipv4($1) | 0xffff00000000
    end
    unless (/\A::(\d+\.\d+\.\d+\.\d+)\z/ =~ text).nil?
      return read_ipv4($1)
    end
    # A dotted quad written at the end of an IPv6 address stands for the last
    # two groups.
    if (/\A(.*:)(\d+\.\d+\.\d+\.\d+)\z/ =~ text).nil? == false
      # Both captures are taken before the quad is read, since reading it
      # matches again and leaves the last match behind.
      opening = $1
      quad = read_ipv4($2)
      text = opening + ("%x" % (quad >> 16)) + ":" + ("%x" % (quad & 0xffff))
    end
    unless (/[^0-9a-fA-F:]/ =~ text).nil?
      raise InvalidAddressError, "invalid address: #{text}"
    end
    left = text
    right = ""
    unless (/\A(.*)::(.*)\z/ =~ text).nil?
      left = $1
      right = $2
    end
    front = left.split(":").reject { |part| part.empty? }
    back = right.split(":").reject { |part| part.empty? }
    rest = 8 - front.length - back.length
    if rest < 0 || (rest > 0 && (/::/ =~ text).nil?)
      raise InvalidAddressError, "invalid address: #{text}"
    end
    groups = front + ["0"] * rest + back
    found = 0
    groups.each do |group|
      raise InvalidAddressError, "invalid address: #{text}" if group.length > 4
      found = found * 65536 + group.to_i(16)
    end
    found
  end

  # ── What the address stands for ──────────────────────────────────────────

  def family
    @family
  end

  def to_i
    @addr
  end

  def prefix
    self.mask_length
  end

  def ipv4?
    @family == Socket::AF_INET
  end

  def ipv6?
    @family == Socket::AF_INET6
  end

  def ipv4_compat?
    self.ipv6? && (@addr >> 32) == 0 && @addr != 0 && @addr != 1
  end

  def ipv4_mapped?
    self.ipv6? && (@addr >> 32) == 0xffff
  end

  def native
    return self unless self.ipv4_compat? || self.ipv4_mapped?
    IPAddr.new(@addr & IN4MASK, Socket::AF_INET)
  end

  def ipv4_compat
    raise InvalidAddressError, "not an IPv4 address" unless self.ipv4?
    IPAddr.new(@addr, Socket::AF_INET6)
  end

  def ipv4_mapped
    raise InvalidAddressError, "not an IPv4 address" unless self.ipv4?
    IPAddr.new(@addr | 0xffff00000000, Socket::AF_INET6)
  end

  # ── Writing it back out ──────────────────────────────────────────────────

  def self.write_number(number, family)
    if family == Socket::AF_INET
      return [(number >> 24) & 0xff, (number >> 16) & 0xff, (number >> 8) & 0xff,
              number & 0xff].map { |one| one.to_s }.join(".")
    end
    groups = []
    index = 7
    while index >= 0
      groups.push("%04x" % ((number >> (index * 16)) & 0xffff))
      index -= 1
    end
    groups.join(":")
  end

  def to_string
    IPAddr.write_number(@addr, @family)
  end

  # The shortest form of the address, with the longest run of zero groups
  # written as `::`.
  def to_s
    written = self.to_string
    return written if self.ipv4?
    written = written.split(":").map do |group|
      trimmed = group.sub(/\A0+/, "")
      trimmed.empty? ? "0" : trimmed
    end.join(":")
    [7, 6, 5, 4, 3, 2].each do |count|
      run = (["0"] * count).join(":")
      pattern = Regexp.new("\\b" + run + "\\b")
      if written == "0:0:0:0:0:0:0:0"
        written = "::"
        break
      end
      unless (pattern =~ written).nil?
        written = written.sub(pattern, ":")
        break
      end
    end
    written = written.sub(/:{3,}/, "::")
    if (/\A::(ffff:)?([\da-fA-F]{1,4}):([\da-fA-F]{1,4})\z/ =~ written).nil? == false
      high = $2.to_i(16)
      low = $3.to_i(16)
      written = "::#{$1}#{high / 256}.#{high % 256}.#{low / 256}.#{low % 256}"
    end
    written
  end

  def inspect
    family = self.ipv4? ? "IPv4" : "IPv6"
    "#<#{self.class.name}: #{family}:#{self.to_string}/#{IPAddr.write_number(@mask_addr, @family)}>"
  end

  def hton
    written = ""
    if self.ipv4?
      [24, 16, 8, 0].each { |shift| written = written + ((@addr >> shift) & 0xff).chr }
      return written
    end
    index = 15
    while index >= 0
      written = written + ((@addr >> (index * 8)) & 0xff).chr
      index -= 1
    end
    written
  end

  # The name a reverse lookup of this address is made under.
  def reverse
    return self.ip6_arpa if self.ipv6?
    [(@addr & 0xff), ((@addr >> 8) & 0xff), ((@addr >> 16) & 0xff),
     ((@addr >> 24) & 0xff)].map { |one| one.to_s }.join(".") + ".in-addr.arpa"
  end

  def ip6_arpa
    raise InvalidAddressError, "not an IPv6 address" unless self.ipv6?
    self.nibbles + ".ip6.arpa"
  end

  def ip6_int
    raise InvalidAddressError, "not an IPv6 address" unless self.ipv6?
    self.nibbles + ".ip6.int"
  end

  def nibbles
    self.to_string.split(":").join.chars.reverse.join(".")
  end
  private :nibbles

  # ── Masks and arithmetic ─────────────────────────────────────────────────

  def mask_length
    counted = 0
    walked = @mask_addr
    width = self.ipv4? ? 32 : 128
    (0...width).each do |bit|
      counted += 1 unless (walked >> bit) & 1 == 0
    end
    counted
  end

  def mask(prefixlen)
    self.clone.mask!(prefixlen)
  end

  def mask!(prefixlen)
    if prefixlen.is_a?(String)
      if (/\A\d+\z/ =~ prefixlen).nil?
        other = IPAddr.new(prefixlen)
        unless other.family == @family
          raise InvalidPrefixError, "address family is not same"
        end
        @mask_addr = other.to_i
        @addr = @addr & @mask_addr
        return self
      end
      prefixlen = prefixlen.to_i
    end
    width = self.ipv4? ? 32 : 128
    if prefixlen < 0 || prefixlen > width
      raise InvalidPrefixError, "invalid length"
    end
    masklen = width - prefixlen
    whole = self.ipv4? ? IN4MASK : IN6MASK
    @mask_addr = (whole >> masklen) << masklen
    @addr = (@addr >> masklen) << masklen
    self
  end

  def clone
    IPAddr.new(self.to_string + "/" + IPAddr.write_number(@mask_addr, @family))
  end

  def with_address(number)
    made = self.clone
    made.instance_variable_set(:@addr, number & (self.ipv4? ? IN4MASK : IN6MASK))
    made
  end
  private :with_address

  def &(other)
    self.with_address(@addr & IPAddr.coerce_other(other, @family))
  end

  def |(other)
    self.with_address(@addr | IPAddr.coerce_other(other, @family))
  end

  def >>(count)
    self.with_address(@addr >> count)
  end

  def <<(count)
    self.with_address(@addr << count)
  end

  def ~
    self.with_address(~@addr)
  end

  def self.coerce_other(other, family)
    return other.to_i if other.is_a?(IPAddr)
    return other if other.is_a?(Integer)
    IPAddr.new(other.to_s).to_i
  end

  def ==(other)
    if other.is_a?(String)
      begin
        other = IPAddr.new(other)
      rescue IPAddr::Error
        return false
      end
    end
    return false unless other.is_a?(IPAddr)
    @family == other.family && @addr == other.to_i
  end

  def eql?(other)
    self == other
  end

  def hash
    [@addr, @mask_addr, @family].hash
  end

  def <=>(other)
    return nil unless other.is_a?(IPAddr)
    return nil unless @family == other.family
    @addr <=> other.to_i
  end

  def succ
    self.with_address(@addr + 1)
  end

  def include?(other)
    other = IPAddr.new(other, @family) unless other.is_a?(IPAddr)
    return false unless other.family == @family
    width = self.ipv4? ? 32 : 128
    masklen = width - self.mask_length
    (other.to_i >> masklen) == (@addr >> masklen)
  end

  def ===(other)
    self.include?(other)
  end

  def to_range
    width = self.ipv4? ? 32 : 128
    masklen = width - self.mask_length
    begins = (@addr >> masklen) << masklen
    ends = begins | ((1 << masklen) - 1)
    IPAddr.new(begins, @family)..IPAddr.new(ends, @family)
  end
end
