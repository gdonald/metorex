pub(super) const SOURCE: &str = r##"
class File
  # Text read from a file named by path. A count and an offset name a run of
  # bytes rather than the whole file, and the options say how the file is
  # opened and what encoding the text is read in.
  def self.read(name, *rest, **options)
    if rest.size > 2
      raise ArgumentError,
            "wrong number of arguments (given #{1 + rest.size}, expected 1..3)"
    end
    length = __read_count__ rest[0], "length"
    offset = __read_count__ rest[1], "offset"
    mode, open_options = __read_opening__ options
    unless mode.start_with?("r") || mode.include?("+")
      raise IOError, "not opened for reading"
    end
    held = if open_options.empty?
      File.open File.path(name), mode
    else
      File.open File.path(name), mode, **open_options
    end
    begin
      held.seek offset unless offset.nil? || offset == 0
      length.nil? ? held.read : held.read(length)
    ensure
      held.close
    end
  end

  # A count of bytes a read was given, which stands for no bound when it is
  # nil and is refused when it counts backwards.
  def self.__read_count__(held, named)
    return nil if held.nil?
    counted = held.is_a?(Integer) ? held : held.to_int
    raise ArgumentError, "negative #{named} #{counted} given" if counted < 0
    counted
  end
  private_class_method :__read_count__

  # The mode a read opens the file in and the options it opens it with.
  # `open_args:` names them outright and every other option is set aside.
  def self.__read_opening__(options)
    named = options[:open_args]
    if named.nil?
      held = options.reject { |key, _| key == :mode || key == :open_args }
      return [options[:mode].nil? ? "r" : options[:mode], held]
    end
    mode = nil
    held = {}
    named.each do |one|
      if one.is_a? Hash
        held = one
      elsif one.is_a? String
        mode = one
      end
    end
    [mode.nil? ? "r" : mode, held]
  end
  private_class_method :__read_opening__

  # A file standing over a descriptor the program already holds, which is
  # what a stream handed over a socket arrives as.
  def self.for_fd(number, mode = nil, **options)
    held = IO.for_fd number, mode, **options
    made = allocate
    made.__send__ :__take_stream__, held.__send__(:__stream_handle__), mode
    made
  end

  # The permission bits of the file this handle was opened on, which answers
  # 0 the way every other system call on a handle does.
  def chmod(mode)
    File.chmod mode, path
    0
  end

  # The owner and group of the file this handle was opened on.
  def chown(owner, group = nil)
    File.chown owner, group, path
    0
  end

  def __take_stream__(handle, mode)
    @handle = handle
    @__file_mode = mode.nil? ? "r" : mode
    @path = ""
    self
  end
  private :__take_stream__

  # Whether a name says where it is from the root, rather than from wherever
  # the program happens to be. A `~` says nothing about the root.
  def self.absolute_path?(name)
    File.__path_text__(name).start_with? File::SEPARATOR
  end

  # The name a path names from the root. A relative one is read from the
  # directory given, or from the one the program is working in.
  def self.absolute_path(name, directory = nil)
    held = File.__path_text__ name
    return held if held.start_with? File::SEPARATOR
    base = directory.nil? ? Dir.pwd : File.absolute_path(directory)
    File.join base, held
  end

  # The name a path names from the root, with `~`, `.`, and `..` read out and
  # the runs of separators inside it collapsed. The leading run is left as it
  # was written, which is what Ruby does with `//host/share`.
  def self.expand_path(name, directory = nil)
    held = File.__path_text__ name
    tag = held.encoding
    if held.start_with? "~"
      held = File.__expanded_home__ held
    elsif !held.start_with? File::SEPARATOR
      # Reading the directory the program works in means writing its name
      # ahead of this one, which text in an encoding without ASCII cannot
      # stand beside.
      unless Encoding.default_external.ascii_compatible?
        raise Encoding::CompatibilityError,
              "ASCII incompatible encoding: #{Encoding.default_external}"
      end
      base = directory.nil? ? Dir.pwd : File.expand_path(directory)
      held = held.empty? ? base : File.join(base, held)
    end
    answered = File.__without_dots__ held
    answered.force_encoding tag
    answered
  end

  # A name opening with `~` read as the home directory it names: the one the
  # program belongs to, or the one the user named after it does.
  def self.__expanded_home__(held)
    at = held.index File::SEPARATOR
    head = at.nil? ? held : held[0, at]
    rest = at.nil? ? "" : held[at, held.length - at]
    return Dir.home(head[1, head.length - 1]) + rest unless head == "~"
    # `HOME` says where the program's own files are. One set to nothing says
    # nothing, and one that does not start from the root names no home.
    named = ENV["HOME"]
    return Dir.home + rest if named.nil?
    if named.empty?
      raise ArgumentError, "couldn't find HOME environment -- expanding `~'"
    end
    unless named.start_with? File::SEPARATOR
      raise ArgumentError, "non-absolute home"
    end
    named + rest
  end

  # A path with `.` and `..` read out and the separators inside it collapsed.
  def self.__without_dots__(held)
    leading = ""
    rest = held
    while rest.start_with? File::SEPARATOR
      leading = leading + File::SEPARATOR
      rest = rest[1, rest.length - 1]
    end
    walked = []
    rest.split(File::SEPARATOR).each do |part|
      next if part.empty? || part == "."
      if part == ".."
        # Nothing stands above the root, so a step up from there is no step
        # at all. A relative path keeps the steps it cannot take.
        if walked.empty?
          walked.push part if leading.empty?
        elsif walked.last == ".."
          walked.push part
        else
          walked.pop
        end
        next
      end
      walked.push part
    end
    answered = leading + walked.join(File::SEPARATOR)
    answered.empty? ? "." : answered
  end

  # A file cut down to the count of bytes it is told to keep, or filled out
  # with zero bytes to reach it.
  def self.truncate(name, length)
    path = File.__path_text__ name
    counted = length
    unless counted.is_a? Integer
      unless counted.respond_to? :to_int
        named = counted.nil? ? "nil" : counted.class.to_s
        raise TypeError, "no implicit conversion of #{named} into Integer"
      end
      counted = counted.to_int
    end
    held = File.open path, "r+"
    begin
      held.truncate counted
    ensure
      held.close
    end
    0
  end

  # A path as the text it stands for, whatever it was written as.
  def self.__path_text__(name)
    # A String subclass carries the text a path is spelled with, and the path
    # itself is that text rather than the object.
    return name.to_s if name.is_a? String
    return name.to_path if name.respond_to? :to_path
    return name.to_str if name.respond_to? :to_str
    raise TypeError, "no implicit conversion of #{name.class} into String"
  end

  # The parts of a path joined with a separator between them. Two parts that
  # each carry one at the boundary keep the right one's, and a part that is
  # itself a list of parts is joined before it is used.
  def self.join(*parts)
    return "" if parts.empty?
    held = ""
    parts.each_with_index do |part, index|
      spelled = File.__joined_part__ part, []
      held = index == 0 ? spelled : File.__join_two__(held, spelled)
    end
    held.dup
  end

  # One part of a path as the text it stands for. An Array is a path of its
  # own, and one that reaches itself names no path at all.
  def self.__joined_part__(part, walking)
    if part.is_a? Array
      if walking.any? { |seen| seen.equal? part }
        raise ArgumentError, "recursive array"
      end
      inside = walking + [part]
      held = ""
      part.each_with_index do |inner, index|
        spelled = File.__joined_part__ inner, inside
        held = index == 0 ? spelled : File.__join_two__(held, spelled)
      end
      return held
    end
    spelled = if part.is_a? String
      part
    elsif part.respond_to? :to_str
      part.to_str
    elsif part.respond_to? :to_path
      part.to_path
    else
      raise TypeError, "no implicit conversion of #{part.class} into String"
    end
    if spelled.include? "\0"
      raise ArgumentError, "string contains null byte"
    end
    spelled
  end

  # Two parts of a path, one after the other. A separator at the boundary is
  # left as the part carrying it wrote it, and where both carry one the right
  # part's stands.
  def self.__join_two__(left, right)
    separator = File::SEPARATOR
    if left.end_with?(separator) && right.start_with?(separator)
      trimmed = left
      while trimmed.end_with? separator
        trimmed = trimmed[0, trimmed.length - separator.length]
      end
      return trimmed + right
    end
    if left.end_with?(separator) || right.start_with?(separator)
      return left + right
    end
    left + separator + right
  end

  def __stream_handle__
    return @handle unless @handle.nil?
    __note_encodings__
    written = @__file_mode.to_s
    both = written.include? "+"
    opening = if written.start_with?("a")
      both ? 5 : 2
    elsif written.start_with?("r")
      both ? 3 : 0
    elsif written.start_with?("w")
      both ? 4 : 1
    else
      0
    end
    @handle = if @__file_flags.nil?
      IO.__stream__ "open", 0, @__file_path.to_s, opening
    else
      IO.__stream__ "open_flags", 0, @__file_path.to_s, @__file_flags
    end
    __apply_options__
    @handle
  end

  # What the options a file was opened with say beyond the mode: the
  # encodings it reads and writes in, what ends the lines it writes, and
  # whether it reads and writes bytes.
  def __apply_options__
    held = @__file_options
    return self if held.nil?
    outer = held[:external_encoding]
    inner = held[:internal_encoding]
    if !outer.nil? || !inner.nil?
      @__file_encoding = IO.__encoding_pair__ outer, inner, ""
    end
    @binmode = true if held[:binmode]
    self
  end
  private :__apply_options__

  def path
    @__file_path
  end

  def pid
    raise IOError, "closed stream" if closed?
    nil
  end

