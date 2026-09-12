# `FileUtils` names the file operations a shell script would spell out: making
# and removing directory trees, copying, moving, and touching files. Metorex
# carries the ones a program reaches for, each written over File and Dir.

module FileUtils
  # Every directory along the path, made where it is not already there.
  def self.mkdir_p(paths, mode: nil, noop: false, verbose: false)
    made = []
    FileUtils.__listed__(paths).each do |path|
      walked = ""
      path.to_s.split(File::SEPARATOR).each do |part|
        walked = walked.empty? ? (path.to_s.start_with?(File::SEPARATOR) ? File::SEPARATOR + part : part) : File.join(walked, part)
        next if walked.empty? || walked == File::SEPARATOR
        next if File.directory? walked
        next if noop
        mode.nil? ? Dir.mkdir(walked) : Dir.mkdir(walked, mode)
      end
      made.push path.to_s
    end
    made
  end

  class << self
    alias_method :makedirs, :mkdir_p
    alias_method :mkpath, :mkdir_p
  end

  # A directory made where it is not already there.
  def self.mkdir(paths, mode: nil, noop: false, verbose: false)
    FileUtils.__listed__(paths).each do |path|
      next if noop
      mode.nil? ? Dir.mkdir(path.to_s) : Dir.mkdir(path.to_s, mode)
    end
    FileUtils.__listed__(paths).map { |path| path.to_s }
  end

  # A file removed, with a missing one passed over.
  def self.rm_f(paths, noop: false, verbose: false)
    FileUtils.__listed__(paths).each do |path|
      next if noop
      begin
        File.unlink path.to_s
      rescue SystemCallError
        nil
      end
    end
    nil
  end

  class << self
    alias_method :safe_unlink, :rm_f
  end

  # Everything under the paths, then the paths themselves.
  def self.rm_rf(paths, noop: false, verbose: false, secure: false)
    FileUtils.__listed__(paths).each do |path|
      next if noop
      FileUtils.remove_entry path.to_s, true
    end
    nil
  end

  class << self
    alias_method :rmtree, :rm_rf
  end

  def self.rm_r(paths, force: false, noop: false, verbose: false, secure: false)
    FileUtils.__listed__(paths).each do |path|
      next if noop
      FileUtils.remove_entry path.to_s, force
    end
    nil
  end

  def self.rm(paths, force: false, noop: false, verbose: false)
    FileUtils.__listed__(paths).each do |path|
      next if noop
      begin
        File.unlink path.to_s
      rescue SystemCallError
        raise unless force
      end
    end
    nil
  end

  class << self
    alias_method :remove, :rm
  end

  # A path and everything under it. A path that is not there is only an error
  # when the caller asked to hear about it.
  def self.remove_entry(path, force = false)
    named = path.to_s
    unless File.exist?(named) || File.symlink?(named)
      return nil if force
      raise Errno::ENOENT, named
    end
    if File.directory?(named) && !File.symlink?(named)
      Dir.children(named).each do |child|
        FileUtils.remove_entry File.join(named, child), force
      end
      Dir.rmdir named
      return nil
    end
    File.unlink named
    nil
  end

  # The same removal, done in a way that a program racing this one cannot
  # redirect. Metorex removes the tree the plain way and answers the same.
  def self.remove_entry_secure(path, force = false)
    FileUtils.remove_entry path, force
  end

  def self.remove_file(path, force = false)
    File.unlink path.to_s
    nil
  rescue SystemCallError
    raise unless force
    nil
  end

  def self.remove_dir(path, force = false)
    FileUtils.remove_entry path, force
  end

  # A file copied, byte for byte.
  def self.copy_file(source, target, preserve = false, dereference = true)
    IO.copy_stream source.to_s, target.to_s
    nil
  end

  def self.cp(sources, target, preserve: false, noop: false, verbose: false)
    FileUtils.__listed__(sources).each do |source|
      next if noop
      landing = File.directory?(target.to_s) ? File.join(target.to_s, File.basename(source.to_s)) : target.to_s
      FileUtils.copy_file source.to_s, landing
    end
    nil
  end

  class << self
    alias_method :copy, :cp
  end

  def self.mv(sources, target, force: false, noop: false, verbose: false, secure: false)
    FileUtils.__listed__(sources).each do |source|
      next if noop
      landing = File.directory?(target.to_s) ? File.join(target.to_s, File.basename(source.to_s)) : target.to_s
      File.rename source.to_s, landing
    end
    nil
  end

  class << self
    alias_method :move, :mv
  end

  # A file brought into being, or its times brought up to date.
  def self.touch(paths, noop: false, verbose: false, mtime: nil, nocreate: false)
    FileUtils.__listed__(paths).each do |path|
      next if noop
      named = path.to_s
      if File.exist? named
        held = mtime.nil? ? Time.now : mtime
        File.utime held, held, named
        next
      end
      next if nocreate
      File.open(named, "w") { |stream| stream }
    end
    nil
  end

  def self.rmdir(paths, parents: false, noop: false, verbose: false)
    FileUtils.__listed__(paths).each do |path|
      next if noop
      Dir.rmdir path.to_s
    end
    nil
  end

  def self.cd(path, verbose: false, &block)
    Dir.chdir path.to_s, &block
  end

  class << self
    alias_method :chdir, :cd
  end

  def self.pwd
    Dir.pwd
  end

  class << self
    alias_method :getwd, :pwd
  end

  def self.compare_file(first, second)
    File.read(first.to_s) == File.read(second.to_s)
  end

  class << self
    alias_method :identical?, :compare_file
    alias_method :cmp, :compare_file
  end

  def self.chmod(mode, paths, noop: false, verbose: false)
    FileUtils.__listed__(paths).each do |path|
      next if noop
      File.chmod mode, path.to_s
    end
    nil
  end

  def self.ln_s(source, target, force: false, noop: false, verbose: false)
    return nil if noop
    File.unlink target.to_s if force && File.symlink?(target.to_s)
    File.symlink source.to_s, target.to_s
    nil
  end

  class << self
    alias_method :symlink, :ln_s
  end

  # One path or many, always as a list.
  def self.__listed__(paths)
    paths.is_a?(Array) ? paths : [paths]
  end
end
