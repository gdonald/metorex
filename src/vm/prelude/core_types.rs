pub(super) const SOURCE: &str = r##"
class Integer
  # `pow` raises the number the way `**` does. Given a modulus as well, it
  # multiplies under that modulus, so a large power stays small.
  def pow(exponent, *rest)
    return self**exponent if rest.empty?
    if rest.size > 1
      raise ArgumentError, "wrong number of arguments (given #{rest.size + 1}, expected 1..2)"
    end
    # A second argument of nil is still a second argument, and only an
    # Integer is one this counts with.
    modulus = rest[0]
    unless exponent.is_a?(Integer) && modulus.is_a?(Integer)
      raise TypeError,
            "Integer#pow() 2nd argument not allowed unless all arguments are integers"
    end
    if exponent < 0
      raise RangeError,
            "Integer#pow() 1st argument cannot be negative when 2nd argument specified"
    end
    raise ZeroDivisionError, "divided by 0" if modulus == 0
    answer = 1
    base = self % modulus
    power = exponent
    while power > 0
      answer = (answer * base) % modulus if power.odd?
      base = (base * base) % modulus
      power = power >> 1
    end
    answer
  end
end

class String
  # A string given another takes that one's characters and its encoding. A
  # string given nothing stays as it is, frozen or not.
  def initialize(other = nil)
    return self if other.nil?
    __native_replace__ other
  end
  private :initialize
end

class Struct
  # The members a struct holds, set from the values given. A struct class of
  # the program's own may write its own and reach this one with `super`.
  def initialize(*values, **keywords)
    named = self.class.members
    if values.empty? && !keywords.empty?
      keywords.each do |key, value|
        instance_variable_set "@__struct_member_#{key}", value
      end
    else
      raise ArgumentError, "struct size differs" if values.size > named.size
      named.each_with_index do |member, index|
        instance_variable_set "@__struct_member_#{member}", values[index]
      end
    end
    self
  end
  private :initialize
end

module ObjectSpace
  # Keyed by identity: two objects that are equal but not the same are two
  # keys.
  class WeakMap
    include Enumerable

    def initialize
      @entries = []
    end

    def []=(key, value)
      place = place_of(key)
      if place.nil?
        @entries.push([key, value])
      else
        @entries[place] = [key, value]
      end
      value
    end

    def [](key)
      place = place_of(key)
      place.nil? ? nil : @entries[place][1]
    end

    def delete(key)
      place = place_of(key)
      if place.nil?
        return yield(key) if block_given?
        return nil
      end
      @entries.delete_at(place)[1]
    end

    def key?(key)
      !place_of(key).nil?
    end

    def member?(key)
      key?(key)
    end

    def include?(key)
      key?(key)
    end

    def has_key?(key)
      key?(key)
    end

    def key(value)
      @entries.each do |entry|
        return entry[0] if entry[1].equal?(value)
      end
      nil
    end

    def size
      @entries.size
    end

    def length
      size
    end

    def keys
      @entries.map { |entry| entry[0] }
    end

    def values
      @entries.map { |entry| entry[1] }
    end

    # A walk with no block is refused once there is anything to walk, which
    # is what Ruby does with a map that cannot answer an enumerator.
    def each
      unless block_given?
        return self if @entries.empty?
        raise LocalJumpError, "no block given (yield)"
      end
      @entries.each { |entry| yield entry[0], entry[1] }
      self
    end

    def each_pair(&block)
      each(&block)
    end

    def each_key
      unless block_given?
        return self if @entries.empty?
        raise LocalJumpError, "no block given (yield)"
      end
      @entries.each { |entry| yield entry[0] }
      self
    end

    def each_value
      unless block_given?
        return self if @entries.empty?
        raise LocalJumpError, "no block given (yield)"
      end
      @entries.each { |entry| yield entry[1] }
      self
    end

    # The map's address, and the pair of addresses each entry holds. What a
    # key or a value writes itself as is read through Kernel, since an object
    # rooted at BasicObject answers no `inspect` of its own.
    def inspect
      written = @entries.map do |entry|
        "#{ObjectSpace::WeakMap.written(entry[0])} => #{ObjectSpace::WeakMap.written(entry[1])}"
      end
      named = format("#<ObjectSpace::WeakMap:0x%016x", object_id)
      written.empty? ? "#{named}>" : "#{named}: #{written.join(", ")}>"
    end

    def self.written(held)
      ::Kernel.instance_method(:inspect).bind(held).call
    end

    # Where a key sits, found by identity rather than by value.
    def place_of(key)
      @entries.each_with_index do |entry, place|
        return place if entry[0].equal?(key)
      end
      nil
    end
    private :place_of
  end

  # Keyed by value, and only by something the collector could free, so a
  # number or a symbol is refused as a key.
  class WeakKeyMap
    def initialize
      @entries = []
    end

    # What an object calls its class, asked without going through the object
    # itself, since one may answer nothing at all.
    def self.named(held)
      ::Kernel.instance_method(:class).bind(held).call.to_s
    end

    # Whether an object has a `hash` at all, asked of its class so an object
    # answering nothing of Kernel's can be asked too.
    def self.answers_hash?(held)
      holder = ::Kernel.instance_method(:class).bind(held).call
      named = holder.ancestors.map { |ancestor| ancestor.to_s }
      named.include?("Object") || named.include?("Kernel") || holder.method_defined?(:hash)
    end

    def []=(key, value)
      unless collectable?(key)
        raise ArgumentError, "WeakKeyMap must be garbage collectable"
      end
      # A key is found again by its hash, so one that has none cannot be
      # stored at all. An object that descends from BasicObject alone answers
      # none of the names Kernel gives, `hash` among them, so the question is
      # put to its class rather than to the object.
      unless ObjectSpace::WeakKeyMap.answers_hash?(key)
        raise NoMethodError,
              "undefined method 'hash' for an instance of #{ObjectSpace::WeakKeyMap.named(key)}"
      end
      place = place_of(key)
      if place.nil?
        @entries.push([key, value])
      else
        @entries[place][1] = value
      end
      value
    end

    def [](key)
      return nil unless collectable?(key)
      place = place_of(key)
      place.nil? ? nil : @entries[place][1]
    end

    def delete(key)
      place = collectable?(key) ? place_of(key) : nil
      if place.nil?
        return yield(key) if block_given?
        return nil
      end
      @entries.delete_at(place)[1]
    end

    def getkey(key)
      return nil unless collectable?(key)
      place = place_of(key)
      place.nil? ? nil : @entries[place][0]
    end

    def key?(key)
      return false unless collectable?(key)
      !place_of(key).nil?
    end

    def clear
      @entries = []
      self
    end

    def size
      @entries.size
    end

    def length
      size
    end

    def inspect
      "#<ObjectSpace::WeakKeyMap:0x#{format("%016x", object_id * 2)} size=#{size}>"
    end

    def to_s
      inspect
    end

    # Whether a key is something the collector could free. A number, a
    # symbol, and the three singletons live for the whole run.
    def collectable?(key)
      # An object rooted at BasicObject answers none of Kernel's names, so
      # the classes are asked about the key rather than the key about itself.
      return false if NilClass === key || TrueClass === key || FalseClass === key
      return false if Numeric === key || Symbol === key
      true
    end
    private :collectable?

    # Where a key sits. The hash decides which entries are worth comparing,
    # and the same object is its own match however its `eql?` answers.
    def place_of(key)
      wanted = key.__send__(:hash)
      @entries.each_with_index do |entry, place|
        held = entry[0]
        next unless held.__send__(:hash) == wanted
        return place if held.equal?(key) || key.__send__(:eql?, held)
      end
      nil
    end
    private :place_of
  end
