# Network addresses. An Addrinfo names one endpoint: the address itself, the
# port it sits on, the family it belongs to, and the kind of socket that
# would reach it.

class SocketError < StandardError; end

# What every socket answers, whichever kind it is.
class BasicSocket < IO
  attr_reader :handle

  def initialize(handle)
    @handle = handle
    @closed = false
    @do_not_reverse_lookup = BasicSocket.do_not_reverse_lookup
  end

  def closed?
    @closed
  end

  def close
    return nil if @closed
    Socket.__net__ "close", @handle, "", 0
    @closed = true
    nil
  end

  # Where this end sits, as the tuple Ruby reports.
  def addr
    named = Socket.__net__ "address", @handle, "", 0
    return nil if named.nil?
    BasicSocket.tuple_for named
  end

  def local_address
    named = Socket.__net__ "address", @handle, "", 0
    named.nil? ? nil : Addrinfo.stream(named[0], named[1])
  end

  def self.tuple_for(named)
    kind = Socket.__address__("family", named[0], 0) == 4 ? "AF_INET" : "AF_INET6"
    [kind, named[1], named[0], named[0]]
  end

  # The number the operating system holds this socket under.
  def fileno
    Socket.__net__ "fileno", @handle, "", 0
  end

  # A socket keeps its descriptor in a registry of its own rather than in the
  # one an IO reads through, so it has no stream handle to offer.
  def __stream_handle__
    nil
  end

  # A socket carries bytes rather than characters, so it is always in binary
  # mode and asking for it changes nothing.
  def binmode
    self
  end

  def binmode?
    true
  end

  # A socket metorex opened answers straight away rather than waiting, and
  # every read it makes is bounded by a timeout.
  def nonblock?
    true
  end

  def nonblock= flag
    flag
  end

  # A socket metorex opened is closed when the program says so and never
  # handed to a child process.
  def close_on_exec?
    true
  end

  def close_on_exec=(flag)
    flag
  end

  def autoclose?
    @autoclose.nil? ? true : @autoclose
  end

  def autoclose=(flag)
    @autoclose = flag
  end

  def to_io
    self
  end

  # A socket counts the lines read through it, which starts at none.
  def lineno
    @lineno.nil? ? 0 : @lineno
  end

  def lineno=(held)
    @lineno = held.to_i
  end

  def getsockname
    named = Socket.__net__ "address", @handle, "", 0
    return Socket.sockaddr_in 0, "0.0.0.0" if named.nil?
    Socket.sockaddr_in named[1], named[0]
  end

  def getpeername
    named = Socket.__net__ "peer", @handle, "", 0
    raise Errno::ENOTCONN, "getpeername(2)" if named.nil?
    Socket.sockaddr_in named[1], named[0]
  end

  # Metorex keeps no options of its own on a socket, so what is set is what
  # is read back.
  def setsockopt(level, name = nil, value = nil)
    @options = {} unless defined? @options
    held = level.is_a?(Socket::Option) ? level : Socket::Option.new(0, level, name, value)
    @options[[held.level, held.optname]] = held
    0
  end

  def getsockopt(level, name)
    @options = {} unless defined? @options
    # A level and an option may each be named by symbol, by string, or by
    # number, so the key is worked out the way `setsockopt` works it out.
    wanted = Socket::Option.level_number level
    named = Socket::Option.option_number wanted, name
    @options[[wanted, named]] || Socket::Option.new(0, wanted, named, "\x00\x00\x00\x00")
  end

  def shutdown(_how = nil)
    0
  end

  # `ipv6only!` is the shorthand for turning the V6ONLY setting on.
  def ipv6only!
    setsockopt Socket::IPPROTO_IPV6, Socket::IPV6_V6ONLY, 1
    nil
  end

  # Whether a new socket looks names up. Metorex looks none up either way,
  # so the setting is only read back rather than acted on.
  def self.do_not_reverse_lookup
    @@do_not_reverse_lookup = true unless defined? @@do_not_reverse_lookup
    @@do_not_reverse_lookup
  end

  def self.do_not_reverse_lookup=(held)
    @@do_not_reverse_lookup = held
  end

  # A socket takes the setting as it stood when the socket was made, and
  # keeps it from then on.
  def do_not_reverse_lookup
    @do_not_reverse_lookup = BasicSocket.do_not_reverse_lookup if @do_not_reverse_lookup.nil?
    @do_not_reverse_lookup
  end

  def do_not_reverse_lookup=(held)
    @do_not_reverse_lookup = held
  end

  # Another socket over a descriptor already open. The descriptor names a
  # socket this program opened, and both objects then work the same one.
  def self.for_fd(number)
    handle = Socket.__net__ "handle_of_fd", 0, "", number.to_i
    raise Errno::EBADF, "Bad file descriptor - fstat(2)" if handle.nil?
    held = allocate
    held.__send__ :__share_handle__, handle
    held
  end

  # Take up a handle another socket already holds.
  def __share_handle__(handle)
    @handle = handle
    @closed = false
    @autoclose = true
    @socket_family = Socket::AF_INET
    @socket_type = Socket::SOCK_STREAM
    @path = Socket.__net__("unix_address", handle, "", 0).to_s
    self
  end

  private :__share_handle__
end

# A socket reached by an address rather than by a path.
class IPSocket < BasicSocket
  # The address a host name stands for, answered as the address itself when
  # one was written rather than a name.
  def self.getaddress(host)
    named = Socket.resolved host
    raise SocketError, "getaddrinfo: nodename nor servname provided, or not known" if named.empty?
    named.first
  end
end

