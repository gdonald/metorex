# Uniform Resource Identifiers, split into the components RFC 2396 names and
# put back together again.
module URI
  VERSION_CODE = "010003"
  VERSION = "1.0.3"

  class Error < StandardError
  end

  class InvalidURIError < Error
  end

  class InvalidComponentError < Error
  end

  class BadURIError < Error
  end

  # Characters a URI may carry unescaped, and the escaping of everything else.
  UNSAFE = /[^\-_.!~*'()a-zA-Z\d;\/?:@&=+$,\[\]]/
  ESCAPED = /%[a-fA-F\d]{2}/

  # A URI written into running text, found by the scheme and colon that opens
  # it and read to the first space.
  ABS_URI_REF = /\b[A-Za-z][A-Za-z0-9+\-.]*:[^\s]*/

  # ── The parser ─────────────────────────────────────────────────────────────

  class RFC2396_Parser
    def split(text)
      URI.split(text)
    end

    def parse(text)
      URI.parse(text)
    end

    def join(*parts)
      URI.join(*parts)
    end

    def extract(text, schemes = nil, &block)
      URI.extract(text, schemes, &block)
    end

    def make_regexp(schemes = nil)
      URI.regexp(schemes)
    end

    def escape(text, unsafe = URI::UNSAFE)
      URI.escape(text, unsafe)
    end

    def unescape(text, escaped = URI::ESCAPED)
      URI.unescape(text)
    end

    def inspect
      "#<#{self.class}>"
    end

    def to_s
      self.inspect
    end

    def ==(other)
      other.is_a?(self.class)
    end
  end

  class RFC3986_Parser < RFC2396_Parser
  end

  Parser = RFC2396_Parser
  DEFAULT_PARSER = RFC2396_Parser.new
  RFC2396_PARSER = DEFAULT_PARSER
  RFC3986_PARSER = RFC3986_Parser.new

  # ── Splitting and building ─────────────────────────────────────────────────

  SPLIT_PATTERN = /\A(?:([A-Za-z][A-Za-z0-9+\-.]*):)?(\/\/([^\/?#]*))?([^?#]*)(?:\?([^#]*))?(?:#(.*))?\z/
  AUTHORITY_PATTERN = /\A(?:([^@]*)@)?(\[[^\]]*\]|[^:]*)(?::(\d*))?\z/

  # The nine components a URI string is made of: scheme, userinfo, host, port,
  # registry, path, opaque, query, and fragment.
  def self.split(text)
    text = coerce_to_string(text)
    found = SPLIT_PATTERN.match(text)
    raise InvalidURIError, "bad URI(is not URI?): #{text}" if found.nil?
    scheme = found[1]
    authority = found[3]
    rest = found[4]
    query = found[5]
    fragment = found[6]
    userinfo = nil
    host = nil
    port = nil
    unless authority.nil?
      inside = AUTHORITY_PATTERN.match(authority)
      raise InvalidURIError, "bad URI(is not URI?): #{text}" if inside.nil?
      userinfo = inside[1]
      host = inside[2]
      port = inside[3]
      port = nil if !port.nil? && port.empty?
    end
    path = rest
    opaque = nil
    if authority.nil? && !scheme.nil? && !rest.empty? && !rest.start_with?("/")
      # An opaque part runs to the fragment, so the question mark inside one
      # opens no query of its own.
      path = nil
      opaque = query.nil? ? rest : rest + "?" + query
      query = nil
    end
    [scheme, userinfo, host, port, nil, path, opaque, query, fragment]
  end

  def self.coerce_to_string(text)
    return text.to_s if text.is_a?(String)
    if text.respond_to?(:to_str)
      converted = text.to_str
      return converted.to_s if converted.is_a?(String)
    end
    raise InvalidURIError, "bad URI(is not URI?): #{text.inspect}"
  end

  # The class a scheme names, keyed by the scheme in upper case.
  def self.scheme_list
    { "HTTP" => URI::HTTP, "HTTPS" => URI::HTTPS, "FTP" => URI::FTP,
      "LDAP" => URI::LDAP, "LDAPS" => URI::LDAPS, "MAILTO" => URI::MailTo,
      "WS" => URI::WS, "WSS" => URI::WSS, "FILE" => URI::File }
  end

  def self.for(scheme, *arguments)
    found = scheme.nil? ? nil : scheme_list[scheme.upcase]
    return Generic.new(scheme, *arguments) if found.nil?
    found.new(scheme, *arguments)
  end

  def self.parse(text)
    self.for(*split(text))
  end

  def self.join(*parts)
    raise ArgumentError, "wrong number of arguments (given 0, expected 1+)" if parts.empty?
    built = parts[0].is_a?(Generic) ? parts[0] : parse(coerce_to_string(parts[0]))
    parts[1..-1].each { |part| built = built.merge(part) }
    built
  end

  # A pattern that finds URIs written into running text, narrowed to the
  # schemes named when any are.
  def self.regexp(schemes = nil)
    return ABS_URI_REF if schemes.nil?
    # No scheme is acceptable, so nothing can match.
    return /[^\s\S]/ if schemes.empty?
    Regexp.new("\\b(?:" + schemes.map { |name| Regexp.escape(name) }.join("|") + "):[^\\s]*")
  end

  def self.extract(text, schemes = nil, &block)
    if schemes.is_a?(Array) == false && !schemes.nil?
      schemes = nil
    end
    found = coerce_to_string(text).scan(regexp(schemes))
    return found if block.nil?
    found.each { |one| block.call(one) }
    nil
  end

  # ── Escaping ───────────────────────────────────────────────────────────────

  def self.escape(text, unsafe = UNSAFE)
    text.to_s.gsub(unsafe) do |character|
      character.bytes.map { |byte| "%%%02X" % byte }.join
    end
  end

  def self.unescape(text)
    text.to_s.gsub(/%[a-fA-F\d]{2}/) { |held| held[1, 2].to_i(16).chr }
  end

  def self.encode_www_form_component(text, enc = nil)
    text.to_s.gsub(/[^*\-.0-9A-Z_a-z]/) do |character|
      next "+" if character == " "
      character.bytes.map { |byte| "%%%02X" % byte }.join
    end
  end

  def self.decode_www_form_component(text, enc = nil)
    unless /\A[^%]*(?:%[0-9a-fA-F][0-9a-fA-F][^%]*)*\z/.match?(text.to_s)
      raise ArgumentError, "invalid %-encoding (#{text})"
    end
    unescape(text.to_s.gsub("+", " "))
  end

  def self.encode_www_form(pairs, enc = nil)
    pairs.map do |pair|
      key = pair[0]
      value = pair[1]
      next encode_www_form_component(key) if value.nil?
      if value.is_a?(Array)
        next value.map { |one| "#{encode_www_form_component(key)}=#{encode_www_form_component(one)}" }.join("&")
      end
      "#{encode_www_form_component(key)}=#{encode_www_form_component(value)}"
    end.join("&")
  end

  def self.decode_www_form(text, enc = nil, separator: "&", use__charset_: false, isindex: false)
    return [] if text.to_s.empty?
    text.to_s.split(separator).map do |pair|
      key, value = pair.split("=", 2)
      [decode_www_form_component(key.to_s), decode_www_form_component(value.to_s)]
    end
  end

  module Util
    def self.make_components_hash(klass, array_hash)
      if array_hash.is_a?(Hash)
        found = {}
        klass.component.each do |name|
          found[name] = array_hash[name] if array_hash.key?(name)
        end
        return found
      end
      unless array_hash.is_a?(Array)
        raise ArgumentError, "expected Array of or Hash of components of #{klass} (#{klass.component[1..-1].inspect})"
      end
      names = klass.component[1..-1]
      if array_hash.size > names.size
        raise ArgumentError, "too many arguments for #{klass}"
      end
      found = {}
      names.each_with_index { |name, index| found[name] = array_hash[index] }
      found
    end
  end

  # ── The general shape of a URI ─────────────────────────────────────────────

  class Generic
    include Comparable

    COMPONENT = [:scheme, :userinfo, :host, :port, :registry, :path, :opaque,
                 :query, :fragment].freeze
    DEFAULT_PORT = nil

    def self.component
      self::COMPONENT
    end

    def self.default_port
      self::DEFAULT_PORT
    end

    def self.build(arguments)
      found = URI::Util.make_components_hash(self, arguments)
      ordered = component[1..-1].map { |name| found[name] }
      new(nil, *ordered)
    end

    def self.build2(arguments)
      build(arguments)
    end

    def initialize(scheme, userinfo, host, port, registry, path, opaque,
                   query, fragment, parser = DEFAULT_PARSER, arg_check = false)
      @parser = parser
      @scheme = scheme.nil? ? nil : scheme.to_s.downcase
      @user = nil
      @password = nil
      self.set_userinfo(userinfo)
      @host = host
      @port = port.nil? || port.to_s.empty? ? self.default_port : port.to_i
      @registry = registry
      @path = path
      @opaque = opaque
      @query = query
      @fragment = fragment
    end

    def parser
      @parser
    end

    def component
      self.class.component
    end

    def component_ary
      self.component.map { |name| self.send(name) }
    end

    def default_port
      self.class.default_port
    end

    # ── Reading the components ───────────────────────────────────────────────

    def scheme
      @scheme
    end

    def userinfo
      return nil if @user.nil?
      return @user if @password.nil?
      "#{@user}:#{@password}"
    end

    def user
      @user
    end

    def password
      @password
    end

    def host
      @host
    end

    def hostname
      held = self.host
      return held if held.nil?
      held.start_with?("[") && held.end_with?("]") ? held[1..-2] : held
    end

    def port
      @port
    end

    def registry
      @registry
    end

    def path
      @path
    end

    def opaque
      @opaque
    end

    def query
      @query
    end

    def fragment
      @fragment
    end

    def absolute?
      !@scheme.nil?
    end

    def absolute
      self.absolute?
    end

    def relative?
      @scheme.nil?
    end

    def hierarchical?
      !@path.nil?
    end

    def opaque?
      !@opaque.nil?
    end

    # ── Writing the components ───────────────────────────────────────────────

    def set_scheme(v)
      @scheme = v.nil? ? nil : v.to_s.downcase
    end

    def scheme=(v)
      self.set_scheme(v)
      v
    end

    def set_userinfo(v)
      if v.nil?
        @user = nil
        @password = nil
        return v
      end
      if v.is_a?(Array)
        @user = v[0]
        @password = v[1]
        return v
      end
      name, secret = v.to_s.split(":", 2)
      @user = name
      @password = secret
      v
    end

    def userinfo=(v)
      self.refuse_opaque("userinfo")
      self.set_userinfo(v)
      v
    end

    def user=(v)
      self.refuse_opaque("user")
      @user = v
      v
    end

    def password=(v)
      self.refuse_opaque("password")
      if !v.nil? && @user.nil?
        raise InvalidURIError, "password component depends user component"
      end
      @password = v
      v
    end

    def host=(v)
      self.refuse_opaque("host")
      @host = v
      v
    end

    def hostname=(v)
      self.host = v
    end

    def port=(v)
      self.refuse_opaque("port")
      @port = v.nil? ? nil : v.to_i
      v
    end

    def path=(v)
      self.refuse_opaque("path")
      self.set_path(v)
      v
    end

    def set_path(v)
      @path = v
    end

    def query=(v)
      self.refuse_opaque("query")
      @query = v
      v
    end

    def set_query(v)
      @query = v
    end

    def fragment=(v)
      @fragment = v.nil? ? nil : v.to_s
      v
    end

    def set_fragment(v)
      @fragment = v
    end

    def opaque=(v)
      unless @host.nil? && @port.nil? && @user.nil? && (@path.nil? || @path.empty?)
        raise InvalidURIError, "can not set opaque with host, port, userinfo or path"
      end
      @opaque = v
      v
    end

    def registry=(v)
      raise InvalidURIError, "can not set registry"
    end

    # A component that lives inside a hierarchical URI cannot be written on an
    # opaque one, which carries the whole of itself after the colon.
    def refuse_opaque(name)
      return if @opaque.nil?
      raise InvalidURIError, "can not set #{name} with opaque"
    end
    private :refuse_opaque

    def select(*names)
      names.map do |name|
        unless name.is_a?(Symbol) && self.component.include?(name)
          raise ArgumentError, "expected of components of #{self.class} (#{self.component.inspect})"
        end
        self.send(name)
      end
    end

    def coerce(other)
      other = URI.parse(other) if other.is_a?(String)
      [other, self]
    end

    # ── Comparing ────────────────────────────────────────────────────────────

    def normalize
      copy = self.dup
      copy.set_path("/") if copy.path.nil? || copy.path.empty?
      copy.set_scheme(copy.scheme) unless copy.scheme.nil?
      copy.set_host(copy.host.downcase) unless copy.host.nil?
      copy
    end

    def normalize!
      copy = self.normalize
      @scheme = copy.scheme
      @host = copy.host
      @path = copy.path
      self
    end

    def set_host(v)
      @host = v
    end

    def ==(other)
      return false unless other.is_a?(Generic)
      return false unless self.class == other.class
      self.normalize.component_ary == other.normalize.component_ary
    end

    def eql?(other)
      return false unless other.is_a?(Generic)
      self.class == other.class && self.component_ary == other.component_ary
    end

    def hash
      self.component_ary.hash
    end

    def <=>(other)
      return nil unless other.is_a?(Generic)
      self.to_s <=> other.to_s
    end

    # ── Writing a URI back out ───────────────────────────────────────────────

    def to_s
      built = ""
      built = built + @scheme + ":" unless @scheme.nil?
      unless @opaque.nil?
        built = built + @opaque
      else
        unless @host.nil? && @user.nil? && @port.nil?
          built = built + "//"
          built = built + self.userinfo + "@" unless self.userinfo.nil?
          built = built + @host unless @host.nil?
          if !@port.nil? && @port != self.default_port
            built = built + ":" + @port.to_s
          end
        end
        built = built + self.rendered_path unless self.rendered_path.nil?
      end
      built = built + "?" + @query unless @query.nil?
      built = built + "#" + @fragment unless @fragment.nil?
      built
    end

    # The path as it is written into a URI string, which the schemes that keep
    # theirs in another shape override.
    def rendered_path
      @path
    end

    def to_str
      self.to_s
    end

    def inspect
      "#<#{self.class} #{self.to_s}>"
    end

    # ── Joining one URI onto another ─────────────────────────────────────────

    def merge(other)
      other = URI.parse(URI.coerce_to_string(other)) unless other.is_a?(Generic)
      raise BadURIError, "both URI are relative" if self.relative?
      return other if other.absolute?
      userinfo = self.userinfo
      host = self.host
      port = self.port
      path = self.path
      opaque = self.opaque
      query = other.query
      if !other.host.nil? || !other.user.nil? || !other.port.nil?
        userinfo = other.userinfo
        host = other.host
        port = other.port
        path = URI.remove_dot_segments(other.path)
        opaque = other.opaque
      elsif !other.opaque.nil?
        opaque = other.opaque
        path = nil
      elsif other.path.nil? || other.path.empty?
        # A reference with no path of its own names the same document, so
        # the path and, when it asks nothing else, the query both stand.
        query = self.query if other.query.nil?
      elsif other.path.start_with?("/")
        path = URI.remove_dot_segments(other.path)
        opaque = nil
      else
        path = URI.remove_dot_segments(URI.merge_paths(self.path.to_s, other.path))
        opaque = nil
      end
      URI.for(self.scheme, userinfo, host, port, nil, path, opaque, query,
              other.fragment)
    end

    def merge!(other)
      joined = self.merge(other)
      @scheme = joined.scheme
      @user = joined.user
      @password = joined.password
      @host = joined.host
      @port = joined.port
      @path = joined.path
      @query = joined.query
      @fragment = joined.fragment
      @opaque = joined.opaque
      self
    end

    def +(other)
      self.merge(other)
    end

    def set_port(v)
      @port = v
    end

    def route_from(other)
      other = URI.parse(URI.coerce_to_string(other)) unless other.is_a?(Generic)
      raise BadURIError, "both URI are relative" if self.relative? || other.relative?
      return self.dup if self.scheme != other.scheme
      relative = URI::Generic.new(nil, self.userinfo, self.host, self.port, nil,
                                  self.path, self.opaque, self.query, self.fragment)
      if relative.userinfo != other.userinfo ||
         relative.host.to_s.downcase != other.host.to_s.downcase ||
         relative.port != other.port
        return self.dup if self.userinfo.nil? && self.host.nil?
        relative.set_port(nil) if relative.port == other.default_port
        return relative
      end
      relative.set_userinfo(nil)
      relative.set_host(nil)
      relative.set_port(nil)
      if !relative.path.nil? && relative.path == other.path
        relative.set_path("")
        relative.set_query(nil) if relative.query == other.query
        return relative
      elsif !relative.opaque.nil?
        relative.set_opaque("")
        relative.set_query(nil)
        return relative
      end
      relative.set_path(URI.route_path(other.path.to_s, self.path.to_s))
      relative.set_path("") if relative.path == "./" && !self.query.nil?
      relative
    end

    def set_opaque(v)
      @opaque = v
    end

    def route_to(other)
      other = URI.parse(URI.coerce_to_string(other)) unless other.is_a?(Generic)
      other.route_from(self)
    end

    def -(other)
      self.route_from(other)
    end

    def find_proxy(env = ENV)
      nil
    end
  end

  # The path a relative URI needs to reach `destination` from `source`.
  def self.route_path(source, destination)
    return "./" if source == destination
    # A `.` or `..` segment names no place a relative path can reach from
    # somewhere else, so a destination carrying one is written whole.
    if destination.split("/").any? { |segment| segment == "." || segment == ".." }
      return destination
    end
    source_parts = source.scan(/(?:\A|[^\/]+)\//)
    destination_parts = destination.scan(/(?:\A|[^\/]+)\/?/)
    while !destination_parts.empty? && destination_parts[0] == source_parts[0]
      source_parts.shift
      destination_parts.shift
    end
    rest = destination_parts.join
    if source_parts.empty?
      return "./" if rest.empty?
      return "./" + rest if !destination_parts.empty? && destination_parts[0].include?(":")
      return rest
    end
    "../" * source_parts.size + rest
  end

  # A relative path laid over the directory the base path names.
  def self.merge_paths(base, relative)
    return "/" + relative if base.nil? || base.empty?
    cut = base.rindex("/")
    return relative if cut.nil?
    base[0, cut + 1] + relative
  end

  # A path with its `.` and `..` segments worked out, the way RFC 3986 says.
  def self.remove_dot_segments(path)
    return path if path.nil?
    rooted = path.start_with?("/")
    trailing = path.end_with?("/") || path.end_with?("/.") || path.end_with?("/..") ||
      path == "." || path == ".."
    kept = []
    path.split("/", -1).each do |segment|
      next if segment == "." || segment.empty?
      if segment == ".."
        kept.pop
        next
      end
      kept.push(segment)
    end
    built = kept.join("/")
    built = "/" + built if rooted
    built = built + "/" if trailing && !built.end_with?("/")
    built = "/" if built.empty? && rooted
    built
  end

  # ── The schemes ────────────────────────────────────────────────────────────

  class HTTP < Generic
    DEFAULT_PORT = 80
    COMPONENT = [:scheme, :userinfo, :host, :port, :path, :query,
                 :fragment].freeze

    def self.build(arguments)
      found = URI::Util.make_components_hash(self, arguments)
      new(found[:scheme] || "http", found[:userinfo], found[:host], found[:port],
          nil, found[:path], nil, found[:query], found[:fragment])
    end

    def request_uri
      return nil if @path.nil?
      written = @path.empty? ? "/" : @path
      @query.nil? ? written : written + "?" + @query
    end
  end

  class HTTPS < HTTP
    DEFAULT_PORT = 443
  end

  class WS < Generic
    DEFAULT_PORT = 80
    COMPONENT = [:scheme, :userinfo, :host, :port, :path, :query].freeze

    def request_uri
      return nil if @path.nil?
      written = @path.empty? ? "/" : @path
      @query.nil? ? written : written + "?" + @query
    end
  end

  class WSS < WS
    DEFAULT_PORT = 443
  end

  class File < Generic
    DEFAULT_PORT = nil
    COMPONENT = [:scheme, :host, :path].freeze

    def set_userinfo(v)
      @user = nil
      @password = nil
      v
    end
  end

  class FTP < Generic
    DEFAULT_PORT = 21
    COMPONENT = [:scheme, :userinfo, :host, :port, :path, :typecode].freeze
    TYPECODE = ["a", "i", "d"].freeze
    TYPECODE_PREFIX = ";type=".freeze

    def initialize(scheme, userinfo, host, port, registry, path, opaque,
                   query, fragment, parser = DEFAULT_PARSER, arg_check = false)
      @typecode = nil
      unless path.nil?
        if path =~ /\A(.*);type=([aid])\z/
          path = $1
          @typecode = $2
        end
        path = path.sub(/\A\//, "")
        path = path.sub(/\A%2F/i, "/")
      end
      super(scheme, userinfo, host, port, registry, path, opaque, query,
            fragment, parser, arg_check)
    end

    def self.build(arguments)
      found = URI::Util.make_components_hash(self, arguments)
      made = new(found[:scheme] || "ftp", found[:userinfo], found[:host],
                 found[:port], nil, found[:path], nil, nil, nil)
      made.typecode = found[:typecode] unless found[:typecode].nil?
      made
    end

    def self.new2(user, password, host, port, path, typecode = nil, arg_check = true)
      if !typecode.nil? && !TYPECODE.include?(typecode)
        raise ArgumentError, "bad typecode is specified: #{typecode}"
      end
      userinfo = nil
      unless user.nil?
        userinfo = password.nil? ? user : "#{user}:#{password}"
      end
      made = new("ftp", userinfo, host, port, nil, "/" + path.to_s, nil, nil, nil)
      made.typecode = typecode
      made
    end

    def typecode
      @typecode
    end

    def set_typecode(v)
      @typecode = v
    end

    def typecode=(v)
      unless v.nil? || TYPECODE.include?(v)
        raise InvalidComponentError, "bad component(expected typecode): #{v}"
      end
      @typecode = v
      v
    end

    def rendered_path
      return nil if @path.nil?
      written = "/" + @path.sub(/\A\//, "%2F")
      @typecode.nil? ? written : written + TYPECODE_PREFIX + @typecode
    end
  end

  class LDAP < Generic
    DEFAULT_PORT = 389
    COMPONENT = [:scheme, :host, :port, :dn, :attributes, :scope, :filter,
                 :extensions].freeze
    SCOPE_BASE = "base".freeze
    SCOPE_ONE = "one".freeze
    SCOPE_SUB = "sub".freeze
    SCOPE = [SCOPE_ONE, SCOPE_SUB, SCOPE_BASE].freeze

    def initialize(scheme, userinfo, host, port, registry, path, opaque,
                   query, fragment, parser = DEFAULT_PARSER, arg_check = false)
      super
      self.read_ldap_parts
    end

    def self.build(arguments)
      found = URI::Util.make_components_hash(self, arguments)
      made = new(found[:scheme] || "ldap", nil, found[:host], found[:port], nil,
                 "/" + found[:dn].to_s, nil, nil, nil)
      made.attributes = found[:attributes]
      made.scope = found[:scope]
      made.filter = found[:filter]
      made.extensions = found[:extensions]
      made
    end

    # The query of an LDAP URI is four fields separated by question marks,
    # sitting after the distinguished name the path holds.
    def read_ldap_parts
      @dn = @path.nil? ? nil : @path.sub(/\A\//, "")
      fields = @query.nil? ? [] : @query.split("?", -1)
      @attributes = fields[0]
      @scope = fields[1]
      @filter = fields[2]
      @extensions = fields[3]
      [@attributes, @scope, @filter, @extensions].each_with_index do |held, index|
        next unless held == ""
      end
    end
    private :read_ldap_parts

    def write_ldap_parts
      @path = "/" + @dn.to_s
      fields = [@attributes, @scope, @filter, @extensions]
      fields.pop while !fields.empty? && fields[-1].nil?
      @query = fields.empty? ? nil : fields.map { |held| held.to_s }.join("?")
    end
    private :write_ldap_parts

    def dn
      @dn
    end

    def attributes
      @attributes
    end

    def scope
      @scope
    end

    def filter
      @filter
    end

    def extensions
      @extensions
    end

    def set_dn(v)
      @dn = v
      self.write_ldap_parts
      v
    end

    def dn=(v)
      self.set_dn(v)
    end

    def set_attributes(v)
      @attributes = v
      self.write_ldap_parts
      v
    end

    def attributes=(v)
      self.set_attributes(v)
    end

    def set_scope(v)
      @scope = v
      self.write_ldap_parts
      v
    end

    def scope=(v)
      unless v.nil? || SCOPE.include?(v)
        raise InvalidComponentError, "bad component(expected scope): #{v}"
      end
      self.set_scope(v)
    end

    def set_filter(v)
      @filter = v
      self.write_ldap_parts
      v
    end

    def filter=(v)
      self.set_filter(v)
    end

    def set_extensions(v)
      @extensions = v
      self.write_ldap_parts
      v
    end

    def extensions=(v)
      self.set_extensions(v)
    end

    def hierarchical?
      false
    end
  end

  class LDAPS < LDAP
    DEFAULT_PORT = 636
  end

  class MailTo < Generic
    DEFAULT_PORT = nil
    COMPONENT = [:scheme, :to, :headers].freeze

    # What a mail address and a header may be written with, which leaves out
    # the colon and question mark that would end the component early.
    SAFE = /\A(?:%[0-9a-fA-F][0-9a-fA-F]|[!$&-.0-9;=@A-Z_a-z~])*\z/

    def initialize(scheme, userinfo, host, port, registry, path, opaque,
                   query, fragment, parser = DEFAULT_PARSER, arg_check = false)
      super
      written = @opaque.to_s
      address, headers = written.split("?", 2)
      @to = address.nil? || address.empty? ? nil : address
      @headers = headers.nil? || headers.empty? ? [] : headers.split("&").map do |pair|
        name, value = pair.split("=", 2)
        [name.to_s, value.to_s]
      end
    end

    def self.build(arguments)
      found = URI::Util.make_components_hash(self, arguments)
      address = found[:to]
      headers = found[:headers]
      check_to(address)
      written = "mailto:" + address.to_s
      unless headers.nil? || headers.empty?
        written = written + "?" + headers.map { |held| check_header(held) }.join("&")
      end
      URI.parse(written)
    end

    def self.check_to(address)
      return true if address.nil? || address.empty?
      address.split(/[,;]/).each do |one|
        unless SAFE.match?(one)
          raise InvalidComponentError, "an address in 'to' is invalid as URI #{one.inspect}"
        end
      end
      true
    end

    # One header, written as `name=value` whether it arrived that way or as a
    # pair. A header carrying anything a URI query may not hold is refused.
    def self.check_header(held)
      written = held.is_a?(Array) ? "#{held[0]}=#{held[1]}" : held.to_s
      name, value = written.split("=", 2)
      if value.nil? || !SAFE.match?(name.to_s) || !SAFE.match?(value.to_s)
        raise InvalidComponentError, "bad component(expected opaque component): #{written}"
      end
      written
    end

    def to
      @to
    end

    def headers
      @headers
    end

    def set_to(v)
      @to = v
      self.write_mail_parts
      v
    end

    def to=(v)
      MailTo.check_to(v)
      self.set_to(v)
    end

    def set_headers(v)
      @headers = v.nil? ? [] : v.map do |held|
        held.is_a?(Array) ? [held[0].to_s, held[1].to_s] : held.to_s.split("=", 2)
      end
      self.write_mail_parts
      v
    end

    def headers=(v)
      self.set_headers(v)
    end

    def write_mail_parts
      written = @to.to_s
      unless @headers.empty?
        written = written + "?" + @headers.map { |pair| "#{pair[0]}=#{pair[1]}" }.join("&")
      end
      @opaque = written
    end
    private :write_mail_parts

    def to_mailtext
      built = "To: #{@to}\n"
      body = ""
      @headers.each do |pair|
        if pair[0] == "body"
          body = URI.unescape(pair[1])
          next
        end
        built = built + "#{pair[0].capitalize}: #{URI.unescape(pair[1])}\n"
      end
      built + "\n" + body
    end

    def to_rfc822text
      self.to_mailtext
    end
  end
end

module Kernel
  # `URI(text)` reads a URI the way `URI.parse` does, and hands back a URI
  # that arrives already parsed.
  def URI(held)
    return held if held.is_a?(URI::Generic)
    URI.parse(URI.coerce_to_string(held))
  end
  module_function :URI
end
