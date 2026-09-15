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
  # Where this end sits, as the four-part array Ruby answers. A reverse
  # lookup is asked for by the argument, and by the setting on the socket or
  # on the class when none is given.
  def addr(reverse_lookup = nil)
    BasicSocket.check_reverse_lookup reverse_lookup
    named = Socket.__net__ "address", @handle, "", 0
    return nil if named.nil?
    BasicSocket.tuple_for named, resolving_names?(reverse_lookup)
  end

  # Whether the name behind an address should be looked up: what the call
  # asked for, or what the socket and the class were set to when it asked
  # for nothing.
  def resolving_names?(reverse_lookup)
    return reverse_lookup unless reverse_lookup.nil?
    !do_not_reverse_lookup
  end
  private :resolving_names?

  def local_address
    named = Socket.__net__ "address", @handle, "", 0
    named.nil? ? nil : Addrinfo.stream(named[0], named[1])
  end

  # What a reverse-lookup argument may be: nothing, a boolean, or one of the
  # two names Ruby answers to. Anything else is refused.
  def self.check_reverse_lookup(wanted)
    return if wanted.nil? || wanted == true || wanted == false
    return if wanted == :hostname || wanted == :numeric
    raise ArgumentError, "invalid reverse_lookup flag: #{wanted.inspect}"
  end

  def self.tuple_for(named, reverse_lookup = nil)
    kind = Socket.__address__("family", named[0], 0) == 4 ? "AF_INET" : "AF_INET6"
    # A reverse lookup names the host the address belongs to, where leaving
    # it out names the address itself.
    spelled = if reverse_lookup == true || reverse_lookup == :hostname
                Socket.__address__("name_of", named[0], 0) || named[0]
              else
                named[0]
              end
    [kind, named[1], spelled, named[0]]
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
  # Change a setting on the socket itself, which the operating system keeps
  # rather than the program.
  # Change a setting on the socket itself. A socket that has no descriptor
  # yet remembers what it was told until it has one.
  def setsockopt(level, name = nil, value = nil)
    @options = {} unless defined? @options
    held = level.is_a?(Socket::Option) ? level : Socket::Option.new(0, level, name, value)
    @options[[held.level, held.optname]] = held
    apply_option held
    0
  end

  # Put one setting on the socket itself. A setting the operating system
  # refuses on a socket already bound is kept and put on the next one the
  # program binds, which is where it would have taken effect anyway.
  def apply_option(held)
    return if @handle.nil?
    Socket.__net__ "set_socket_option", @handle,
                   "#{held.level}/#{held.optname}/#{held.data}", 0
  rescue Errno::EINVAL
    raise unless @bound.nil?
  end
  private :apply_option

  # Put every setting the program asked for on a socket it has just opened.
  def apply_options
    return if @options.nil?
    @options.each_value { |held| apply_option held }
  end
  private :apply_options

  # What a setting holds right now, read back from the operating system when
  # there is a socket behind this one to ask.
  def getsockopt(level, name)
    @options = {} unless defined? @options
    wanted = Socket::Option.level_number level
    named = Socket::Option.option_number wanted, name
    # A setting the program asked for that the operating system would not
    # take yet is what it reads back, since that is what the next socket
    # this one binds will carry.
    kept = @options[[wanted, named]]
    return kept if !kept.nil? && @bound.nil?
    unless @handle.nil?
      held = Socket.__net__ "socket_option", @handle, "#{wanted}/#{named}", 4
      return Socket::Option.new(@socket_family || Socket::AF_INET, wanted, named, held)
    end
    @options[[wanted, named]] || Socket::Option.new(0, wanted, named, "\x00\x00\x00\x00")
  end

  # Close one direction of a connection. `SHUT_RD` stops reading, `SHUT_WR`
  # stops writing, and `SHUT_RDWR` stops both.
  def shutdown(how = nil)
    # A mode may be spelled out by something that answers `to_str`, the way
    # any name a socket takes may be.
    unless how.nil? || how.is_a?(Integer) || how.is_a?(Symbol) || how.is_a?(String)
      unless how.respond_to? :to_str
        raise TypeError, "no implicit conversion of #{how.class} into String"
      end
      how = how.to_str
    end
    wanted = case how
             when nil, Socket::SHUT_RDWR, :SHUT_RDWR, "SHUT_RDWR", :RDWR, "RDWR"
               Socket::SHUT_RDWR
             when Socket::SHUT_RD, :SHUT_RD, "SHUT_RD", :RD, "RD" then Socket::SHUT_RD
             when Socket::SHUT_WR, :SHUT_WR, "SHUT_WR", :WR, "WR" then Socket::SHUT_WR
             when Integer
               raise ArgumentError, "invalid shutdown mode: #{how}"
             else
               # A name nothing answers to is not a mode at all, which is
               # what Ruby reports as a socket error rather than a bad
               # argument.
               raise SocketError, "invalid shutdown mode: #{how}"
             end
    Socket.__net__ "shutdown", @handle, "", wanted
    0
  end

  # Closing one side leaves the other open, which is what a socket allows
  # where a plain stream does not. Closing both closes the socket itself.
  def close_write
    raise IOError, "closed stream" if closed?
    @write_closed = true
    begin
      shutdown Socket::SHUT_WR
    rescue SystemCallError
      nil
    end
    close if @read_closed
    nil
  end

  def close_read
    raise IOError, "closed stream" if closed?
    @read_closed = true
    begin
      shutdown Socket::SHUT_RD
    rescue SystemCallError
      nil
    end
    close if @write_closed
    nil
  end

  # Whether the side that reads, or the side that writes, has been closed on
  # its own.
  def read_closed?
    @read_closed == true
  end

  def write_closed?
    @write_closed == true
  end

  # Where the other end sits, and where this end does. A socket in the UNIX
  # family is named by a path rather than by a name and a port.
  def remote_address
    # A socket that reached a server under a name answers with that name,
    # which is what the other end is known by.
    return Addrinfo.unix(@peer_path.to_s) unless @peer_path.nil?
    if @socket_type == Socket::SOCK_DGRAM
      named = Socket.__net__ "udp_peer", @handle, "", 0
      raise Errno::ENOTCONN, "socket is not connected" if named.nil?
      return Addrinfo.built(named[0], named[1], Socket::SOCK_DGRAM, 0)
    end
    named = Socket.__net__ "peer", @handle, "", 0
    return Addrinfo.built(named[0], named[1], Socket::SOCK_STREAM, 0) unless named.nil?
    path = Socket.__net__ "unix_address", @handle, "", 0
    raise Errno::ENOTCONN, "socket is not connected" if path.nil?
    Addrinfo.unix path.to_s
  end

  def local_address
    if @socket_type == Socket::SOCK_DGRAM
      named = Socket.__net__ "udp_address", @handle, "", 0
      raise Errno::ENOTCONN, "socket is not connected" if named.nil?
      return Addrinfo.built(named[0], named[1], Socket::SOCK_DGRAM, 0)
    end
    named = Socket.__net__ "address", @handle, "", 0
    return Addrinfo.built(named[0], named[1], Socket::SOCK_STREAM, 0) unless named.nil?
    path = Socket.__net__ "unix_address", @handle, "", 0
    raise Errno::ENOTCONN, "socket is not connected" if path.nil?
    Addrinfo.unix path.to_s
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
    # A socket taken up from a descriptor answers the name the other end is
    # known by, which the operating system holds against the descriptor.
    named = Socket.__net__ "unix_peer", handle, "", 0
    @peer_path = named unless named.nil? || named.to_s.empty?
    # A socket taken up from a descriptor reports the family and the kind
    # the one behind that descriptor really is.
    case Socket.__net__("kind_of_handle", handle, "", 0)
    when "unix_stream"
      @socket_family = Socket::AF_UNIX
      @socket_type = Socket::SOCK_STREAM
    when "inet_datagram"
      @socket_family = Socket::AF_INET
      @socket_type = Socket::SOCK_DGRAM
    when "unix_datagram"
      @socket_family = Socket::AF_UNIX
      @socket_type = Socket::SOCK_DGRAM
    else
      @socket_family = Socket::AF_INET
      @socket_type = Socket::SOCK_STREAM
    end
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
    AF_IPX = 23
    PF_IPX = 23
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
    SOCK_RDM = 4
    SOCK_SEQPACKET = 5

    IPPROTO_IP = 0
    IPPROTO_ICMP = 1
    IPPROTO_TCP = 6
    IPPROTO_UDP = 17
    IPPROTO_IPV6 = 41
    IPPROTO_RAW = 255

    SOL_SOCKET = 0xffff
    SO_DEBUG = 0x0001
    SO_ACCEPTCONN = 0x0002
    SO_REUSEADDR = 0x0004
    SO_KEEPALIVE = 0x0008
    SO_DONTROUTE = 0x0010
    SO_BROADCAST = 0x0020
    SO_USELOOPBACK = 0x0040
    SO_OOBINLINE = 0x0100
    SO_REUSEPORT = 0x0200
    SO_LINGER = 0x0080
    SO_SNDBUF = 0x1001
    SO_RCVBUF = 0x1002
    SO_TYPE = 0x1008
    SO_ERROR = 0x1007
    SO_SNDLOWAT = 0x1003
    SO_RCVLOWAT = 0x1004
    SO_SNDTIMEO = 0x1005
    SO_RCVTIMEO = 0x1006

    # What a send or a receive may be told to do differently, as the
    # operating system on this platform numbers them.
    MSG_OOB = 0x1
    MSG_PEEK = 0x2
    MSG_DONTROUTE = 0x4
    MSG_EOR = 0x8
    MSG_TRUNC = 0x10
    MSG_CTRUNC = 0x20
    MSG_WAITALL = 0x40
    MSG_DONTWAIT = 0x80
    MSG_EOF = 0x100
    MSG_NOSIGNAL = 0x80000

    TCP_NODELAY = 0x01
    TCP_MAXSEG = 0x02
    TCP_KEEPALIVE = 0x10
    TCP_NOPUSH = 0x04
    TCP_NOOPT = 0x08

    # What a message may carry alongside its bytes over a socket named by a
    # path, and what the address resolver reports when it cannot answer.
    SCM_RIGHTS = 0x01
    SCM_TIMESTAMP = 0x02
    SCM_CREDS = 0x03

    EAI_ADDRFAMILY = 1
    EAI_AGAIN = 2
    EAI_BADFLAGS = 3
    EAI_FAIL = 4
    EAI_FAMILY = 5
    EAI_MEMORY = 6
    EAI_NODATA = 7
    EAI_NONAME = 8
    EAI_SERVICE = 9
    EAI_SOCKTYPE = 10
    EAI_SYSTEM = 11
    EAI_OVERFLOW = 14

    IP_TTL = 4
    IP_MULTICAST_TTL = 10
    IP_MULTICAST_LOOP = 11
    IP_ADD_MEMBERSHIP = 12
    IP_DROP_MEMBERSHIP = 13
    IP_DEFAULT_MULTICAST_TTL = 1
    IP_DEFAULT_MULTICAST_LOOP = 1
    IP_MAX_MEMBERSHIPS = 4095
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
  def self.getaddrinfo(host, port, family = nil, socktype = nil, protocol = nil, flags = nil,
                       reverse_lookup = nil)
    numbered = family.nil? || family == 0 ? nil : Socket.family_numbered(Socket.named_part(family))
    if numbered == Socket::AF_UNIX
      raise Socket::ResolutionError.new("getaddrinfo: ai_family not supported",
                                        Socket::EAI_FAMILY)
    end
    named =
      if host.nil? || host.to_s.empty?
        # No host at all names every address a server would answer on, or
        # the loopback when the address is for reaching one.
        passive = !flags.nil? && flags.to_i & Socket::AI_PASSIVE != 0
        if numbered == Socket::AF_INET6
          [passive ? "::" : "::1"]
        elsif numbered.nil?
          passive ? ["0.0.0.0", "::"] : ["127.0.0.1", "::1"]
        else
          [passive ? "0.0.0.0" : "127.0.0.1"]
        end
      else
        Socket.resolved Socket.named_part(host)
      end
    named = named.select do |address|
      wanted = Socket.__address__("family", address, 0) == 4 ? Socket::AF_INET : Socket::AF_INET6
      numbered.nil? || wanted == numbered
    end
    wanted = port.nil? ? 0 : Socket.port_number(port)
    named.map do |address|
      kind = Socket.__address__("family", address, 0) == 4 ? "AF_INET" : "AF_INET6"
      # Asking for a reverse lookup names the host the address belongs to,
      # where leaving it out names the address itself.
      wants_name =
        case reverse_lookup
        when nil then !BasicSocket.do_not_reverse_lookup
        when :numeric, false then false
        else true
        end
      spelled = wants_name ? (Socket.__address__("name_of", address, 0) || address) : address
      [kind, wanted, spelled, address,
       kind == "AF_INET" ? Constants::AF_INET : Constants::AF_INET6,
       socktype.nil? || socktype == 0 ? Constants::SOCK_STREAM : Socket.socktype_numbered(Socket.named_part(socktype)),
       protocol.nil? || protocol == 0 ? Constants::IPPROTO_TCP : protocol]
    end
  end

  # The addresses a host name stands for. A written address stands for
  # itself rather than being looked up.
  def self.resolved(host)
    held = host.nil? || host.to_s.empty? ? "0.0.0.0" : host.to_s
    # Two names stand for addresses rather than for hosts: everything on the
    # network, and every name this machine answers on.
    return ["255.255.255.255"] if held == "<broadcast>"
    return ["0.0.0.0"] if held == "<any>"
    return [Socket.__address__("normalize", held, 0)] unless Socket.__address__("family", held, 0).nil?
    found = Socket.__address__ "resolve", held, 0
    found.select { |address| Socket.__address__("family", address, 0) == 4 } +
      found.select { |address| Socket.__address__("family", address, 0) != 4 }
  end

  # A port named by number, or by the name a service is known under.
  def self.port_number(port)
    # No port at all is the one the operating system picks, which is what
    # binding to zero asks for.
    return 0 if port.nil?
    return port if port.is_a? Integer
    port = port.to_int if !port.is_a?(String) && port.respond_to?(:to_int)
    return port if port.is_a? Integer
    port = port.to_str if !port.is_a?(String) && port.respond_to?(:to_str)
    # A port has to read as a number or as a name in the end. Anything that
    # spells itself out as neither is refused the way Ruby refuses it.
    unless port.is_a? String
      raise TypeError, "no implicit conversion of #{port.class} into String"
    end
    named = port.to_s
    return 0 if named.empty?
    return named.to_i if named =~ /\A\d+\z/
    found = Socket.__net__ "service_port", 0, "#{named}/tcp", 0
    return found unless found.nil?
    raise SocketError, "getaddrinfo: Servname not supported for ai_socktype"
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

  def self.getaddrinfo(host, port, family = nil, socktype = nil, protocol = nil, flags = nil,
                       _reverse_lookup = nil)
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

  # The parts an address is put back together from: the families and the
  # socket kind by name, the address itself paired with its port, and the
  # protocol by the name it is known under.
  def marshal_dump
    address = unix? ? @address : [@address, @port.to_s]
    [afamily_name, address, pfamily_name, socktype_name_or_number,
     protocol_name_or_number, @canonname]
  end

  def marshal_load(held)
    named, address, pfamily, socktype, protocol, canonname = held
    if address.is_a? Array
      @address = address[0]
      @port = address[1].to_i
    else
      @address = address
      @port = 0
    end
    @socktype = socktype.is_a?(Integer) ? socktype : Addrinfo.socktype_numbered(socktype)
    @protocol = protocol.is_a?(Integer) ? protocol : Addrinfo.protocol_numbered(protocol)
    @canonname = canonname
    @afamily = named == "AF_UNIX" ? Socket::AF_UNIX : (named == "AF_INET6" ? Socket::AF_INET6 : Socket::AF_INET)
    @pfamily = @afamily
    self
  end

  # The name a protocol family goes by, which is the address family's name
  # with the other prefix.
  def pfamily_name
    afamily_name.sub("AF_", "PF_")
  end

  # The name a protocol is known under, or the number itself when it has
  # none. A UNIX address names no protocol at all.
  def protocol_name_or_number
    return @protocol if unix? || @protocol.nil?
    case @protocol
    when Socket::IPPROTO_TCP then "IPPROTO_TCP"
    when Socket::IPPROTO_UDP then "IPPROTO_UDP"
    when Socket::IPPROTO_IP then "IPPROTO_IP"
    when Socket::IPPROTO_ICMP then "IPPROTO_ICMP"
    else @protocol
    end
  end

  def self.protocol_numbered(named)
    case named.to_s
    when "IPPROTO_TCP" then Socket::IPPROTO_TCP
    when "IPPROTO_UDP" then Socket::IPPROTO_UDP
    when "IPPROTO_ICMP" then Socket::IPPROTO_ICMP
    else Socket::IPPROTO_IP
    end
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

  # A socket connected to this address. The connection is handed to a block
  # when one is given, and closed after it.
  def connect(timeout: nil)
    held = if @afamily == Socket::AF_UNIX
             Socket.unix @address
           elsif @socktype == Socket::SOCK_DGRAM
             # A socket carrying each message on its own is pointed at the
             # address rather than connected over one.
             opened = UDPSocket.new @afamily
             opened.connect @address, @port
             opened
           else
             Socket.tcp @address, @port
           end
    return held unless block_given?
    begin
      yield held
    ensure
      held.close unless held.closed?
    end
  end

  # An address of the same family and kind as this one, named by an Addrinfo
  # of its own or by the parts an address of that family is written in.
  def family_addrinfo(*args)
    if args.first.is_a? Addrinfo
      unless args.length == 1
        raise ArgumentError, "wrong number of arguments (given #{args.length}, expected 1)"
      end
      # An address of another family names somewhere this one cannot reach.
      unless args.first.pfamily == @pfamily
        raise ArgumentError, "protocol family mismatch: #{args.first.pfamily} != #{@pfamily}"
      end
      unless args.first.socktype == @socktype
        raise ArgumentError, "socket type mismatch: #{args.first.socktype} != #{@socktype}"
      end
      return args.first
    end
    if unix?
      raise ArgumentError, "wrong number of arguments (given #{args.length}, expected 1)" unless args.length == 1
      return Addrinfo.unix(args.first.to_s, @socktype)
    end
    unless args.length == 2
      raise ArgumentError, "wrong number of arguments (given #{args.length}, expected 2)"
    end
    Addrinfo.built Socket.resolved(args[0]).first, Socket.port_number(args[1]),
                   @socktype, @protocol
  end

  # Reach this address from a name and a port of this machine's own. The
  # local end may be named by an Addrinfo or by its parts.
  def connect_from(*args, **_options, &block)
    local = family_addrinfo(*args)
    held = Socket.new @afamily, @socktype
    held.bind Socket.sockaddr_in(local.ip_port, local.ip_address)
    held.connect Socket.sockaddr_in(@port, @address)
    Socket.opened_for held, &block
  end

  # Reach this address, then hand the socket what it should say. The other
  # end is named by this address itself.
  def connect_to(*args, **options, &block)
    family_addrinfo(*args).connect_from(self, **options, &block)
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
    @socktype = Addrinfo.socktype_numbered_or_zero socktype
    @protocol = protocol
    @canonname = nil
    @afamily = Socket.__address__("family", named, 0) == 4 ? Socket::AF_INET : Socket::AF_INET6
    @pfamily = @afamily
    self
  end

  def fill_unix(path, socktype)
    @address = path
    @port = nil
    @socktype = Addrinfo.socktype_numbered_or_zero socktype
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

  # The number a socket type is known by. A type given by name is looked up,
  # and no type at all stays 0, which is what a plain address reports.
  def self.socktype_numbered_or_zero(held)
    return 0 if held.nil? || held == 0
    return held if held.is_a? Integer
    Socket.socktype_numbered held
  end

  private :afamily_name, :socktype_name_or_number