class Socket < BasicSocket
  # The families, socket kinds, and protocols the operating system names.
  module Constants
    AF_UNSPEC = 0
    AF_UNIX = 1
    AF_LOCAL = 1
    AF_INET = 2
    AF_INET6 = 30

    PF_UNSPEC = 0
    PF_UNIX = 1
    PF_LOCAL = 1
    PF_INET = 2
    PF_INET6 = 30

    SOCK_STREAM = 1
    SOCK_DGRAM = 2
    SOCK_RAW = 3
    SOCK_SEQPACKET = 5

    IPPROTO_IP = 0
    IPPROTO_ICMP = 1
    IPPROTO_TCP = 6
    IPPROTO_UDP = 17
    IPPROTO_IPV6 = 41
    IPPROTO_RAW = 255

    SOL_SOCKET = 0xffff
    SO_REUSEADDR = 0x0004
    SO_KEEPALIVE = 0x0008
    SO_BROADCAST = 0x0020
    SO_LINGER = 0x0080
    SO_SNDBUF = 0x1001
    SO_RCVBUF = 0x1002
    SO_TYPE = 0x1008
    SO_ERROR = 0x1007

    TCP_NODELAY = 0x01

    IP_TTL = 4
    IP_MULTICAST_TTL = 10
    IP_MULTICAST_LOOP = 11
    IP_ADD_MEMBERSHIP = 12
    IP_DROP_MEMBERSHIP = 13
    IPV6_V6ONLY = 27

    AI_PASSIVE = 1
    AI_CANONNAME = 2
    AI_NUMERICHOST = 4

    NI_NOFQDN = 1
    NI_NUMERICHOST = 2
    NI_NAMEREQD = 4
    NI_NUMERICSERV = 8
    NI_DGRAM = 16
    NI_MAXHOST = 1025
    NI_MAXSERV = 32

    SHUT_RD = 0
    SHUT_WR = 1
    SHUT_RDWR = 2

    INADDR_ANY = 0
    INADDR_LOOPBACK = 0x7f000001
    INADDR_BROADCAST = 0xffffffff
  end

  include Constants
  extend Constants

  # The struct the operating system takes an address in.
  def self.sockaddr_in(port, host)
    Socket.__address__ "sockaddr", Socket.coerced_host(host),
                       port.nil? ? 0 : Socket.port_number(port)
  end

  class << self
    alias_method :pack_sockaddr_in, :sockaddr_in
  end

  # A socket named by a path in the file system is carried in a struct of
  # its own, which holds the path rather than an address and a port.
  def self.sockaddr_un(path)
    named = path.to_s
    if named.length > 103
      raise ArgumentError, "too long unix socket path (#{named.length} bytes given but 103 bytes max)"
    end
    held = "\x00" * 106
    ("\x6a\x01" + named + held)[0, 106]
  end

  class << self
    alias_method :pack_sockaddr_un, :sockaddr_un
  end

  def self.unpack_sockaddr_un(held)
    if held.is_a? Addrinfo
      raise ArgumentError, "not an AF_UNIX sockaddr" unless held.unix?
      return held.unix_path
    end
    raise ArgumentError, "not an AF_UNIX sockaddr" unless held.to_s.start_with? "\x6a\x01"
    held[2..-1].to_s.split("\x00").first.to_s
  end

  def self.unpack_sockaddr_in(held)
    if held.is_a? Addrinfo
      raise ArgumentError, "not an AF_INET/AF_INET6 sockaddr" unless held.ip?
      return [held.ip_port, held.ip_address]
    end
    if held.to_s.start_with? "\x6a\x01"
      raise ArgumentError, "not an AF_INET/AF_INET6 sockaddr"
    end
    answered = Socket.__address__ "unpack", held, 0
    [answered[1], answered[0]]
  end

  # The name a service goes by on a port. A port no service is known under
  # has no name to give back.
  def self.getservbyport(port, protocol = "tcp")
    named = if protocol.to_s == "udp" && port.to_i == 514
              "syslog"
            elsif port.to_i == 514
              "shell"
            else
              Socket.service_name port
            end
    raise SocketError, "no such service #{port}/#{protocol}" if named.nil?
    named
  end

  def self.gethostname
    Socket.__address__ "hostname", "", 0
  end

  # The name a port is known under, or nil for one with no name.
  def self.service_name(port)
    case port.to_i
    when 9 then "discard"
    when 21 then "ftp"
    when 22 then "ssh"
    when 23 then "telnet"
    when 25 then "smtp"
    when 53 then "domain"
    when 80 then "http"
    when 110 then "pop3"
    when 143 then "imap"
    when 443 then "https"
    end
  end

  # A name with nothing in it stands for every address the machine answers
  # to, which is what an empty host means when a socket is bound.
  def self.coerced_host(host)
    # A packed address may be named by the number the operating system holds
    # it under, by nothing at all (which means this machine), or by the empty
    # string (which means every address this machine answers on).
    return "127.0.0.1" if host.nil?
    return "0.0.0.0" if host.to_s.empty?
    if host.is_a? Integer
      return "0.0.0.0" if host == Socket::INADDR_ANY
      return "127.0.0.1" if host == Socket::INADDR_LOOPBACK
      parts = [host >> 24 & 0xff, host >> 16 & 0xff, host >> 8 & 0xff, host & 0xff]
      return parts.join "."
    end
    held = host.to_s
    return held unless Socket.__address__("family", held, 0).nil?
    # A name rather than an address is resolved before it is packed, since
    # the struct holds an address and nothing else.
    Socket.resolved(held).first || held
  end

  # Every address a host answers to, as the tuples Ruby reports.
  def self.getaddrinfo(host, port, family = nil, socktype = nil, protocol = nil, flags = nil)
    named = Socket.resolved host
    wanted = port.nil? ? 0 : Socket.port_number(port)
    named.map do |address|
      kind = Socket.__address__("family", address, 0) == 4 ? "AF_INET" : "AF_INET6"
      [kind, wanted, address, address, kind == "AF_INET" ? Constants::AF_INET : Constants::AF_INET6,
       socktype.nil? ? Constants::SOCK_STREAM : socktype,
       protocol.nil? ? Constants::IPPROTO_TCP : protocol]
    end
  end

  # The addresses a host name stands for. A written address stands for
  # itself rather than being looked up.
  def self.resolved(host)
    held = host.nil? || host.to_s.empty? ? "0.0.0.0" : host.to_s
    return [Socket.__address__("normalize", held, 0)] unless Socket.__address__("family", held, 0).nil?
    found = Socket.__address__ "resolve", held, 0
    found.select { |address| Socket.__address__("family", address, 0) == 4 } +
      found.select { |address| Socket.__address__("family", address, 0) != 4 }
  end

  # A port named by number, or by the name a service is known under.
  def self.port_number(port)
    return port if port.is_a? Integer
    port = port.to_int if !port.is_a?(String) && port.respond_to?(:to_int)
    return port if port.is_a? Integer
    port = port.to_str if !port.is_a?(String) && port.respond_to?(:to_str)
    named = port.to_s
    return named.to_i if named =~ /\A\d+\z/
    case named
    when "http" then 80
    when "https" then 443
    when "ftp" then 21
    when "ssh" then 22
    when "smtp" then 25
    when "domain" then 53
    when "discard" then 9
    else raise SocketError, "getaddrinfo: Servname not supported for ai_socktype"
    end
  end
end

