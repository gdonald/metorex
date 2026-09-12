# `Dir.tmpdir` names the directory the system sets aside for scratch files.
# The interpreter answers it, so requiring this only says the name is wanted.
# `Dir.mktmpdir` removes the tree it made through FileUtils, the way Ruby's
# own does, which is why that library comes along with this one.
require "fileutils"

class Dir
  # A scratch directory of its own, named from the prefix and suffix asked
  # for. A separator in either is dropped, so the name stays inside the
  # scratch directory rather than reaching out of it.
  def self.mktmpdir(basename = nil, parent = nil)
    root = parent.nil? ? Dir.tmpdir : parent.to_s
    # A prefix is text, and a pair of them is a prefix and a suffix. Anything
    # else names no directory.
    unless basename.nil? || basename.is_a?(String) || basename.is_a?(Array)
      raise ArgumentError, "unexpected prefix: #{basename.inspect}"
    end
    prefix, suffix = basename.is_a?(Array) ? basename : [basename.nil? ? "d" : basename.to_s, ""]
    prefix = prefix.delete("/").delete("\\")
    suffix = suffix.delete("/").delete("\\")
    made = nil
    10.times do
      candidate = File.join root, "#{prefix}#{Process.pid}-#{rand 1000000}#{suffix}"
      next if File.exist? candidate
      Dir.mkdir candidate, 0700
      made = candidate
      break
    end
    raise Errno::EEXIST, "no free name in #{root}" if made.nil?
    return made unless block_given?
    begin
      yield made
    ensure
      FileUtils.remove_entry made
    end
  end
end