end


# One end of a connection someone made or accepted.
class TCPSocket < IPSocket
  def initialize(host, port = nil, _local_host = nil, _local_port = nil,
                 connect_timeout: nil, resolv_timeout: nil)
    # An accepted connection is already open, and arrives as the number the
    # interpreter holds it under rather than as a name and a port.
    return super(host) if port.nil?
    if block_given?
      warn "warning: TCPSocket::new() does not take block; use TCPSocket::open() instead"
    end
    super TCPSocket.connected(host, Socket.port_number(port), connect_timeout)
  end

  # A connection to the first address the host answers on. A host written as
  # nothing names this machine, which may answer on either family, so each
  # address is tried until one of them takes the connection.
  def self.connected(host, port, connect_timeout = nil)
    named = if host.nil? || host.to_s.empty?
              ["127.0.0.1", "::1"]
            else
              Socket.resolved host
            end
    refused = nil
    handle = nil
    started = Process.clock_gettime Process::CLOCK_MONOTONIC
    named.each do |address|
      break unless handle.nil?
      begin
        handle = Socket.__net__ "connect", 0, address, port
      rescue SystemCallError => problem
        refused = problem
      end
    end
    return handle unless handle.nil?
    # A connection given a limit on how long it may take reports running out
    # of time as its own kind of error rather than as a refusal.
    unless connect_timeout.nil?
      spent = Process.clock_gettime(Process::CLOCK_MONOTONIC) - started
      raise IO::TimeoutError, "Connect timeout expired" if spent >= connect_timeout
    end
    raise refused
  end

  def self.open(host, port = nil, local_host = nil, local_port = nil,
                connect_timeout: nil, resolv_timeout: nil)
    held = new host, port, local_host, local_port,
               connect_timeout: connect_timeout, resolv_timeout: resolv_timeout
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

  # Read what has arrived. A buffer handed in takes the place of its own
  # characters, keeping the encoding it was tagged with.
  def read(length = nil, buffer = nil)
    raise IOError, "closed stream" if closed? || read_closed?
    held = read_into_string length
    return held if buffer.nil?
    # The buffer keeps the encoding it was tagged with, since what arrived
    # is bytes rather than characters of any particular encoding.
    tagged = buffer.encoding
    buffer.replace held
    buffer.force_encoding tagged
    buffer
  end

  def read_into_string(length)
    # What a read past a separator took but did not hand back is answered
    # before anything more is asked of the connection.
    waiting = @pending
    unless waiting.nil? || waiting.empty?
      @pending = nil
      return waiting if length.nil?
      wanted = length.to_i
      if waiting.length > wanted
        @pending = waiting[wanted, waiting.length - wanted]
        return waiting[0, wanted]
      end
      return waiting
    end
    Socket.__net__ "read", @handle, "", length.nil? ? 0 : length.to_i
  end
  private :read_into_string

  # A connection the other end has finished with hands back nothing at all
  # rather than an empty string.
  def recv(length = nil, _flags = nil, buffer = nil)
    held = read length
    return nil if held.empty?
    return held if buffer.nil?
    tagged = buffer.encoding
    buffer.replace held
    buffer.force_encoding tagged
    buffer
  end

  # Take what has already arrived without waiting for more. Nothing there
  # yet is reported as a wait rather than as an end, either by raising or by
  # answering the reason, depending on what was asked for.
  def recv_nonblock(length = nil, _flags = 0, buffer = nil, exception: true)
    begin
      held = read length
    rescue Errno::EAGAIN, Errno::EWOULDBLOCK => trouble
      return :wait_readable unless exception
      raise IO::EAGAINWaitReadable, trouble.message
    end
    return nil if held.empty?
    return held if buffer.nil?
    # The buffer keeps the encoding it was tagged with, since what arrived
    # is bytes rather than characters of any particular encoding.
    tagged = buffer.encoding
    buffer.replace held
    buffer.force_encoding tagged
    buffer
  end

  def read_nonblock(length = nil, buffer = nil, exception: true)
    recv_nonblock length, 0, buffer, exception: exception
  end
  alias_method :write_nonblock, :write

  def eof?
    false
  end

  def readpartial(length, _buffer = nil)
    held = read length
    raise EOFError, "end of file reached" if held.empty?
    held
  end

  # Read up to and including the separator, which is what a protocol written
  # in lines asks for. Without one the next thing to arrive is the answer.
  def gets(separator = "\n")
    return nil if separator.nil? && (held = read).empty?
    return held if separator.nil?
    ending = separator.to_s
    collected = +""
    while !ending.empty?
      at = collected.index(ending)
      unless at.nil?
        ends = at + ending.length
        @pending = collected[ends, collected.length - ends]
        return collected[0, ends]
      end
      piece = begin
        read
      rescue SystemCallError
        nil
      end
      break if piece.nil? || piece.empty?
      collected << piece
    end
    collected.empty? ? nil : collected
  end

  def write(text)
    Socket.__net__ "write", @handle, text.to_s, 0
  end

  # Read what has arrived over the connection, paired with where the other
  # end sits. A buffer handed in takes the place of its own characters.
  def recvfrom(length = nil, flags = 0, buffer = nil)
    held = recv length, flags
    # A connection the other end has finished with hands back nothing at all.
    return nil if held.nil?
    named = Socket.__net__("peer", @handle, "", 0) || ["0.0.0.0", 0]
    [Socket.filled_buffer(held, buffer),
     BasicSocket.tuple_for(named, !do_not_reverse_lookup)]
  end

  # Send what the message holds, with the flags naming anything unusual
  # about how it travels.
  def send(message, flags = 0, _destination = nil)
    return write message if flags.to_i.zero?
    Socket.__net__ "write", @handle, message.to_s, flags.to_i
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
  # Where the other end sits, as the four-part array Ruby answers.
  def peeraddr(reverse_lookup = nil)
    BasicSocket.check_reverse_lookup reverse_lookup
    named = Socket.__net__ "peer", @handle, "", 0
    named.nil? ? nil : BasicSocket.tuple_for(named, resolving_names?(reverse_lookup))
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
    if block_given?
      warn "warning: TCPServer::new() does not take block; use TCPServer::open() instead"
    end
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
    raise IOError, "closed stream" if closed? || read_closed?
    raise Errno::ENOTCONN, "socket is not connected"
  end

  def read(length = nil, *)
    raise IOError, "closed stream" if closed? || read_closed?
    # Reading nothing at all is answered without reaching for a connection
    # this socket does not have.
    return "" if length == 0
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
  # A connected socket is not something anything else connects to, so there
  # is no such address to report.
  def connect_address
    raise SocketError, "getnameinfo: ai_family not supported"
  end

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
    # The name this end reached the server under is the name the other end
    # is known by, which is what `remote_address` answers.
    @peer_path = @path
    @path = ""
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
    Socket.__net__ "unix_read", @handle, "0", length.nil? ? 0 : length.to_i
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

  # Read what has arrived, saying where it came from. A buffer handed in
  # takes the place of its own characters, keeping its encoding.
  # Hand a stream of this program to the other end, which takes it up as a
  # stream of its own.
  def send_io(io)
    number = io.respond_to?(:fileno) ? io.fileno : io.to_i
    Socket.__net__ "send_fd", @handle, "", number
    nil
  end

  # Take a stream the other end handed over, as the class asked for.
  def recv_io(kind = IO, mode = nil)
    number = Socket.__net__ "receive_fd", @handle, "", 0
    made = kind.nil? ? number : kind.for_fd(number, mode)
    made
  end

  # Read what has arrived, saying where it came from. A connection carries
  # no name for the other end, which is what the empty path stands for.
  def recvfrom(length = nil, flags = 0, buffer = nil)
    if @socket_type == Socket::SOCK_DGRAM
      taken = Socket.__net__ "unix_dgram_receive", @handle, "",
                             length.nil? ? 0 : length.to_i
      return [Socket.filled_buffer(taken[0], buffer), ["AF_UNIX", taken[1]]]
    end
    held = Socket.__net__ "unix_read", @handle, flags.to_i.to_s,
                          length.nil? ? 0 : length.to_i
    [Socket.filled_buffer(held, buffer), ["AF_UNIX", ""]]
  end