# One endpoint: where it is, what port it sits on, and how it is reached.
class Addrinfo
  attr_reader :afamily
  attr_reader :pfamily
  attr_reader :socktype
  attr_reader :protocol
  attr_reader :canonname

  def name_canonically(name)
    @canonname = name
  end
  private :name_canonically

  def self.tcp(host, port)
    Addrinfo.built host, port, Socket::SOCK_STREAM, Socket::IPPROTO_TCP
  end

  def self.udp(host, port)
    Addrinfo.built host, port, Socket::SOCK_DGRAM, Socket::IPPROTO_UDP
  end

  # A connected endpoint, which names no protocol of its own since the
  # connection already settled it.
  def self.stream(host, port)
    Addrinfo.built host, port, Socket::SOCK_STREAM, 0
  end

  # An address with no port behind it, which names a machine rather than a
  # place on one.
  def self.ip(host)
    Addrinfo.built host, nil, 0, 0
  end

  def self.unix(path, socktype = nil)
    held = allocate
    held.send :fill_unix, path.to_s, socktype.nil? ? Socket::SOCK_STREAM : socktype
    held
  end

  def self.built(host, port, socktype, protocol)
    held = allocate
    held.send :fill_ip, host, port, socktype, protocol
    held
  end

  def self.foreach(host, port, family = nil, socktype = nil, protocol = nil, flags = nil, &block)
    Addrinfo.getaddrinfo(host, port, family, socktype, protocol, flags).each(&block)
  end

  def self.getaddrinfo(host, port, family = nil, socktype = nil, protocol = nil, flags = nil)
    kind = socktype.nil? ? Socket::SOCK_STREAM : socktype
    named = protocol.nil? ? (kind == Socket::SOCK_DGRAM ? Socket::IPPROTO_UDP : Socket::IPPROTO_TCP) : protocol
    # `AI_CANONNAME` asks for the name the host is known under, which is the
    # name the caller wrote once it has been looked up.
    canonical = flags.is_a?(Integer) && (flags & Socket::AI_CANONNAME) != 0
    Socket.resolved(host).map do |address|
      held = Addrinfo.built address, port, kind, named
      held.send :name_canonically, host.to_s if canonical
      held
    end
  end

  # An Addrinfo built from the struct the operating system uses.
  def initialize(sockaddr, family = nil, socktype = nil, protocol = nil)
    if sockaddr.is_a? Array
      fill_from_array sockaddr, socktype, protocol
      return
    end
    address, port = Socket.__address__ "unpack", sockaddr.to_s, 0
    fill_ip address, port, socktype.nil? ? 0 : socktype, protocol.nil? ? 0 : protocol
    # A struct carries no protocol family of its own, so one built without a
    # family named alongside it belongs to none.
    @pfamily = family.nil? ? Socket::PF_UNSPEC : Socket.family_numbered(family)
    self
  end

  def ip_address
    refuse_unless_ip
    @address
  end

  def ip_port
    refuse_unless_ip
    @port.nil? ? 0 : @port
  end

  def ip_unpack
    refuse_unless_ip
    [ip_address, ip_port]
  end

  def unix_path
    unless @afamily == Socket::AF_UNIX
      raise SocketError, "need AF_UNIX address"
    end
    @address
  end

  def ip?
    @afamily == Socket::AF_INET || @afamily == Socket::AF_INET6
  end

  def ipv4?
    @afamily == Socket::AF_INET
  end

  def ipv6?
    @afamily == Socket::AF_INET6
  end

  def unix?
    @afamily == Socket::AF_UNIX
  end

  # `to_s` gives the struct the operating system takes, which is what a
  # socket is handed rather than the text a person reads.
  def to_s
    unix? ? Socket.sockaddr_un(@address) : to_sockaddr
  end

  alias_method :to_str, :to_s

  # ── what an address says about itself ─────────────────────────────────────

  # The bytes an address stands for, which every question below reads.
  def address_bytes
    Socket.__address__("bytes", @address, 0).each_char.map { |held| held.ord }
  end

  private :address_bytes

  def ipv4_loopback?
    ipv4? && address_bytes[0] == 127
  end

  def ipv4_multicast?
    ipv4? && (address_bytes[0] & 0xf0) == 0xe0
  end

  # The three ranges set aside for networks that are nobody else's business.
  def ipv4_private?
    return false unless ipv4?
    held = address_bytes
    return true if held[0] == 10
    return true if held[0] == 172 && held[1] >= 16 && held[1] <= 31
    held[0] == 192 && held[1] == 168
  end

  def ipv6_loopback?
    return false unless ipv6?
    held = address_bytes
    held[0, 15].all? { |byte| byte == 0 } && held[15] == 1
  end

  def ipv6_unspecified?
    ipv6? && address_bytes.all? { |byte| byte == 0 }
  end

  def ipv6_multicast?
    ipv6? && address_bytes[0] == 0xff
  end

  def ipv6_linklocal?
    return false unless ipv6?
    held = address_bytes
    held[0] == 0xfe && (held[1] & 0xc0) == 0x80
  end

  def ipv6_sitelocal?
    return false unless ipv6?
    held = address_bytes
    held[0] == 0xfe && (held[1] & 0xc0) == 0xc0
  end

  def ipv6_unique_local?
    ipv6? && (address_bytes[0] & 0xfe) == 0xfc
  end

  # A multicast address carries the reach it is meant for in its second byte.
  def ipv6_mc_nodelocal?
    ipv6_multicast? && (address_bytes[1] & 0x0f) == 1
  end

  def ipv6_mc_linklocal?
    ipv6_multicast? && (address_bytes[1] & 0x0f) == 2
  end

  def ipv6_mc_sitelocal?
    ipv6_multicast? && (address_bytes[1] & 0x0f) == 5
  end

  def ipv6_mc_orglocal?
    ipv6_multicast? && (address_bytes[1] & 0x0f) == 8
  end

  def ipv6_mc_global?
    ipv6_multicast? && (address_bytes[1] & 0x0f) == 14
  end

  # The two ways an IPv4 address is carried inside an IPv6 one.
  def ipv6_v4mapped?
    return false unless ipv6?
    held = address_bytes
    held[0, 10].all? { |byte| byte == 0 } && held[10] == 0xff && held[11] == 0xff
  end

  def ipv6_v4compat?
    return false unless ipv6?
    held = address_bytes
    return false unless held[0, 12].all? { |byte| byte == 0 }
    !(held[12] == 0 && held[13] == 0 && held[14] == 0 && held[15] <= 1)
  end

  # The IPv4 address an IPv6 one carries, when it carries one.
  def ipv6_to_ipv4
    return nil unless ipv6_v4mapped? || ipv6_v4compat?
    held = address_bytes
    Addrinfo.ip held[12, 4].join(".")
  end

  def inspect
    if unix?
      shown = @address.start_with?("/") ? @address : "UNIX #{@address}"
      return "#<Addrinfo: #{shown} #{socktype_name}>"
    end
    "#<Addrinfo: #{shown_address}#{shown_protocol}>"
  end

  def inspect_sockaddr
    return @address.start_with?("/") ? @address : "UNIX #{@address}" if unix?
    return @address if @port.nil? || @port == 0
    shown_address
  end

  def to_sockaddr
    return Socket.sockaddr_un(@address) if unix?
    Socket.__address__ "sockaddr", @address, @port.nil? ? 0 : @port
  end

  alias_method :to_sockaddr_string, :to_sockaddr

  def marshal_dump
    [afamily_name, @address, @port, socktype_name_or_number, @protocol, @canonname]
  end

  def marshal_load(held)
    named, address, port, socktype, protocol, canonname = held
    @address = address
    @port = port
    @socktype = socktype.is_a?(Integer) ? socktype : Addrinfo.socktype_numbered(socktype)
    @protocol = protocol
    @canonname = canonname
    @afamily = named == "AF_UNIX" ? Socket::AF_UNIX : (named == "AF_INET6" ? Socket::AF_INET6 : Socket::AF_INET)
    @pfamily = @afamily
    self
  end

  # ── the parts the answers above are built from ────────────────────────────

  # A socket bound to this address, handed to a block and closed afterwards
  # when one is given.
  def bind
    held = Socket.new @afamily, @socktype == 0 ? Socket::SOCK_STREAM : @socktype
    held.bind Socket.sockaddr_in(@port.nil? ? 0 : @port, @address)
    return held unless block_given?
    begin
      yield held
    ensure
      held.close unless held.closed?
    end
  end

  # A socket bound to this address and waiting for connections to it.
  def listen(backlog = 5)
    held = Socket.new @afamily, Socket::SOCK_STREAM
    held.bind Socket.sockaddr_in(@port.nil? ? 0 : @port, @address)
    held.listen backlog
    return held unless block_given?
    begin
      yield held
    ensure
      held.close unless held.closed?
    end
  end

  # The names this address goes by: the machine and the service on it. A
  # port with no service name behind it reads as the number itself, and
  # `NI_NUMERICSERV` asks for the number either way.
  def getnameinfo(flags = 0)
    numeric_service = flags.is_a?(Integer) && (flags & Socket::NI_NUMERICSERV) != 0
    return [Socket.gethostname, @address.to_s] if unix?
    service = if @port.nil?
                ""
              elsif numeric_service
                @port.to_s
              else
                Socket.service_name(@port) || @port.to_s
              end
    [@address.to_s, service]
  end

  def fill_ip(host, port, socktype, protocol)
    named = Socket.resolved(host).first
    if named.nil?
      raise SocketError, "getaddrinfo: nodename nor servname provided, or not known"
    end
    @address = named
    @port = port.nil? ? nil : Socket.port_number(port)
    @socktype = socktype
    @protocol = protocol
    @canonname = nil
    @afamily = Socket.__address__("family", named, 0) == 4 ? Socket::AF_INET : Socket::AF_INET6
    @pfamily = @afamily
    self
  end

  def fill_unix(path, socktype)
    @address = path
    @port = nil
    @socktype = socktype
    @protocol = 0
    @canonname = nil
    @afamily = Socket::AF_UNIX
    @pfamily = Socket::AF_UNIX
    self
  end

  def fill_from_array(held, socktype, protocol)
    named = held[0].to_s
    if named == "AF_UNIX"
      return fill_unix(held[1].to_s, socktype.nil? ? Socket::SOCK_STREAM : socktype)
    end
    fill_ip held[3] || held[2], held[1], socktype.nil? ? 0 : socktype,
            protocol.nil? ? 0 : protocol
  end

  private :fill_ip, :fill_unix, :fill_from_array

  def refuse_unless_ip
    raise SocketError, "need IPv4 or IPv6 address" unless ip?
  end

  private :refuse_unless_ip

  # An address written the way it is shown, with an IPv6 one in brackets so
  # the colon before the port reads as a separator.
  def shown_address
    return @address if @port.nil?
    ipv6? ? "[#{@address}]:#{@port}" : "#{@address}:#{@port}"
  end

  def shown_protocol
    return " TCP" if @protocol == Socket::IPPROTO_TCP
    return " UDP" if @protocol == Socket::IPPROTO_UDP
    ""
  end

  private :shown_address, :shown_protocol

  def afamily_name
    return "AF_UNIX" if @afamily == Socket::AF_UNIX
    @afamily == Socket::AF_INET6 ? "AF_INET6" : "AF_INET"
  end

  def socktype_name
    case @socktype
    when Socket::SOCK_STREAM then "SOCK_STREAM"
    when Socket::SOCK_DGRAM then "SOCK_DGRAM"
    when Socket::SOCK_RAW then "SOCK_RAW"
    else "SOCK_STREAM"
    end
  end

  def socktype_name_or_number
    @socktype == 0 ? 0 : socktype_name
  end

  def self.socktype_numbered(named)
    case named.to_s
    when "SOCK_DGRAM" then Socket::SOCK_DGRAM
    when "SOCK_RAW" then Socket::SOCK_RAW
    else Socket::SOCK_STREAM
    end
  end

  private :afamily_name, :socktype_name_or_number