end

# The file questions File answers, gathered as a module so they can be asked
# without naming File and mixed into anything that wants them.
module FileTest
  module_function

  def blockdev?(path)
    File.blockdev?(path)
  end

  def chardev?(path)
    File.chardev?(path)
  end

  def directory?(path)
    File.directory?(path)
  end

  def empty?(path)
    File.empty?(path)
  end

  def executable?(path)
    File.executable?(path)
  end

  def executable_real?(path)
    File.executable_real?(path)
  end

  def exist?(path)
    File.exist?(path)
  end

  def file?(path)
    File.file?(path)
  end

  def grpowned?(path)
    File.grpowned?(path)
  end

  def identical?(path, other)
    File.identical?(path, other)
  end

  def owned?(path)
    File.owned?(path)
  end

  def pipe?(path)
    File.pipe?(path)
  end

  def readable?(path)
    File.readable?(path)
  end

  def readable_real?(path)
    File.readable_real?(path)
  end

  def setgid?(path)
    File.setgid?(path)
  end

  def setuid?(path)
    File.setuid?(path)
  end

  def size(path)
    File.size(path)
  end

  def size?(path)
    File.size?(path)
  end

  def socket?(path)
    File.socket?(path)
  end

  def sticky?(path)
    File.sticky?(path)
  end

  def symlink?(path)
    File.symlink?(path)
  end

  def world_readable?(path)
    File.world_readable?(path)
  end

  def world_writable?(path)
    File.world_writable?(path)
  end

  def writable?(path)
    File.writable?(path)
  end

  def writable_real?(path)
    File.writable_real?(path)
  end

  def zero?(path)
    File.zero?(path)
  end
end

# A queue is built by the interpreter, and `initialize` is the private method
# Ruby reports for it.
class Queue
  def initialize(*items)
    self
  end
  private :initialize
end

class SizedQueue
  def initialize(*counted)
    self
  end
  private :initialize
end