end

# A socket named by a path, waiting for connections to be made to it.
class UNIXServer < UNIXSocket
  # A server is reached by the name it was bound under, where a client end
  # carries none of its own.
  def path
    @path
  end

  # Where something should connect to reach this listener.
  def connect_address
    Addrinfo.unix @path
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

  # Where this end sits, and where the other end does, as a socket carrying
  # each message on its own names them.
  def local_address
    named = Socket.__net__ "udp_address", @handle, "", 0
    raise Errno::ENOTCONN, "socket is not connected" if named.nil?
    Addrinfo.built named[0], named[1], Socket::SOCK_DGRAM, 0
  end

  def remote_address
    named = Socket.__net__ "udp_peer", @handle, "", 0
    raise Errno::ENOTCONN, "socket is not connected" if named.nil?
    Addrinfo.built named[0], named[1], Socket::SOCK_DGRAM, 0
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

  # Point this socket at a name and a port. A host written as nothing names
  # this machine, over the loopback of the family the socket belongs to.
  def connect(host, port)
    named = if host.nil? || host.to_s.empty?
              @family == Socket::AF_INET6 ? "::1" : "127.0.0.1"
            else
              Socket.resolved(host).first
            end
    Socket.__net__ "udp_connect", @handle, named, port.to_i
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
  def recvfrom(length = nil, flags = 0, buffer = nil)
    held = Socket.__net__ "udp_receive", @handle, flags.to_i.to_s,
                          length.nil? ? 0 : length.to_i
    named = BasicSocket.tuple_for([held[1], held[2]], !do_not_reverse_lookup)
    [Socket.filled_buffer(held[0], buffer), named]
  end

  def recv(length = nil, flags = 0, buffer = nil)
    recvfrom(length, flags, buffer)[0]
  end

  # Take what has already arrived without waiting. Nothing there yet is
  # reported as a wait rather than as an error the caller cannot tell apart.
  def recvfrom_nonblock(length = nil, flags = 0, buffer = nil, exception: true)
    begin
      recvfrom length, flags.to_i | Socket::MSG_DONTWAIT, buffer
    rescue Errno::EAGAIN, Errno::EWOULDBLOCK => trouble
      return :wait_readable unless exception
      raise IO::EAGAINWaitReadable, trouble.message
    end
  end
  alias_method :recv_nonblock, :recv

  def addr
    named = Socket.__net__ "udp_address", @handle, "", 0
    return nil if named.nil?
    BasicSocket.tuple_for named
  end

  # Where this end sits. The address names no protocol of its own, which is
  # what Ruby reports for one read back off a socket.
  def local_address
    named = Socket.__net__ "udp_address", @handle, "", 0
    return nil if named.nil?
    Addrinfo.built named[0], named[1], Socket::SOCK_DGRAM, 0
  end

  def remote_address
    named = Socket.__net__ "udp_peer", @handle, "", 0
    raise Errno::ENOTCONN, "socket is not connected" if named.nil?
    Addrinfo.built named[0], named[1], Socket::SOCK_DGRAM, 0
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
    # it has a descriptor of its own before it is bound to anything. The
    # address it starts on decides which family it belongs to, which is what
    # lets it reach an address of that family later.
    @handle = if datagram? && @socket_family == Socket::AF_UNIX
                Socket.__net__("unix_dgram_open", 0, "", 0)
              elsif datagram?
                Socket.__net__("udp_open", 0,
                               @socket_family == Socket::AF_INET6 ? "::" : "0.0.0.0", 0)
              end
  end

  # What was read, put into the buffer when one was handed in. The buffer
  # keeps the encoding it was tagged with, since what arrived is bytes.
  def self.filled_buffer(held, buffer)
    return held if buffer.nil?
    tagged = buffer.encoding
    buffer.replace held
    buffer.force_encoding tagged
    buffer
  end

  # The number an argument stands for, refusing anything that does not read
  # as one the way Ruby refuses it.
  def self.integer_part(held)
    return held if held.is_a? Integer
    unless held.respond_to? :to_int
      raise TypeError, "no implicit conversion of #{held.class} into Integer"
    end
    numbered = held.to_int
    unless numbered.is_a? Integer
      raise TypeError,
            "can't convert #{held.class} to Integer (#{held.class}#to_int gives #{numbered.class})"
    end
    numbered
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
    when "INET" then Socket::AF_INET
    when "INET6" then Socket::AF_INET6
    when "UNIX", "LOCAL" then Socket::AF_UNIX
    when "UNSPEC" then Socket::AF_UNSPEC
    else raise SocketError, "unknown socket domain #{held}"
    end
  end

  def self.socktype_numbered(held)
    return held if held.is_a? Integer
    case held.to_s.upcase.sub("SOCK_", "")
    when "STREAM" then Socket::SOCK_STREAM
    when "DGRAM" then Socket::SOCK_DGRAM
    when "RAW" then Socket::SOCK_RAW
    when "SEQPACKET" then Socket::SOCK_SEQPACKET
    when "RDM" then Socket::SOCK_RDM
    else raise SocketError, "unknown socket type #{held}"
    end
  end

  def datagram?
    @socket_type == Socket::SOCK_DGRAM
  end

  private :datagram?

  # Take the name and port this socket is to answer on.
  def bind(sockaddr)
    # A socket is bound once. Binding a second time is what the operating
    # system reports as an invalid argument.
    raise Errno::EINVAL, "Invalid argument - bind(2)" unless @bound.nil?
    # A socket in the UNIX family is bound to a path rather than to a name
    # and a port.
    if @socket_family == Socket::AF_UNIX
      path = Socket.unpack_sockaddr_un sockaddr
      @bound = [path, 0]
      taken = if datagram?
                Socket.__net__ "unix_dgram_open", 0, path, 0
              else
                Socket.__net__ "unix_listen", 0, path, 0
              end
      Socket.__net__("close", @handle, "", 0) if datagram? && !@handle.nil?
      @handle = taken
      @path = path
      apply_options
      return 0
    end
    port, address = Socket.unpack_sockaddr_in sockaddr
    taken = if datagram?
              Socket.__net__ "udp_open", 0, address, port
            else
              Socket.__net__ "listen", 0, address, port
            end
    # A datagram socket is already open before it is bound, so the one it
    # started on is let go once the bound one takes its place.
    Socket.__net__("close", @handle, "", 0) if datagram? && !@handle.nil?
    @handle = taken
    @bound = [address, port]
    apply_options
    0
  end

  def connect(sockaddr)
    port, address = Socket.unpack_sockaddr_in sockaddr
    if datagram?
      @handle = Socket.__net__("udp_open", 0, "0.0.0.0", 0) if @handle.nil?
      Socket.__net__ "udp_connect", @handle, address, port
      @connected_to = [address, port]
    else
      # A connection is made once. Asking for a second one on the same
      # socket is what the operating system reports as already connected.
      raise Errno::EISCONN, "Socket is already connected - connect(2)" if @connected
      @handle = Socket.__net__ "connect", 0, address, port
      @connected = true
      @connected_to = [address, port]
    end
    0
  end

  # Say how many connections may be waiting to be taken. A socket carrying
  # each message on its own has nothing to listen for.
  def listen(backlog = 5)
    wanted = Socket.integer_part backlog
    if datagram?
      raise Errno::EOPNOTSUPP, "Operation not supported on socket - listen(2)"
    end
    raise ArgumentError, "negative backlog" if wanted.negative?
    @listening = true
    0
  end

  # Where something should connect to reach this socket. A socket bound to
  # nothing has no such address, and one bound to every name on the machine
  # is reached over the loopback. The address names no protocol of its own,
  # which is what Ruby reports for one read back off a socket.
  def connect_address
    raise SocketError, "getnameinfo: ai_family not supported" if @bound.nil?
    if @socket_family == Socket::AF_UNIX
      return Addrinfo.unix(@bound.first, @socket_type)
    end
    named = Socket.__net__(datagram? ? "udp_address" : "address", @handle, "", 0)
    raise SocketError, "getnameinfo: ai_family not supported" if named.nil?
    address = named[0]
    address = "127.0.0.1" if address == "0.0.0.0"
    address = "::1" if address == "::"
    Addrinfo.built address, named[1], @socket_type, 0
  end

  # Take the next connection made to this socket, as a socket of the same
  # kind paired with the address it came from.
  def accept
    raise IOError, "closed stream" if closed?
    if @handle.nil? || (!@bound.nil? && !@listening)
      raise Errno::EINVAL, "Invalid argument - accept(2)"
    end
    handle = Socket.__net__ accept_action, @handle, "", 0
    taken = Socket.new @socket_family, @socket_type
    taken.__send__ :__take_handle__, handle
    [taken, accepted_address(handle)]
  end

  # Which of the two kinds of listener this socket is, since a socket named
  # by a path in the file system is taken from differently.
  def accept_action
    @socket_family == Socket::AF_UNIX ? "unix_accept" : "accept"
  end
  private :accept_action

  # The address a connection came from. It names the kind of socket it is,
  # and no protocol of its own, which is what `accept` reports.
  def accepted_address(handle)
    return Addrinfo.unix("") if @socket_family == Socket::AF_UNIX
    named = Socket.__net__ "peer", handle, "", 0
    return Addrinfo.built("0.0.0.0", 0, @socket_type, 0) if named.nil?
    Addrinfo.built named[0], named[1], @socket_type, 0
  end
  private :accepted_address

  # One try at taking a connection already waiting. With nothing waiting the
  # caller is told to wait, either by an exception or by the answer when it
  # asked for no exception.
  def accept_nonblock(exception: true)
    raise IOError, "closed stream" if closed?
    # A socket nothing is listening on has no connections to take, which is
    # what the operating system reports as an invalid argument.
    if @handle.nil? || (!@bound.nil? && !@listening)
      raise Errno::EINVAL, "Invalid argument - accept(2)"
    end
    handle = Socket.__net__ unix? ? "unix_accept_now" : "accept_now", @handle, "", 0
    if handle.nil?
      return :wait_readable unless exception
      raise IO::EAGAINWaitReadable, "Resource temporarily unavailable - accept(2) would block"
    end
    taken = Socket.new @socket_family, @socket_type
    taken.__send__ :__take_handle__, handle
    [taken, accepted_address(handle)]
  end

  # The number the operating system knows the next connection by, paired
  # with the address it came from.
  def sysaccept
    raise IOError, "closed stream" if closed?
    handle = Socket.__net__ accept_action, @handle, "", 0
    taken = Socket.new @socket_family, @socket_type
    taken.__send__ :__take_handle__, handle
    [taken.fileno, accepted_address(handle)]
  end

  # A message may name where it goes, which is what a socket carrying each
  # message on its own needs.
  # Send one message, saying where it goes when the socket is not already
  # pointed anywhere. Metorex carries no data alongside the message itself,
  # so anything named after the destination is not sent.
  def sendmsg(message, flags = 0, destination = nil, *_controls)
    if destination.nil? && datagram? && @connected_to.nil?
      raise Errno::EDESTADDRREQ, "Destination address required - sendmsg(2)"
    end
    named = destination.is_a?(Addrinfo) ? destination.to_sockaddr : destination
    send message, flags, named
  end

  def sendmsg_nonblock(message, flags = 0, destination = nil, *controls, exception: true)
    begin
      sendmsg message, flags.to_i | Socket::MSG_DONTWAIT, destination, *controls
    rescue Errno::EAGAIN, Errno::EWOULDBLOCK => trouble
      return :wait_writable unless exception
      raise IO::EAGAINWaitWritable, trouble.message
    end
  end

  def send(message, _flags = 0, destination = nil)
    # A socket named by a path carries no address to send to, so a
    # destination given to one is the path it is already joined to.
    if !destination.nil? && !unix?
      port, address = Socket.unpack_sockaddr_in destination
      @handle = Socket.__net__("udp_open", 0, "0.0.0.0", 0) if @handle.nil?
      Socket.__net__ "udp_connect", @handle, address, port
      @connected_to = [address, port]
    end
    if unix? && datagram?
      path = destination.nil? ? "" : Socket.unpack_sockaddr_un(destination)
      return Socket.__net__("unix_dgram_send", @handle, "#{path}\0#{message}", 0)
    end
    return Socket.__net__("udp_send", @handle, message.to_s, 0) if datagram?
    return Socket.__net__("unix_write", @handle, message.to_s, 0) if unix?
    Socket.__net__ "write", @handle, message.to_s, 0
  end

  # Whether this socket is named by a path in the file system rather than by
  # an address and a port.
  def unix?
    @socket_family == Socket::AF_UNIX
  end
  private :unix?

  alias_method :write, :send

  # Read what has arrived. A buffer handed in takes the place of its own
  # characters, keeping the encoding it was tagged with.
  def recv(length = nil, flags = 0, buffer = nil)
    held =
      if datagram?
        Socket.__net__("udp_receive", @handle, flags.to_i.to_s,
                       length.nil? ? 0 : length.to_i)[0]
      else
        # A connection the other end has finished with hands back nothing
        # at all rather than an empty string.
        raise Errno::ENOTCONN, "socket is not connected" if @handle.nil?
        taken = Socket.__net__(unix? ? "unix_read" : "read", @handle, "",
                               length.nil? ? 0 : length.to_i)
        return nil if taken.empty?
        taken
      end
    return held if buffer.nil?
    # The buffer keeps the encoding it was tagged with, since what arrived
    # is bytes rather than characters of any particular encoding.
    tagged = buffer.encoding
    buffer.replace held
    buffer.force_encoding tagged
    buffer
  end

  def read(length = nil, buffer = nil)
    recv length, 0, buffer
  end

  # A socket reads and writes lines the way a stream does.
  def gets(separator = "\n")
    ending = separator.nil? ? nil : separator.to_s
    collected = +""
    loop do
      held = recv 1
      break if held.nil? || held.empty?
      collected << held
      break if !ending.nil? && collected.end_with?(ending)
    end
    collected.empty? ? nil : collected
  end

  def puts(*lines)
    return send("\n") if lines.empty?
    lines.each do |line|
      spelled = line.to_s
      send(spelled.end_with?("\n") ? spelled : "#{spelled}\n")
    end
    nil
  end

  def print(*parts)
    parts.each { |part| send part.to_s }
    nil
  end

  def <<(text)
    send text.to_s
    self
  end

  def recv_nonblock(length = nil, flags = 0, buffer = nil, exception: true)
    begin
      held = recv length, flags.to_i | Socket::MSG_DONTWAIT, nil
    rescue Errno::EAGAIN, Errno::EWOULDBLOCK => trouble
      return :wait_readable unless exception
      raise IO::EAGAINWaitReadable, trouble.message
    end
    return held if buffer.nil?
    # The buffer keeps the encoding it was tagged with, since what arrived
    # is bytes rather than characters of any particular encoding.
    tagged = buffer.encoding
    buffer.replace held
    buffer.force_encoding tagged
    buffer
  end

  def read_nonblock(length = nil, buffer = nil, exception: true)
    recv_nonblock length, 0, buffer, exception: exception
  end

  alias_method :write_nonblock, :write

  # Read a message, saying where it came from as an address rather than as
  # the four-part array a socket of a named kind answers.
  def recvfrom(length = nil, flags = 0, buffer = nil)
    # A connection reads what arrived over it and names the other end, where
    # a socket carrying each message on its own reads one message and names
    # where that one came from.
    unless datagram?
      taken = recv length, flags
      return nil if taken.nil?
      named = Socket.__net__("peer", @handle, "", 0) || ["0.0.0.0", 0]
      return [Socket.filled_buffer(taken, buffer),
              Addrinfo.built(named[0], named[1], @socket_type, 0)]
    end
    held = Socket.__net__ "udp_receive", @handle, flags.to_i.to_s,
                          length.nil? ? 0 : length.to_i
    [Socket.filled_buffer(held[0], buffer),
     Addrinfo.built(held[1], held[2], @socket_type, 0)]
  end

  def recvfrom_nonblock(length = nil, flags = 0, buffer = nil, exception: true)
    begin
      recvfrom length, flags.to_i | Socket::MSG_DONTWAIT, buffer
    rescue Errno::EAGAIN, Errno::EWOULDBLOCK => trouble
      return :wait_readable unless exception
      raise IO::EAGAINWaitReadable, trouble.message
    end
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

  # Where this end sits. A socket that has not been bound to anything yet
  # still names the address it would be reached at, which is every name on
  # the machine and no port at all.
  def local_address
    named = Socket.__net__(datagram? ? "udp_address" : "address", @handle, "", 0)
    return Addrinfo.unix(@path.to_s) if @socket_family == Socket::AF_UNIX
    named = [@socket_family == Socket::AF_INET6 ? "::" : "0.0.0.0", 0] if named.nil?
    Addrinfo.built named[0], named[1], @socket_type, 0
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
  # Two sockets already joined to each other. A pair in the UNIX family is
  # made through a name in the file system that is removed straight away, so
  # neither end carries a name of its own.
  def self.pair(family = nil, socktype = nil, protocol = 0)
    numbered = family.nil? ? Socket::AF_INET : Socket.family_numbered(Socket.named_part(family))
    Socket.socktype_numbered Socket.named_part(socktype) unless socktype.nil?
    if numbered == Socket::AF_UNIX
      path = "#{Dir.tmpdir}/metorex_pair_#{Process.pid}_#{rand 1_000_000}.sock"
      listener = Socket.unix_server_socket path
      begin
        first = Socket.unix path
        second, = listener.accept
      ensure
        listener.close unless listener.closed?
        File.delete path if File.exist? path
      end
      return [first, second]
    end
    listener = Socket.__net__ "listen", 0, "127.0.0.1", 0
    named = Socket.__net__ "address", listener, "", 0
    first = Socket.tcp "127.0.0.1", named[1]
    second = Socket.new :INET, :STREAM
    second.__send__ :__take_handle__, Socket.__net__("accept", listener, "", 0)
    Socket.__net__ "close", listener, "", 0
    [first, second]
  end

  class << self
    alias_method :socketpair, :pair
  end

  # One network interface this machine has: what it is called, the flags it
  # carries, and the addresses it answers on.
  class Ifaddr
    attr_reader :name
    attr_reader :flags
    attr_reader :ifindex

    def initialize(name, flags, ifindex, addr, netmask, broadaddr)
      @name = name
      @flags = flags
      @ifindex = ifindex
      @addr = addr
      @netmask = netmask
      @broadaddr = broadaddr
    end

    def addr
      @addr.nil? ? nil : Addrinfo.ip(@addr)
    end

    def netmask
      @netmask.nil? ? nil : Addrinfo.ip(@netmask)
    end

    def broadaddr
      @broadaddr.nil? ? nil : Addrinfo.ip(@broadaddr)
    end

    def dstaddr
      nil
    end

    def inspect
      "#<Socket::Ifaddr #{@name} #{@addr}>"
    end
  end

  # Every network interface this machine has.
  def self.getifaddrs
    Socket.__net__("interfaces", 0, "", 0).map do |held|
      Ifaddr.new held[0], held[1], held[2], held[3], held[4], held[5]
    end
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
      # A socket answers where a message came from either as an address or
      # as the four parts one is written in.
      remote = from.is_a?(Addrinfo) ? from : Addrinfo.udp(from[2], from[1])
      source = UDPSource.new(remote, socket.local_address) do |reply|
        socket.send reply, 0, Socket.sockaddr_in(remote.ip_port, remote.ip_address)
      end
      yield message, source
    end
    nil
  end

  # What a lookup of a name or an address answers when it cannot. The code
  # says which of the resolver's own reasons it was.
  class ResolutionError < SocketError
    attr_reader :error_code

    def initialize(message = nil, error_code = nil)
      super message
      @error_code = error_code
    end
  end

  # The name and the service an address stands for. A numeric flag asks for
  # the address and the port as they are written rather than for the names
  # they resolve to.
  def self.getnameinfo(address, flags = 0)
    if address.is_a? String
      port, host = Socket.unpack_sockaddr_in address
    elsif address.is_a? Array
      if address.length < 3 || address.length > 4
        raise ArgumentError, "array size should be 3 or 4, #{address.length} given"
      end
      unless %w[AF_INET AF_INET6].include?(address[0].to_s) ||
             [Socket::AF_INET, Socket::AF_INET6].include?(address[0])
        raise Socket::ResolutionError.new("getnameinfo: ai_family not supported",
                                          Socket::EAI_FAMILY)
      end
      named = address.length >= 4 && !address[3].nil? ? address[3] : address[2]
      host = Socket.resolved(named).first
      port = address[1]
    else
      raise TypeError, "no implicit conversion of #{address.class} into String"
    end
    wanted = flags.to_i
    spelled =
      if wanted & Socket::NI_NUMERICHOST != 0
        host.to_s
      else
        Socket.__address__("name_of", host.to_s, 0) || host.to_s
      end
    service =
      if wanted & Socket::NI_NUMERICSERV != 0
        port.to_i.to_s
      else
        Socket.__net__("service_name", 0, "tcp", port.to_i) || port.to_i.to_s
      end
    [spelled, service]
  end

  # Reach a server over a connection, binding this end to a name of its own
  # when one is asked for.
  def self.tcp(host, port, local_host = nil, local_port = nil, connect_timeout: nil)
    held = Socket.new :INET, :STREAM
    held.bind Socket.sockaddr_in(local_port.to_i, local_host) unless local_host.nil?
    held.connect Socket.sockaddr_in(port, host)
    return held unless block_given?
    begin
      yield held
    ensure
      held.close unless held.closed?
    end
  end

  # The port a named service is reached on. The names are the ones the
  # operating system keeps, read through the system's own table.
  def self.getservbyname(service, protocol = "tcp")
    found = Socket.__net__ "service_port", 0, "#{service}/#{protocol}", 0
    raise SocketError, "getaddrinfo: Servname not supported for ai_socktype" if found.nil?
    found
  end

  def self.getservbyport(port, protocol = "tcp")
    found = Socket.__net__ "service_name", 0, protocol.to_s, port.to_i
    raise SocketError, "getnameinfo: Servname not supported" if found.nil?
    found
  end

  # Take connections one after another, handing each to the block with the
  # address it came from. The loop ends when the block breaks out of it.
  def self.accept_loop(*sockets)
    listening = sockets.flatten
    raise ArgumentError, "no sockets" if listening.empty?
    loop do
      listening.each do |listener|
        connection, address = listener.accept
        yield connection, address
      end
    end
  end

  # Listen on a path in the file system and hand each connection to the
  # block, closing the listener and removing the path afterwards.
  def self.unix_server_loop(path, &block)
    listener = Socket.unix_server_socket path
    begin
      accept_loop listener, &block
    ensure
      listener.close unless listener.closed?
      File.delete path if File.exist? path
    end
  end

  # Take message after message from the sockets handed in, with where each
  # one came from.
  def self.udp_server_loop_on(sockets, &block)
    loop do
      udp_server_recv sockets, &block
    end
  end

  def self.udp_server_loop(host = nil, port = nil, &block)
    host, port = nil, host if port.nil?
    udp_server_sockets(host, port) do |sockets|
      udp_server_loop_on sockets, &block
    end
  end

  def self.tcp_server_loop(host = nil, port = nil, &block)
    host, port = nil, host if port.nil?
    tcp_server_sockets(host, port) do |sockets|
      accept_loop sockets, &block
    end
  end

  # Every address this machine answers on, as far as it can be asked.
  def self.ip_address_list
    Socket.resolved(Socket.gethostname).map { |held| Addrinfo.ip held }
  rescue SocketError
    [Addrinfo.ip("127.0.0.1")]
  end

  # What an address stands for: the name it resolves to, the other names it
  # goes by, its family, and the address itself.
  def self.gethostbyaddr(address, family = nil)
    bytes = address.to_s.each_char.map { |held| held.ord }
    numbered = family.nil? ? (bytes.length == 4 ? Socket::AF_INET : Socket::AF_INET6) : Socket.family_numbered(Socket.named_part(family))
    # An address of four bytes belongs to one family and one of sixteen to
    # the other, so a family that does not match is refused.
    wanted = numbered == Socket::AF_INET6 ? 16 : 4
    unless bytes.length == wanted
      raise SocketError, "gethostbyaddr: address is not #{wanted} bytes"
    end
    spelled = if numbered == Socket::AF_INET
                bytes.join "."
              else
                written = bytes.each_slice(2)
                               .map { |pair| format("%02x%02x", pair[0], pair[1]) }
                               .join ":"
                Socket.__address__("normalize", written, 0) || written
              end
    named = Socket.__address__("name_of", spelled, 0) || spelled
    [named, [], numbered, address.to_s]
  end

  # What a host name stands for: the name itself, the other names it goes
  # by, which family its addresses belong to, and each address as the bytes
  # the operating system holds it in.
  def self.gethostbyname(host)
    found = Socket.resolved host
    kind = Socket.__address__("family", found.first, 0) == 4 ? Socket::AF_INET : Socket::AF_INET6
    named = %w[<broadcast> <any>].include?(host.to_s) ? found.first : host.to_s
    [named, [], kind] + found.map { |address| Socket.__address__("bytes", address, 0) }
  end

  # Take up a handle the operating system already holds open.
  def __take_handle__(handle)
    @handle = handle
    self
  end

  # The name this socket reached a server under, which is the name the other
  # end answers to even though this end has none of its own.
  def __connected_to__(path)
    @peer_path = path
    self
  end

  private :__connected_to__

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
    held.__send__ :__connected_to__, path.to_s
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