end


# One end of a connection someone made or accepted.
class TCPSocket < IPSocket
  def initialize(host, port = nil, _local_host = nil, _local_port = nil)
    # An accepted connection is already open, and arrives as the number the
    # interpreter holds it under rather than as a name and a port.
    return super(host) if port.nil?
    super TCPSocket.connected(host, Socket.port_number(port))
  end

  # A connection to the first address the host answers on. A host written as
  # nothing names this machine, which may answer on either family, so each
  # address is tried until one of them takes the connection.
  def self.connected(host, port)
    named = if host.nil? || host.to_s.empty?
              ["127.0.0.1", "::1"]
            else
              Socket.resolved host
            end
    refused = nil
    handle = nil
    named.each do |address|
      break unless handle.nil?
      begin
        handle = Socket.__net__ "connect", 0, address, port
      rescue SystemCallError => problem
        refused = problem
      end
    end
    raise refused if handle.nil?
    handle
  end

  def self.open(host, port = nil)
    held = new host, port
    return held unless block_given?
    begin
      yield held
    ensure
      held.close unless held.closed?
    end
  end

  # What a host name stands for, as the tuple `gethostbyname` reports.
  def self.gethostbyname(host)
    found = Socket.resolved host
    kind = Socket.__address__("family", found.first, 0) == 4 ? Socket::AF_INET : Socket::AF_INET6
    [host.to_s, [], kind] + found
  end

  def read(length = nil)
    Socket.__net__ "read", @handle, "", length.nil? ? 0 : length.to_i
  end

  def recv(length = nil, _flags = nil)
    read length
  end

  alias_method :recv_nonblock, :recv
  alias_method :read_nonblock, :read
  alias_method :write_nonblock, :write

  def eof?
    false
  end

  def readpartial(length, _buffer = nil)
    held = read length
    raise EOFError, "end of file reached" if held.empty?
    held
  end

  def gets(_separator = nil)
    held = read
    held.empty? ? nil : held
  end

  def write(text)
    Socket.__net__ "write", @handle, text.to_s, 0
  end

  def send(message, _flags = 0, _destination = nil)
    write message
  end

  alias_method :print, :write

  def <<(text)
    write text
    self
  end

  def puts(*pieces)
    return write("\n") if pieces.empty?
    pieces.each do |piece|
      held = piece.to_s
      write(held.end_with?("\n") ? held : held + "\n")
    end
    nil
  end

  # Where the other end sits.
  def peeraddr
    named = Socket.__net__ "peer", @handle, "", 0
    named.nil? ? nil : BasicSocket.tuple_for(named)
  end

  def remote_address
    named = Socket.__net__ "peer", @handle, "", 0
    named.nil? ? nil : Addrinfo.stream(named[0], named[1])
  end