end

# The numbers the operating system keeps about a file, presented the way Ruby
# presents them.
class File
  class Stat
    include Comparable

    def initialize(path, follow = true)
      unless path.is_a?(String)
        if path.respond_to?(:to_path)
          path = path.to_path
        elsif path.respond_to?(:to_str)
          path = path.to_str
        else
          raise TypeError, "no implicit conversion of #{path.class} into String"
        end
      end
      @path = path.to_s
      @fields = File.__stat_fields__(@path, follow)
    end

    def dev
      @fields[:dev]
    end

    def dev_major
      (@fields[:dev] >> 24) & 0xff
    end

    def dev_minor
      @fields[:dev] & 0xffffff
    end

    def ino
      @fields[:ino]
    end

    def mode
      @fields[:mode]
    end

    def nlink
      @fields[:nlink]
    end

    def uid
      @fields[:uid]
    end

    def gid
      @fields[:gid]
    end

    def rdev
      @fields[:rdev]
    end

    def rdev_major
      (@fields[:rdev] >> 24) & 0xff
    end

    def rdev_minor
      @fields[:rdev] & 0xffffff
    end

    def size
      @fields[:size]
    end

    def size?
      @fields[:size] == 0 ? nil : @fields[:size]
    end

    def blksize
      @fields[:blksize]
    end

    def blocks
      @fields[:blocks]
    end

    def atime
      Time.at(@fields[:atime])
    end

    def mtime
      Time.at(@fields[:mtime])
    end

    def ctime
      Time.at(@fields[:ctime])
    end

    def birthtime
      raise NotImplementedError, "birthtime() function is unimplemented" if @fields[:birthtime].nil?
      Time.at(@fields[:birthtime])
    end

    def ftype
      @fields[:ftype]
    end

    def file?
      @fields[:ftype] == "file"
    end

    def directory?
      @fields[:ftype] == "directory"
    end

    def symlink?
      @fields[:ftype] == "link"
    end

    def chardev?
      @fields[:ftype] == "characterSpecial"
    end

    def blockdev?
      @fields[:ftype] == "blockSpecial"
    end

    def pipe?
      @fields[:ftype] == "fifo"
    end

    def socket?
      @fields[:ftype] == "socket"
    end

    def zero?
      @fields[:size] == 0
    end

    # The permission bits, which say who may read, write, and run the file.
    def setuid?
      (@fields[:mode] & 0o4000) != 0
    end

    def setgid?
      (@fields[:mode] & 0o2000) != 0
    end

    def sticky?
      (@fields[:mode] & 0o1000) != 0
    end

    def world_readable?
      (@fields[:mode] & 0o004) == 0 ? nil : @fields[:mode] & 0o7777
    end

    def world_writable?
      (@fields[:mode] & 0o002) == 0 ? nil : @fields[:mode] & 0o7777
    end

    def owned?
      @fields[:uid] == Process.uid
    end

    # A file belongs to this process's group when its group is any of the
    # ones the process is in, not only the one it runs as.
    def grpowned?
      return true if @fields[:gid] == Process.gid
      Process.groups.include?(@fields[:gid])
    end

    def readable?
      self.permitted?(0o400, 0o040, 0o004)
    end

    def writable?
      self.permitted?(0o200, 0o020, 0o002)
    end

    def executable?
      self.permitted?(0o100, 0o010, 0o001)
    end

    def readable_real?
      self.readable?
    end

    def writable_real?
      self.writable?
    end

    def executable_real?
      self.executable?
    end

    # Whether this process may do the thing the three bits stand for, read
    # against whichever of owner, group, and other it counts as.
    def permitted?(owner, group, other)
      return true if Process.uid == 0
      return (@fields[:mode] & owner) != 0 if @fields[:uid] == Process.uid
      return (@fields[:mode] & group) != 0 if @fields[:gid] == Process.gid
      (@fields[:mode] & other) != 0
    end
    private :permitted?

    def <=>(other)
      return nil unless other.is_a?(File::Stat)
      # Both sides are read as the times they stand for, down to the part of
      # a second each keeps, so two readings of one file compare equal.
      self.mtime <=> other.mtime
    end

    def inspect
      written = "#<File::Stat dev=0x#{self.dev.to_s(16)}, ino=#{self.ino}"
      written = written + ", mode=#{"%07o" % self.mode}, nlink=#{self.nlink}"
      written = written + ", uid=#{self.uid}, gid=#{self.gid}"
      written = written + ", rdev=0x#{self.rdev.to_s(16)}, size=#{self.size}"
      written = written + ", blksize=#{self.blksize.inspect}, blocks=#{self.blocks.inspect}"
      written = written + ", atime=#{self.atime.inspect}, mtime=#{self.mtime.inspect}"
      written = written + ", ctime=#{self.ctime.inspect}"
      # Ruby writes the time a file was made only where it reads one, which
      # is not every platform, so the description matches what it writes.
      unless @fields[:birthtime].nil? || RUBY_PLATFORM.include?("linux")
        written = written + ", birthtime=#{self.birthtime.inspect}"
      end
      written + ">"
    end

    def to_s
      self.inspect
    end
  end

  def self.stat(path)
    File::Stat.new(path, true)
  end

  def self.lstat(path)
    File::Stat.new(path, false)
  end

  def self.birthtime(path)
    File::Stat.new(path, true).birthtime
  end

  def self.atime(path)
    File::Stat.new(path, true).atime
  end

  def self.mtime(path)
    File::Stat.new(path, true).mtime
  end

  def self.ctime(path)
    File::Stat.new(path, true).ctime
  end

  # The questions about a file that read what the operating system keeps
  # about it. A name with nothing behind it answers the way Ruby's does
  # rather than raising.
  def self.__stat_answer__(path, follow, missing, &block)
    begin
      held = File::Stat.new(path, follow)
    rescue SystemCallError, Errno::ENOENT
      return missing
    end
    block.call(held)
  end

  def self.ftype(path)
    File::Stat.new(path, false).ftype
  end

  def self.zero?(path)
    self.__stat_answer__(path, true, false) { |held| held.zero? }
  end

  def self.empty?(path)
    self.zero?(path)
  end

  def self.world_readable?(path)
    self.__stat_answer__(path, true, nil) { |held| held.world_readable? }
  end

  def self.world_writable?(path)
    self.__stat_answer__(path, true, nil) { |held| held.world_writable? }
  end

  def self.readable?(path)
    self.__stat_answer__(path, true, false) { |held| held.readable? }
  end

  def self.readable_real?(path)
    self.readable?(path)
  end

  def self.writable?(path)
    self.__stat_answer__(path, true, false) { |held| held.writable? }
  end

  def self.writable_real?(path)
    self.writable?(path)
  end

  def self.executable_real?(path)
    self.__stat_answer__(path, true, false) { |held| held.executable? }
  end

  def self.owned?(path)
    self.__stat_answer__(path, true, false) { |held| held.owned? }
  end

  def self.grpowned?(path)
    self.__stat_answer__(path, true, false) { |held| held.grpowned? }
  end

  def self.setuid?(path)
    self.__stat_answer__(path, true, false) { |held| held.setuid? }
  end

  def self.setgid?(path)
    self.__stat_answer__(path, true, false) { |held| held.setgid? }
  end

  def self.sticky?(path)
    self.__stat_answer__(path, true, false) { |held| held.sticky? }
  end

  def self.blockdev?(path)
    self.__stat_answer__(path, false, false) { |held| held.blockdev? }
  end

  def self.chardev?(path)
    self.__stat_answer__(path, false, false) { |held| held.chardev? }
  end

  def self.pipe?(path)
    self.__stat_answer__(path, false, false) { |held| held.pipe? }
  end

  def self.socket?(path)
    self.__stat_answer__(path, false, false) { |held| held.socket? }
  end

  # Two names stand for the same file when the device and the number the
  # filesystem keeps it under both match.
  def self.identical?(one, other)
    first = self.__stat_answer__(one, true, nil) { |held| held }
    second = self.__stat_answer__(other, true, nil) { |held| held }
    return false if first.nil? || second.nil?
    first.dev == second.dev && first.ino == second.ino
  end

  # `File.stat` and `File.lstat` read a name, and the instance forms read the
  # name the handle was opened under. The two differ over a symlink: `stat`
  # follows it to what it points at, `lstat` reports the link itself.
  def self.stat(path)
    File::Stat.new(path)
  end

  def self.lstat(path)
    File::Stat.new(path, false)
  end

  # The numbers come from the descriptor rather than the name, so a file
  # that was removed while it is still open still reports its own.
  def stat
    raise IOError, "closed stream" if closed?
    File::Stat.new("/dev/fd/#{fileno}")
  end

  # The one file a handle stands for, which Ruby answers 0 for.
  def chown(owner, group)
    File.chown(owner, group, path)
    0
  end

  def lstat
    raise IOError, "closed stream" if closed?
    File::Stat.new(self.path, false)
  end

  def atime
    stat.atime
  end

  def mtime
    stat.mtime
  end

  def ctime
    stat.ctime
  end

  def birthtime
    stat.birthtime
  end

  # A handle that was never opened has nothing behind it, so every reading
  # of it is refused rather than answered.
  def read(*)
    raise IOError, "uninitialized stream" if @__file_path.nil?
    super
  end

  # The name the handle was opened under. An IO that never came from a name
  # has none, which is what `to_path` answers for.
  # The name an object stands for: a String as it is, and anything else
  # through `to_path`. Ruby leaves the name exactly as it was written.
  def self.path(held)
    return __checked_path__ held if held.is_a? String
    unless held.respond_to? :to_path
      raise TypeError, "no implicit conversion of #{held.class} into String"
    end
    named = held.to_path
    unless named.is_a? String
      raise TypeError, "no implicit conversion of #{named.class} into String"
    end
    __checked_path__ named
  end

  # A name the operating system can take: NUL ends a C string, and a name
  # spelled in an encoding without ASCII cannot be compared with one.
  def self.__checked_path__(named)
    unless named.encoding.ascii_compatible?
      raise Encoding::CompatibilityError,
            "incompatible character encodings: #{named.encoding} and US-ASCII"
    end
    if named.include? "\0"
      raise ArgumentError, "path name contains null byte"
    end
    named
  end

  # Ruby hands back a fresh String each time, tagged the way the name it was
  # opened under was, so a program may change what it is given.
  def path
    return nil if @__file_path.nil?
    @__file_path.dup
  end

  def to_path
    path
  end

  # An IO stands for itself where one is asked for.
  def to_io
    self
  end

  # The time the file was created, which the filesystem records separately
  # from the last write.
  # A name nothing stands for is refused rather than answered with nil, the
  # way every other reading of a missing file is.
  def self.birthtime(path)
    File::Stat.new(path).birthtime
  end

  def birthtime
    File.birthtime(self.path)
  end
end

# An open directory, walked one name at a time.
# The maps that hold their entries only as long as something else does. Both
# are written here as ordinary maps, since metorex frees an object when the
# last reference to it goes and never before.
# A hook the interpreter calls as it runs. A TracePoint is built over the
# events it cares about, switched on around a block, and handed itself when
# one of those events happens. The readings it answers describe the event
# being handled, so asking for one outside a handler is refused."##;