class Dir
  include Enumerable

  # The directory a descriptor names, made the one the program works from.
  # With a block the program works from there only while the block runs.
  def self.fchdir(number)
    was = Dir.pwd
    IO.__stream__ "fchdir", 0, "", number
    return 0 unless block_given?
    begin
      yield
    ensure
      Dir.chdir was
    end
  end

  def initialize(path, **options)
    unless path.is_a?(String)
      unless path.respond_to?(:to_path)
        raise TypeError, "no implicit conversion of #{path.class} into String"
      end
      path = path.to_path
    end
    @path = path.to_s
    unless File.directory?(@path)
      missing = Errno::ENOENT.new
      # Ruby names the call and the path after the reason with no dash
      # between them, which the message an Errno builds always puts there.
      missing.__restore_message__ "No such file or directory @ dir_initialize - #{@path}"
      raise missing
    end
    @names = options.key?(:encoding) ? Dir.entries(@path, encoding: options[:encoding]) : Dir.entries(@path)
    @position = 0
    @closed = false
  end

  # The directory this handle names, made the one the program works from.
  # With a block the program works from there only while the block runs. The
  # directory the program came from is reached again through this handle, so
  # a directory removed while the block ran is not an error here.
  def chdir(&block)
    return Dir.chdir(@path) if block.nil?
    was = Dir.pwd
    Dir.chdir(@path)
    begin
      block.call
    ensure
      begin
        Dir.chdir(was)
      rescue Errno::ENOENT
      end
    end
  end

  def self.open(path, **options, &block)
    made = Dir.new(path, **options)
    return made if block.nil?
    begin
      block.call(made)
    ensure
      made.close
    end
  end

  # The path stands whether the directory is still open or not, which is what
  # Ruby answers for a closed one.
  def path
    @path
  end

  def to_path
    @path
  end

  def inspect
    "#<Dir:#{@path}>"
  end

  def closed?
    @closed
  end

  def close
    # A descriptor another Dir already closed is gone, which is what the
    # operating system says when this one is asked to close it too.
    unless @handle.nil?
      unless IO.__stream__("live?", @handle, "", 0)
        raise Errno::EBADF, "closedir"
      end
      IO.__stream__ "close", @handle, "", 0
    end
    @closed = true
    nil
  end

  # The number the operating system holds this directory under, opened the
  # first time one is asked for.
  def fileno
    self.refuse_closed
    @handle = IO.__stream__("open", 0, @path, 0) if @handle.nil?
    IO.__stream__ "fileno", @handle, "", 0
  end

  # Another Dir over a descriptor already open, which reads the same
  # directory and closes the same descriptor.
  def self.for_fd(number)
    unless number.is_a?(Integer)
      raise TypeError, "no implicit conversion from nil to integer" if number.nil?
      unless number.respond_to?(:to_int)
        named = number == true || number == false ? number.inspect : number.class
        raise TypeError, "no implicit conversion of #{named} into Integer"
      end
      number = number.to_int
    end
    made = allocate
    made.__send__ :__adopt__, number
    made
  end

  def __adopt__(number)
    @names = IO.__stream__ "fdopendir", 0, "", number
    @handle = IO.__stream__ "adopt", 0, "", number
    @path = nil
    @position = 0
    @closed = false
    self
  end

  # Every operation that walks the names needs the directory still open.
  def refuse_closed
    raise IOError, "closed directory" if @closed
  end
  private :refuse_closed

  def read
    self.refuse_closed
    return nil if @position >= @names.length
    found = @names[@position]
    @position += 1
    found
  end

  def pos
    self.refuse_closed
    @position
  end

  def tell
    self.pos
  end

  def seek(position)
    self.refuse_closed
    @position = position
    self
  end

  def pos=(position)
    self.seek(position)
    position
  end

  def rewind
    self.refuse_closed
    @position = 0
    self
  end

  # Walking the whole directory starts at the beginning and leaves the
  # position at the end, which is where a read after it finds nothing.
  def each(&block)
    self.refuse_closed
    return self.to_enum(:each) if block.nil?
    @position = 0
    @names.each { |name| block.call(name) }
    @position = @names.length
    self
  end

  def each_child(&block)
    self.refuse_closed
    walked = @names.reject { |name| name == "." || name == ".." }
    return self.to_enum(:each_child) if block.nil?
    @position = 0
    walked.each { |name| block.call(name) }
    @position = @names.length
    self
  end

  def children
    self.refuse_closed
    @names.reject { |name| name == "." || name == ".." }
  end

  def entries
    self.refuse_closed
    @names.dup
  end
end

class Array
  # An array is its own list of elements, which is what a pattern reads out
  # of it.
  def deconstruct
    self
  end
end

class Hash
  # A hash is its own set of keys, which is what a pattern reads out of it.
  # The names a pattern asked for make no difference to what is answered.
  def deconstruct_keys(keys)
    self
  end
end
"##;