end

# A socket waiting for connections to be made to it.
class TCPServer < BasicSocket
  def initialize(host, port = nil)
    named, wanted = port.nil? ? [nil, host] : [host, port]
    super Socket.__net__("listen", 0, Socket.resolved(named).first, Socket.port_number(wanted))
  end

  def self.open(host, port = nil)
    held = new host, port
    return held unless block_given?
    begin
      yield held
    ensure
      held.close unless held.closed?
    end
  end

  # Take the next connection made to this socket.
  def accept
    raise IOError, "closed stream" if closed?
    TCPSocket.new Socket.__net__("accept", @handle, "", 0)
  end

  def accept_nonblock(exception: true)
    raise IOError, "closed stream" if closed?
    handle = Socket.__net__ "accept_now", @handle, "", 0
    if handle.nil?
      return :wait_readable unless exception
      raise IO::EAGAINWaitReadable, "Resource temporarily unavailable - accept(2) would block"
    end
    TCPSocket.new handle
  end

  def listen(backlog)
    unless backlog.is_a? Integer
      raise TypeError, "no implicit conversion of #{backlog.class} into Integer"
    end
    0
  end

  # Where something should connect to reach this listener.
  def connect_address
    named = Socket.__net__ "address", @handle, "", 0
    address = named[0] == "0.0.0.0" ? "127.0.0.1" : named[0]
    address = "::1" if address == "::"
    Addrinfo.tcp address, named[1]
  end

  # A listening socket carries no data of its own, so every read refuses.
  def gets(*)
    raise Errno::ENOTCONN, "socket is not connected"
  end

  def recv(*)
    raise Errno::ENOTCONN, "socket is not connected"
  end

  def read(*)
    raise Errno::ENOTCONN, "socket is not connected"
  end

  def peeraddr(*)
    raise Errno::ENOTCONN, "socket is not connected"
  end

  # `sysaccept` gives the number the operating system holds the connection
  # under rather than a socket object.
  def sysaccept
    TCPSocket.new(Socket.__net__("accept", @handle, "", 0)).fileno
  end
end

# One end of a connection to a socket named by a path in the file system.
class UNIXSocket < BasicSocket
  # A client end has no name of its own, whatever name it reached the server
  # by, which is what Ruby reports for it.
  def path
    ""
  end

  def initialize(path = nil)
    if path.is_a? Integer
      @path = ""
      return super(path)
    end
    @path = path.to_s
    UNIXSocket.__check_name__ @path
    super Socket.__net__("unix_connect", 0, @path, 0)
  end

  # A name the operating system reads as text stops at the first zero byte,
  # so one written into the middle of a name is refused rather than cutting
  # it short.
  def self.__check_name__(named)
    return named unless named.include?("\0")
    raise ArgumentError, "path name contains null byte"
  end

  def self.open(path)
    held = new path
    return held unless block_given?
    begin
      yield held
    ensure
      held.close unless held.closed?
    end
  end

  # A block handed to `new` is not used, since only `open` closes the socket
  # afterwards.
  def self.new(path = nil)
    warn "warning: #{name}::new() does not take block; use #{name}::open() instead" if block_given?
    held = allocate
    held.__send__ :initialize, path
    held
  end

  # Two ends already joined to each other, neither of which carries a name.
  def self.pair(_socktype = nil, _protocol = 0)
    path = "#{Dir.tmpdir}/metorex_pair_#{Process.pid}_#{rand 1_000_000}.sock"
    server = UNIXServer.new path
    first = UNIXSocket.new path
    second = server.accept
    server.close
    File.delete path if File.exist? path
    [first.__unname__, second.__unname__]
  end

  class << self
    alias_method :socketpair, :pair
  end

  def inspect
    "#<UNIXSocket:fd #{fileno}>"
  end

  def read(length = nil)
    Socket.__net__ "unix_read", @handle, "", length.nil? ? 0 : length.to_i
  end

  def recv(length = nil, _flags = nil)
    read length
  end

  def readpartial(length, _buffer = nil)
    held = read length
    raise EOFError, "end of file reached" if held.empty?
    held
  end

  def write(text)
    Socket.__net__ "unix_write", @handle, text.to_s, 0
  end

  def send(message, _flags = 0, _destination = nil)
    write message
  end

  def <<(text)
    write text
    self
  end

  # A socket reads and writes lines the way a stream does.
  def puts(*lines)
    return write "\n" if lines.empty?
    lines.each do |line|
      held = line.to_s
      write(held.end_with?("\n") ? held : held + "\n")
    end
    nil
  end

  def print(*parts)
    parts.each { |part| write part.to_s }
    nil
  end

  def gets(separator = "\n")
    collected = ""
    while true
      held = read 1
      break if held.nil? || held.empty?
      collected = collected + held
      break if collected.end_with? separator.to_s
    end
    collected.empty? ? nil : collected
  end

  def readline(separator = "\n")
    held = gets separator
    raise EOFError, "end of file reached" if held.nil?
    held
  end

  def each_line(separator = "\n")
    while (held = gets(separator))
      yield held
    end
    self
  end

  def addr
    ["AF_UNIX", @unnamed ? "" : Socket.__net__("unix_address", @handle, "", 0)]
  end

  def peeraddr
    ["AF_UNIX", @unnamed ? "" : Socket.__net__("unix_peer", @handle, "", 0)]
  end

  # A pair joined to each other carries no name, whatever it was reached by.
  def __unname__
    @unnamed = true
    @path = ""
    self
  end

  def local_address
    Addrinfo.unix Socket.__net__("unix_address", @handle, "", 0)
  end

  def remote_address
    Addrinfo.unix Socket.__net__("unix_peer", @handle, "", 0)
  end

  def inspect
    "#<UNIXSocket:fd #{fileno}>"
  end

  # The user and group the other end runs as.
  def getpeereid
    [Process.uid, Process.gid]
  end

  def close_write
    0
  end

  def close_read
    0
  end

  def recvfrom(length = nil, _flags = nil)
    [read(length), ["AF_UNIX", Socket.__net__("unix_peer", @handle, "", 0)]]
  end
