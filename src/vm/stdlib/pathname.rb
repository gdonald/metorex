# A filesystem path as an object, with the naming operations kept apart from
# the ones that reach the filesystem.
class Pathname
  include Comparable

  SEPARATOR = "/".freeze

  def initialize(path)
    unless path.is_a?(String)
      unless path.respond_to?(:to_path)
        raise TypeError, "no implicit conversion of #{path.class} into String"
      end
      path = path.to_path
    end
    written = path.to_s
    raise ArgumentError, "pathname contains null byte" if written.include?("\0")
    @path = written
  end

  def self.pwd
    Pathname.new(Dir.pwd)
  end

  def self.getwd
    self.pwd
  end

  def self.glob(pattern, flags = 0, base: nil, **rest)
    unless rest.empty?
      raise ArgumentError, "unknown keyword: :#{rest.keys[0]}"
    end
    found = base.nil? ? Dir.glob(pattern.to_s, flags) :
      Dir.glob(pattern.to_s, flags, base: base.to_s)
    found.map { |one| Pathname.new(one) }
  end

  # ── What the path is written as ──────────────────────────────────────────

  def to_s
    @path
  end

  def to_path
    @path
  end

  def to_str
    @path
  end

  def inspect
    "#<Pathname:#{@path}>"
  end

  def ==(other)
    other.is_a?(Pathname) && other.to_s == @path
  end

  def eql?(other)
    self == other
  end

  def ===(other)
    self == other
  end

  def hash
    @path.hash
  end

  def <=>(other)
    return nil unless other.is_a?(Pathname)
    @path <=> other.to_s
  end

  def freeze
    @path.freeze
    super
  end

  def sub(pattern, replacement = nil, &block)
    return Pathname.new(@path.sub(pattern, &block)) if replacement.nil?
    Pathname.new(@path.sub(pattern, replacement))
  end

  def sub_ext(extension)
    Pathname.new(@path.sub(/\.[^.\/]*\z/, "") + extension)
  end

  # ── Reading the shape of a path ──────────────────────────────────────────

  def absolute?
    @path.start_with?(SEPARATOR)
  end

  def relative?
    !self.absolute?
  end

  def root?
    !(/\A\/+\z/ =~ @path).nil?
  end

  def mountpoint?
    false
  end

  # The last part of the path, with a suffix taken off when one is named.
  def basename(suffix = "")
    parts = @path.split(SEPARATOR).reject { |part| part.empty? }
    return Pathname.new(SEPARATOR) if parts.empty?
    last = parts[-1]
    if suffix == ".*"
      last = last.sub(/\.[^.]*\z/, "")
    elsif !suffix.empty? && last.end_with?(suffix) && last != suffix
      last = last[0, last.length - suffix.length]
    end
    Pathname.new(last)
  end

  # Everything above the last part of the path.
  def dirname
    parts = @path.split(SEPARATOR).reject { |part| part.empty? }
    return Pathname.new(self.absolute? ? SEPARATOR : ".") if parts.length <= 1
    walked = parts[0...-1].join(SEPARATOR)
    Pathname.new(self.absolute? ? SEPARATOR + walked : walked)
  end

  # The extension the last part of the path ends with, or "" when it has none.
  def extname
    last = self.basename.to_s
    found = last.rindex(".")
    return "" if found.nil? || found == 0 || found == last.length - 1
    last[found..-1]
  end

  def parent
    self + ".."
  end

  def each_filename(&block)
    parts = @path.split(SEPARATOR).reject { |part| part.empty? }
    return parts.each if block.nil?
    parts.each { |part| block.call(part) }
    nil
  end

  def descend(&block)
    walked = []
    built = nil
    self.cleaned_parts.each do |part|
      built = built.nil? ? part : (built == SEPARATOR ? built + part : built + SEPARATOR + part)
      walked.push(Pathname.new(built))
    end
    return walked.each if block.nil?
    walked.each { |one| block.call(one) }
    nil
  end

  def ascend(&block)
    walked = self.descend.to_a.reverse
    return walked.each if block.nil?
    walked.each { |one| block.call(one) }
    nil
  end

  def cleaned_parts
    parts = @path.split(SEPARATOR).reject { |part| part.empty? }
    self.absolute? ? [SEPARATOR] + parts : parts
  end
  private :cleaned_parts

  # ── Joining ──────────────────────────────────────────────────────────────

  def +(other)
    Pathname.new(Pathname.join_paths(@path, other.to_s))
  end

  def /(other)
    self + other
  end

  def join(*others)
    others.inject(self) { |built, one| built + one.to_s }
  end

  # One path laid after another, with the `.` and `..` the join produces
  # worked out. An absolute second path replaces the first outright.
  def self.join_paths(base, added)
    return added if added.start_with?(SEPARATOR)
    return base if added.empty?
    rooted = base.start_with?(SEPARATOR)
    parts = []
    (base.split(SEPARATOR) + added.split(SEPARATOR)).each do |part|
      next if part.empty? || part == "."
      if part == ".." && !parts.empty? && parts[-1] != ".."
        parts.pop
        next
      end
      if part == ".." && rooted && parts.empty?
        next
      end
      parts.push(part)
    end
    built = parts.join(SEPARATOR)
    return SEPARATOR + built if rooted
    built.empty? ? "." : built
  end

  # The way from `base` to this path, walked without touching the filesystem.
  def relative_path_from(base)
    base = Pathname.new(base.to_s) unless base.is_a?(Pathname)
    unless self.absolute? == base.absolute?
      raise ArgumentError, "different prefix: #{@path.inspect} and #{base.to_s.inspect}"
    end
    here = Pathname.clean_parts(@path)
    there = Pathname.clean_parts(base.to_s)
    shared = 0
    while shared < here.length && shared < there.length && here[shared] == there[shared]
      shared += 1
    end
    # A `..` left in the base after the shared part names a directory this
    # path knows nothing about, so no relative way there can be written.
    if there[shared..-1].include?("..")
      raise ArgumentError, "base directory may not contain #{'..'.inspect}"
    end
    walked = [".."] * (there.length - shared) + here[shared..-1]
    walked.empty? ? Pathname.new(".") : Pathname.new(walked.join(SEPARATOR))
  end

  # The parts a path names once its `.` and `..` are worked out. A leading
  # `..` on a relative path stands, since there is nothing above it to drop.
  def self.clean_parts(path)
    parts = []
    path.split(SEPARATOR).each do |part|
      next if part.empty? || part == "."
      if part == ".." && !parts.empty? && parts[-1] != ".."
        parts.pop
        next
      end
      parts.push(part)
    end
    parts
  end

  def cleanpath
    parts = Pathname.clean_parts(@path)
    return Pathname.new(SEPARATOR + parts.join(SEPARATOR)) if self.absolute?
    Pathname.new(parts.empty? ? "." : parts.join(SEPARATOR))
  end

  # ── Reaching the filesystem ──────────────────────────────────────────────

  def exist?
    File.exist?(@path)
  end

  def file?
    File.file?(@path)
  end

  def directory?
    File.directory?(@path)
  end

  def empty?
    return Dir.entries(@path).reject { |one| one == "." || one == ".." }.empty? if self.directory?
    File.size(@path) == 0
  end

  def size
    File.size(@path)
  end

  def read(*arguments)
    File.read(@path, *arguments)
  end

  def readlines(*arguments)
    File.readlines(@path, *arguments)
  end

  def write(text, *arguments)
    File.write(@path, text, *arguments)
  end

  def birthtime
    File.birthtime(@path)
  end

  def mtime
    File.mtime(@path)
  end

  def atime
    File.atime(@path)
  end

  def ctime
    File.ctime(@path)
  end

  def realpath(base = nil)
    Pathname.new(File.realpath(@path))
  end

  def realdirpath(base = nil)
    Pathname.new(File.realpath(@path))
  end

  def expand_path(base = nil)
    return Pathname.new(File.expand_path(@path)) if base.nil?
    Pathname.new(File.expand_path(@path, base.to_s))
  end

  def children(with_directory = true)
    Dir.entries(@path).reject { |one| one == "." || one == ".." }.map do |one|
      with_directory ? self + one : Pathname.new(one)
    end
  end

  def entries
    Dir.entries(@path).map { |one| Pathname.new(one) }
  end

  def glob(pattern, flags = 0, &block)
    found = Dir.glob(Pathname.join_paths(@path, pattern.to_s), flags).map do |one|
      Pathname.new(one)
    end
    return found if block.nil?
    found.each { |one| block.call(one) }
    nil
  end

  def mkdir
    Dir.mkdir(@path)
  end

  def rmdir
    Dir.rmdir(@path)
  end

  def delete
    File.delete(@path)
  end

  def unlink
    self.delete
  end

  def open(*arguments, &block)
    File.open(@path, *arguments, &block)
  end
end

module Kernel
  # `Pathname(path)` makes one, and hands back a Pathname that arrives
  # already made.
  def Pathname(path)
    return path if path.is_a?(Pathname)
    Pathname.new(path)
  end
  module_function :Pathname
end
