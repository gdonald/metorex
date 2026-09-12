# A file with a name nobody else is using, which is removed when it is
# closed with `close!` or unlinked. The file itself is a File the Tempfile
# stands in front of.

require 'delegate'
require 'tmpdir'

class Tempfile < DelegateClass(File)
  # How many names to try before giving up on finding one that is free.
  MAX_TRIES = 10

  attr_reader :path

  def self.new(basename = "", tmpdir = nil, **options)
    held = allocate
    held.send :initialize, basename, tmpdir, **options
    held
  end

  # `Tempfile.open` answers an open Tempfile, and with a block hands it over
  # and closes it afterwards without removing the file.
  def self.open(basename = "", tmpdir = nil, **options)
    held = new basename, tmpdir, **options
    return held unless block_given?
    begin
      yield held
    ensure
      held.close
    end
  end

  # `Tempfile.create` makes a file that is removed when the block ends, and
  # answers the File itself rather than a Tempfile.
  def self.create(basename = "", tmpdir = nil, **options)
    prefix, suffix = Tempfile.__prefix_and_suffix__ basename
    directory = tmpdir.nil? ? Dir.tmpdir : Tempfile.__directory_text__(tmpdir)
    # A mode names the flags the file is opened with, and the file is always
    # opened to be read and written whatever else they say.
    wanted = options[:mode]
    wanted = wanted | File::RDWR | File::CREAT | File::EXCL unless wanted.nil?
    passed = options.reject { |key, _| key == :mode || key == :anonymous }
    path = Tempfile.reserve_path [prefix, suffix], directory
    handle = File.open path, "w+", **passed
    # A file asked for anonymously is taken off the directory at once, so
    # nothing else can reach it and it names only where it was.
    if options[:anonymous]
      File.unlink path
      handle.instance_variable_set :@__file_path, directory + File::SEPARATOR
    end
    return handle unless block_given?
    begin
      yield handle
    ensure
      handle.close unless handle.closed?
      File.delete path if File.exist? path
    end
  end

  # The prefix and the suffix a name is built from. Both are text, and a pair
  # of them may be written as an Array.
  def self.__prefix_and_suffix__(basename)
    if basename.is_a? Array
      prefix = basename[0]
      suffix = basename.length > 1 ? basename[1] : ""
    else
      prefix = basename
      suffix = ""
    end
    prefix = "" if prefix.nil?
    suffix = "" if suffix.nil?
    raise ArgumentError, "unexpected prefix: #{prefix.inspect}" unless prefix.is_a? String
    raise ArgumentError, "unexpected suffix: #{suffix.inspect}" unless suffix.is_a? String
    [prefix, suffix]
  end

  # The directory a scratch file is made in, as the text it is named by.
  def self.__directory_text__(held)
    return held if held.is_a? String
    unless held.respond_to? :to_str
      raise TypeError, "no implicit conversion of #{held.class} into String"
    end
    held.to_str
  end

  # A name in the scratch directory that no file is using, made from the
  # base name and suffix the caller asked for.
  def self.reserve_path(basename, tmpdir)
    directory = tmpdir.nil? ? Dir.tmpdir : tmpdir.to_s
    prefix, suffix = basename.is_a?(Array) ? basename : [basename.to_s, ""]
    # A separator in either part would reach out of the scratch directory,
    # so it is dropped rather than followed.
    prefix = prefix.to_s.delete("/").delete("\\")
    suffix = suffix.to_s.delete("/").delete("\\")
    MAX_TRIES.times do
      stamp = "#{Process.pid}-#{rand 1000000}"
      candidate = File.join directory, "#{prefix}#{stamp}#{suffix}"
      unless File.exist? candidate
        File.write candidate, ""
        File.chmod 0600, candidate
        return candidate
      end
    end
    raise Errno::EEXIST, "no free name in #{directory}"
  end

  def initialize(basename = "", tmpdir = nil, **options)
    @path = Tempfile.reserve_path basename, tmpdir
    @unlinked = false
    @handle = File.open @path, "w+", **options
    super(@handle)
  end

  # Reopening leaves what the file holds where it is, so a Tempfile closed
  # and opened again reads back what was written.
  def open
    @handle.close unless @handle.nil? || @handle.closed?
    @handle = File.open @path, "r+"
    __setobj__ @handle
    self
  end

  def close(unlink_now = false)
    _close
    unlink if unlink_now
    nil
  end

  # `close!` closes the file and removes it, which is what a scratch file is
  # for in the first place.
  def close!
    close true
  end

  # `_close` shuts the file without touching the name, which `close` uses and
  # a subclass may reach.
  def _close
    @handle.close unless @handle.nil? || @handle.closed?
    @handle = nil
    nil
  end

  protected :_close

  def unlink
    return nil if @unlinked
    File.delete @path if File.exist? @path
    @unlinked = true
    @path = nil
    nil
  end

  alias_method :delete, :unlink

  def size
    return 0 if @path.nil?
    @handle.flush unless @handle.nil? || @handle.closed?
    File.size @path
  end

  alias_method :length, :size

  def inspect
    "#<Tempfile:#{@path}>"
  end
end