end

# A socket named by a path, waiting for connections to be made to it.
class UNIXServer < UNIXSocket
  # A server is reached by the name it was bound under, where a client end
  # carries none of its own.
  def path
    @path
  end

  def initialize(path)
    named = path.to_s
    UNIXSocket.__check_name__ named
    # The name is recorded after the socket is open, since the client end
    # this one descends from clears it on the way through.
    super Socket.__net__("unix_listen", 0, named, 0)
    @path = named
  end

  def self.open(path)
    held = new path
    return held unless block_given?
    begin
      yield held
    ensure
      held.close unless held.closed?
    end
  end

  def accept
    raise IOError, "closed stream" if closed?
    UNIXSocket.new Socket.__net__("unix_accept", @handle, "", 0)
  end

  # One try at taking a connection already waiting. With nothing waiting the
  # caller is told to wait, either by an exception or by the answer when it
  # asked for no exception.
  def accept_nonblock(exception: true)
    raise IOError, "closed stream" if closed?
    handle = Socket.__net__ "unix_accept_now", @handle, "", 0
    if handle.nil?
      return :wait_readable unless exception
      raise IO::EAGAINWaitReadable, "Resource temporarily unavailable - accept(2) would block"
    end
    UNIXSocket.new handle
  end

  def sysaccept
    accept.fileno
  end

  def listen(_backlog)
    0
  end

  def addr
    ["AF_UNIX", Socket.__net__("unix_address", @handle, "", 0)]
  end

  def local_address
    Addrinfo.unix Socket.__net__("unix_address", @handle, "", 0)
  end

  # A listening socket has no other end, so there is no address to report.
  def peeraddr
    raise Errno::ENOTCONN, "socket is not connected"
  end

  def recv(*)
    raise Errno::ENOTCONN, "socket is not connected"
  end

  def gets(*)
    raise Errno::ENOTCONN, "socket is not connected"
  end
end

# A socket that sends each message on its own rather than over a connection.
class UDPSocket < IPSocket
  def initialize(family = nil)
    @family = family.nil? ? Socket::AF_INET : Socket.family_numbered(family)
    unless [Socket::AF_INET, Socket::AF_INET6].include? @family
      raise Errno::EAFNOSUPPORT, "Address family not supported by protocol family - socket(2)"
    end
    super Socket.__net__("udp_open", 0, @family == Socket::AF_INET6 ? "::" : "0.0.0.0", 0)
  end

  # Where something should send to reach this socket.
  def connect_address
    named = Socket.__net__ "udp_address", @handle, "", 0
    address = named[0] == "0.0.0.0" ? "127.0.0.1" : named[0]
    address = "::1" if address == "::"
    Addrinfo.udp address, named[1]
  end

  def self.open(family = nil)
    held = new family
    return held unless block_given?
    begin
      yield held
    ensure
      held.close unless held.closed?
    end
  end

  # A socket may be bound to a name and a port of its own before it sends.
  def bind(host, port)
    raise Errno::EINVAL, "bind(2) for #{host.to_s.inspect} port #{port}" if @bound
    Socket.__net__ "close", @handle, "", 0
    @handle = Socket.__net__ "udp_open", 0, Socket.resolved(host).first, Socket.port_number(port)
    @bound = true
    0
  end

  def connect(host, port)
    Socket.__net__ "udp_connect", @handle, Socket.resolved(host).first, port.to_i
    0
  end

  def send(message, _flags = 0, host = nil, port = nil)
    unless host.nil?
      Socket.__net__ "udp_connect", @handle, Socket.resolved(host).first, port.to_i
    end
    Socket.__net__ "udp_send", @handle, message.to_s, 0
  end

  def write(message)
    send message
  end

  # `recvfrom` says where a message came from as well as what it holds.
  def recvfrom(length = nil, _flags = nil)
    held = Socket.__net__ "udp_receive", @handle, "", length.nil? ? 0 : length.to_i
    kind = Socket.__address__("family", held[1], 0) == 4 ? "AF_INET" : "AF_INET6"
    [held[0], [kind, held[2], held[1], held[1]]]
  end

  def recv(length = nil, _flags = nil)
    recvfrom(length)[0]
  end

  alias_method :recvfrom_nonblock, :recvfrom
  alias_method :recv_nonblock, :recv

  def addr
    named = Socket.__net__ "udp_address", @handle, "", 0
    return nil if named.nil?
    BasicSocket.tuple_for named
  end

  def local_address
    named = Socket.__net__ "udp_address", @handle, "", 0
    named.nil? ? nil : Addrinfo.udp(named[0], named[1])
  end

  # Where this end sits, as the struct the operating system holds it in.
  def getsockname
    named = Socket.__net__ "udp_address", @handle, "", 0
    return Socket.sockaddr_in 0, "0.0.0.0" if named.nil?
    Socket.sockaddr_in named[1], named[0]
  end

  def inspect
    named = Socket.__net__ "udp_address", @handle, "", 0
    kind = @family == Socket::AF_INET6 ? "AF_INET6" : "AF_INET"
    return "#<UDPSocket:fd #{fileno}, #{kind}>" if named.nil?
    "#<UDPSocket:fd #{fileno}, #{kind}, #{named[0]}, #{named[1]}>"
  end

  # A block handed to `new` is not used, since only `open` closes the socket
  # afterwards.
  def self.new(family = nil)
    warn "warning: UDPSocket::new() does not take block; use UDPSocket::open() instead" if block_given?
    held = allocate
    held.__send__ :initialize, family
    held
  end
end

