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
    path = Tempfile.reserve_path basename, tmpdir
    handle = File.open path, "w+"
    return handle unless block_given?
    begin
      yield handle
    ensure
      handle.close unless handle.closed?
      File.delete path if File.exist? path
    end
  end

  # A name in the scratch directory that no file is using, made from the
  # base name and suffix the caller asked for.
  def self.reserve_path(basename, tmpdir)
    directory = tmpdir.nil? ? Dir.tmpdir : tmpdir.to_s
    prefix, suffix = basename.is_a?(Array) ? basename : [basename.to_s, ""]
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
    @handle = File.open @path, "w+"
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
