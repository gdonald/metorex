# Turning a name into an address and back. Metorex reads the hosts file the
# operating system keeps, which is the resolver every lookup starts with.

class Resolv
  class ResolvError < StandardError
  end

  class ResolvTimeout < ResolvError
  end

  # The names and addresses a hosts file records, read once and kept.
  class Hosts
    DefaultFileName = "/etc/hosts"

    attr_reader :filename

    def initialize filename = DefaultFileName
      @filename = filename
      @addresses = nil
      @names = nil
    end

    def lazy_initialize
      return self unless @addresses.nil?
      @addresses = {}
      @names = {}
      return self unless File.exist? @filename
      File.readlines(@filename).each do |line|
        held = line.sub(/#.*/, "").strip
        next if held.empty?
        address, *names = held.split(/\s+/)
        next if names.empty?
        @addresses[address] = (@addresses[address] || []) + names
        names.each { |name| @names[name] = (@names[name] || []) + [address] }
      end
      self
    end

    def each_address name
      lazy_initialize
      (@names[name] || []).each { |address| yield address }
    end

    def each_name address
      lazy_initialize
      (@addresses[address] || []).each { |name| yield name }
    end

    def getaddress name
      each_address(name) { |address| return address }
      raise ResolvError, "cannot interpret as address: #{name}"
    end

    def getaddresses name
      found = []
      each_address(name) { |address| found << address }
      found
    end

    def getname address
      each_name(address) { |name| return name }
      raise ResolvError, "cannot interpret as address: #{address}"
    end

    def getnames address
      found = []
      each_name(address) { |name| found << name }
      found
    end
  end

  # Every lookup goes through the resolvers in turn, and the first answer
  # wins.
  DefaultResolver = nil

  def initialize resolvers = nil
    @resolvers = resolvers.nil? ? [Hosts.new] : resolvers
  end

  attr_reader :resolvers

  def each_address name
    @resolvers.each do |resolver|
      resolver.each_address(name) { |address| yield address }
    end
  end

  def each_name address
    @resolvers.each do |resolver|
      resolver.each_name(address) { |name| yield name }
    end
  end

  def getaddress name
    each_address(name) { |address| return address }
    raise ResolvError, "cannot interpret as address: #{name}"
  end

  def getaddresses name
    found = []
    each_address(name) { |address| found << address }
    found
  end

  def getname address
    each_name(address) { |name| return name }
    raise ResolvError, "cannot interpret as address: #{address}"
  end

  def getnames address
    found = []
    each_name(address) { |name| found << name }
    found
  end

  def self.getaddress name
    DefaultResolver.getaddress name
  end

  def self.getaddresses name
    DefaultResolver.getaddresses name
  end

  def self.getname address
    DefaultResolver.getname address
  end

  def self.getnames address
    DefaultResolver.getnames address
  end
end

class Resolv
  # The resolver every class-level lookup goes through, built once the class
  # itself is in place.
  remove_const :DefaultResolver
  DefaultResolver = Resolv.new
end