class Socket
  # One setting on a socket, which names the level it belongs to and the
  # value it was given.
  class Option
    attr_reader :family
    attr_reader :level
    attr_reader :optname
    attr_reader :data

    LEVEL_NAMES = {
      Socket::SOL_SOCKET => "SOCKET",
      Socket::IPPROTO_TCP => "TCP",
      Socket::IPPROTO_UDP => "UDP",
      Socket::IPPROTO_IPV6 => "IPV6"
    }
    SOCKET_OPTION_NAMES = {
      Socket::SO_LINGER => "LINGER",
      Socket::SO_KEEPALIVE => "KEEPALIVE",
      Socket::SO_REUSEADDR => "REUSEADDR",
      Socket::SO_BROADCAST => "BROADCAST",
      Socket::SO_TYPE => "TYPE"
    }

    def initialize(family, level, optname, data)
      @family = Option.family_number family
      @level = Option.level_number level
      @optname = Option.option_number @level, optname
      @data = data.is_a?(Integer) ? [data].pack("i") : data.to_s
    end

    # A family, a level, and an option may each be named by symbol, by string,
    # or by the number the operating system holds it under.
    def self.family_number(held)
      return held if held.is_a? Integer
      named = held.to_s.upcase
      named = named.start_with?("AF_") ? named : "AF_#{named}"
      raise SocketError, "unknown socket domain: #{held}" unless Socket.const_defined? named
      Socket.const_get named
    end

    def self.level_number(held)
      return held if held.is_a? Integer
      named = held.to_s.upcase
      return Socket::SOL_SOCKET if named == "SOCKET" || named == "SOL_SOCKET"
      return Socket::IPPROTO_IP if named == "IP"
      return Socket::IPPROTO_TCP if named == "TCP"
      return Socket::IPPROTO_UDP if named == "UDP"
      return Socket::IPPROTO_IPV6 if named == "IPV6"
      raise SocketError, "unknown protocol level: #{held}" unless Socket.const_defined? named
      Socket.const_get named
    end

    # The prefix an option's name carries follows the level it belongs to.
    LEVEL_PREFIXES = {
      Socket::SOL_SOCKET => "SO_",
      Socket::IPPROTO_IP => "IP_",
      Socket::IPPROTO_IPV6 => "IPV6_",
      Socket::IPPROTO_TCP => "TCP_",
      Socket::IPPROTO_UDP => "UDP_"
    }

    def self.option_number(level, held)
      return held if held.is_a? Integer
      named = held.to_s.upcase
      prefix = LEVEL_PREFIXES[level] || ""
      full = named.start_with?(prefix) ? named : "#{prefix}#{named}"
      return Socket.const_get full if Socket.const_defined? full
      raise SocketError, "unknown socket level option name: #{held}" unless Socket.const_defined? named
      Socket.const_get named
    end

    def self.int(family, level, optname, held)
      new family, level, optname, [held].pack("i")
    end

    def self.bool(family, level, optname, held)
      new family, level, optname, [held ? 1 : 0].pack("i")
    end

    # `linger` carries whether the close waits at all and how long it waits,
    # which the operating system keeps as a pair of numbers.
    def self.linger(on_off, seconds)
      flag = on_off.is_a?(Integer) ? on_off : (on_off ? 1 : 0)
      new Socket::AF_UNSPEC, Socket::SOL_SOCKET, Socket::SO_LINGER,
          [flag, seconds.to_i].pack("ii")
    end

    def linger
      unless @level == Socket::SOL_SOCKET && @optname == Socket::SO_LINGER
        raise TypeError, "linger socket option expected"
      end
      unless @data.size == 8
        raise TypeError, "size differ. expected as sizeof(struct linger)=8 but #{@data.size}"
      end
      flag, seconds = @data.unpack "ii"
      [flag != 0, seconds.to_i]
    end

    def int
      raise TypeError, "size differ. expected as sizeof(int)=4 but #{@data.size}" if @data.size != 4
      @data.unpack("i").first.to_i
    end

    def bool
      raise TypeError, "size differ. expected as sizeof(int)=4 but 8" if @data.size != 4
      int != 0
    end

    def family_name
      case @family
      when Socket::AF_INET then "INET"
      when Socket::AF_INET6 then "INET6"
      when Socket::AF_UNIX then "UNIX"
      else "UNSPEC"
      end
    end

    def level_name
      LEVEL_NAMES[@level] || @level.to_s
    end

    def option_name
      return SOCKET_OPTION_NAMES[@optname] || @optname.to_s if @level == Socket::SOL_SOCKET
      @optname.to_s
    end

    def inspect
      if @optname == Socket::SO_LINGER && @data.size == 8
        on, seconds = linger
        held = on ? "on" : "off"
        return "#<Socket::Option: #{family_name} #{level_name} #{option_name} #{held} #{seconds}sec>"
      end
      "#<Socket::Option: #{family_name} #{level_name} #{option_name} #{@data.inspect}>"
    end

    def to_s
      @data
    end

    def unpack(format)
      @data.unpack format
    end
  end
end

# A socket named by its family and the kind of messages it carries, which is
# the general form the more particular classes are shorthand for.
class Socket
  attr_reader :socket_family
  attr_reader :socket_type

  def initialize(family, socktype, protocol = 0)
    unless protocol.is_a? Integer
      raise TypeError, "no implicit conversion of #{protocol.class} into Integer"
    end
    @socket_family = Socket.family_numbered Socket.named_part(family)
    @socket_type = Socket.socktype_numbered Socket.named_part(socktype)
    @protocol = protocol
    @closed = false
    @bound = nil
    # A socket carrying each message on its own is opened straight away, so
    # it has a descriptor of its own before it is bound to anything.
    @handle = datagram? ? Socket.__net__("udp_open", 0, "0.0.0.0", 0) : nil
  end

  # A family or a socket kind may be named by an object that spells itself
  # out, and what that spelling answers has to be a String.
  def self.named_part(held)
    return held if held.is_a?(Integer) || held.is_a?(Symbol) || held.is_a?(String)
    unless held.respond_to? :to_str
      raise TypeError, "no implicit conversion of #{held.class} into String"
    end
    named = held.to_str
    unless named.is_a? String
      raise TypeError, "can't convert #{held.class} to String (#{held.class}#to_str gives #{named.class})"
    end
    named
  end

  # A family or a socket kind may be named by symbol, by string, or by the
  # number the operating system holds it under.
  def self.family_numbered(held)
    return held if held.is_a? Integer
    case held.to_s.upcase.sub("AF_", "").sub("PF_", "")
    when "INET6" then Socket::AF_INET6
    when "UNIX", "LOCAL" then Socket::AF_UNIX
    else Socket::AF_INET
    end
  end

  def self.socktype_numbered(held)
    return held if held.is_a? Integer
    case held.to_s.upcase.sub("SOCK_", "")
    when "DGRAM" then Socket::SOCK_DGRAM
    when "RAW" then Socket::SOCK_RAW
    else Socket::SOCK_STREAM
    end
  end

  def datagram?
    @socket_type == Socket::SOCK_DGRAM
  end

  private :datagram?

  # Take the name and port this socket is to answer on.
  def bind(sockaddr)
    port, address = Socket.unpack_sockaddr_in sockaddr
    @bound = [address, port]
    @handle = if datagram?
                Socket.__net__ "udp_open", 0, address, port
              else
                Socket.__net__ "listen", 0, address, port
              end
    0
  end

  def connect(sockaddr)
    port, address = Socket.unpack_sockaddr_in sockaddr
    if datagram?
      @handle = Socket.__net__("udp_open", 0, "0.0.0.0", 0) if @handle.nil?
      Socket.__net__ "udp_connect", @handle, address, port
    else
      @handle = Socket.__net__ "connect", 0, address, port
    end
    0
  end

  def listen(_backlog = 5)
    0
  end

  def accept
    handle = Socket.__net__ "accept", @handle, "", 0
    [TCPSocket.new(handle), Addrinfo.tcp(*Socket.__net__("peer", handle, "", 0))]
  end

  # A message may name where it goes, which is what a socket carrying each
  # message on its own needs.
  def send(message, _flags = 0, destination = nil)
    unless destination.nil?
      port, address = Socket.unpack_sockaddr_in destination
      @handle = Socket.__net__("udp_open", 0, "0.0.0.0", 0) if @handle.nil?
      Socket.__net__ "udp_connect", @handle, address, port
    end
    return Socket.__net__("udp_send", @handle, message.to_s, 0) if datagram?
    Socket.__net__ "write", @handle, message.to_s, 0
  end

  alias_method :write, :send

  def recv(length = nil, _flags = nil)
    return Socket.__net__("read", @handle, "", length.nil? ? 0 : length.to_i) unless datagram?
    Socket.__net__("udp_receive", @handle, "", length.nil? ? 0 : length.to_i)[0]
  end

  alias_method :read, :recv
  alias_method :recv_nonblock, :recv
  alias_method :read_nonblock, :recv
  alias_method :write_nonblock, :write

  def recvfrom(length = nil, _flags = nil)
    held = Socket.__net__ "udp_receive", @handle, "", length.nil? ? 0 : length.to_i
    kind = Socket.__address__("family", held[1], 0) == 4 ? "AF_INET" : "AF_INET6"
    [held[0], [kind, held[2], held[1], held[1]]]
  end

  def getsockname
    return Socket.sockaddr_in 0, "0.0.0.0" if @handle.nil?
    named = Socket.__net__(datagram? ? "udp_address" : "address", @handle, "", 0)
    return Socket.sockaddr_in 0, "0.0.0.0" if named.nil?
    Socket.sockaddr_in named[1], named[0]
  end

  def addr
    named = Socket.__net__(datagram? ? "udp_address" : "address", @handle, "", 0)
    named.nil? ? nil : BasicSocket.tuple_for(named)
  end

  def local_address
    named = Socket.__net__(datagram? ? "udp_address" : "address", @handle, "", 0)
    return nil if named.nil?
    datagram? ? Addrinfo.udp(named[0], named[1]) : Addrinfo.tcp(named[0], named[1])
  end

  def close
    return nil if @closed
    Socket.__net__ "close", @handle, "", 0 unless @handle.nil?
    @closed = true
    nil
  end

  def fileno
    @handle.nil? ? -1 : Socket.__net__("fileno", @handle, "", 0)
  end

  # Two sockets already joined to each other, which is what `pair` gives.
  def self.pair(family = nil, socktype = nil, protocol = 0)
    listener = Socket.__net__ "listen", 0, "127.0.0.1", 0
    named = Socket.__net__ "address", listener, "", 0
    first = TCPSocket.new "127.0.0.1", named[1]
    second = TCPSocket.new Socket.__net__("accept", listener, "", 0)
    Socket.__net__ "close", listener, "", 0
    [first, second]
  end

  class << self
    alias_method :socketpair, :pair
  end

  # Where a datagram came from, which `udp_server_recv` hands to its block
  # alongside the message.
  class UDPSource
    attr_reader :remote_address
    attr_reader :local_address

    def initialize(remote_address, local_address, &reply)
      @remote_address = remote_address
      @local_address = local_address
      @reply = reply
    end

    def reply(message)
      @reply.call message if @reply
    end

    def inspect
      "#<Socket::UDPSource #{@remote_address.inspect} to #{@local_address.inspect}>"
    end
  end

  # Take one message from each socket handed in, with where it came from.
  def self.udp_server_recv(sockets)
    sockets.each do |socket|
      message, from = socket.recvfrom 65536
      remote = Addrinfo.udp from[2], from[1]
      source = UDPSource.new(remote, socket.local_address) do |reply|
        socket.send reply, 0, from[2], from[1]
      end
      yield message, source
    end
    nil
  end

  # Every address this machine answers on, as far as it can be asked.
  def self.ip_address_list
    Socket.resolved(Socket.gethostname).map { |held| Addrinfo.ip held }
  rescue SocketError
    [Addrinfo.ip("127.0.0.1")]
  end

  def self.gethostbyname(host)
    TCPSocket.gethostbyname host
  end

  # Take up a handle the operating system already holds open.
  def __take_handle__(handle)
    @handle = handle
    self
  end

  private :__take_handle__

  # Hand a socket to a block and close it once the block is done, which is
  # what the `Socket.` openers do when one is given.
  def self.opened_for(socket)
    return socket unless block_given?
    begin
      yield socket
    ensure
      socket.close unless socket.closed?
    end
  end

  # A stream socket joined to a name in the file system.
  def self.unix(path, &block)
    held = Socket.new :UNIX, :STREAM
    held.__send__ :__take_handle__, Socket.__net__("unix_connect", 0, path.to_s, 0)
    opened_for held, &block
  end

  # A socket waiting for connections under a name in the file system.
  def self.unix_server_socket(path, &block)
    held = Socket.new :UNIX, :STREAM
    held.__send__ :__take_handle__, Socket.__net__("unix_listen", 0, path.to_s, 0)
    opened_for held, &block
  end

  # The sockets a server listens on, which metorex answers as the one socket
  # it opens for the address asked for.
  def self.tcp_server_sockets(host = nil, port = nil, &block)
    host, port = nil, host if port.nil?
    held = Socket.new :INET, :STREAM
    held.bind Socket.sockaddr_in(port.nil? ? 0 : port, host)
    held.listen 5
    sockets_opened_for [held], &block
  end

  def self.udp_server_sockets(host = nil, port = nil, &block)
    host, port = nil, host if port.nil?
    held = Socket.new :INET, :DGRAM
    held.bind Socket.sockaddr_in(port.nil? ? 0 : port, host)
    sockets_opened_for [held], &block
  end

  # Hand a list of sockets to a block and close them all afterwards.
  def self.sockets_opened_for(sockets)
    return sockets unless block_given?
    begin
      yield sockets
    ensure
      sockets.each { |socket| socket.close unless socket.closed? }
    end
  end
end
