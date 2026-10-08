# `FileUtils` names the file operations a shell script would spell out:
# making and removing directory trees, copying, linking, moving, changing
# modes and owners, and touching files. Each is a module function, so a class
# that includes FileUtils calls them by their bare names, and each prints the
# shell command it stands for when given `verbose: true`.
require "rbconfig"

module FileUtils
  VERSION = "1.8.0"

  # A module function that stays private on FileUtils itself.
  def self.private_module_function(name)
    module_function name
    private_class_method name
  end

  def pwd
    Dir.pwd
  end
  module_function :pwd

  alias getwd pwd
  module_function :getwd

  def cd(dir, verbose: nil, &block)
    fu_output_message "cd #{dir}" if verbose
    result = Dir.chdir(dir, &block)
    fu_output_message "cd -" if verbose and block
    result
  end
  module_function :cd

  alias chdir cd
  module_function :chdir

  # Whether `new` exists and is newer than every file of `old_list` that
  # exists.
  def uptodate?(new, old_list)
    return false unless File.exist?(new)
    new_time = File.mtime(new)
    old_list.each do |old|
      next unless File.exist?(old)
      return false unless new_time > File.mtime(old)
    end
    true
  end
  module_function :uptodate?

  def remove_trailing_slash(dir)
    dir == "/" ? dir : dir.chomp("/")
  end
  private_module_function :remove_trailing_slash

  def mkdir(list, mode: nil, noop: nil, verbose: nil)
    list = fu_list(list)
    fu_output_message "mkdir #{mode ? ('-m %03o ' % mode) : ''}#{list.join ' '}" if verbose
    return if noop
    list.each { |dir| fu_mkdir dir, mode }
  end
  module_function :mkdir

  # Every directory along each path, made where it is not there yet.
  def mkdir_p(list, mode: nil, noop: nil, verbose: nil)
    list = fu_list(list)
    fu_output_message "mkdir -p #{mode ? ('-m %03o ' % mode) : ''}#{list.join ' '}" if verbose
    return *list if noop
    list.each do |item|
      path = remove_trailing_slash(item)
      missing = []
      until File.directory?(path) || File.dirname(path) == path
        missing.push path
        path = File.dirname(path)
      end
      missing.reverse_each do |dir|
        begin
          fu_mkdir dir, mode
        rescue SystemCallError
          raise unless File.directory?(dir)
        end
      end
    end
    return *list
  end
  module_function :mkdir_p

  alias mkpath mkdir_p
  alias makedirs mkdir_p
  module_function :mkpath
  module_function :makedirs

  def fu_mkdir(path, mode)
    path = remove_trailing_slash(path)
    if mode
      Dir.mkdir path, mode
      File.chmod mode, path
    else
      Dir.mkdir path
    end
  end
  private_module_function :fu_mkdir

  def rmdir(list, parents: nil, noop: nil, verbose: nil)
    list = fu_list(list)
    fu_output_message "rmdir #{parents ? '-p ' : ''}#{list.join ' '}" if verbose
    return if noop
    list.each do |dir|
      Dir.rmdir(dir = remove_trailing_slash(dir))
      next unless parents
      begin
        until (parent = File.dirname(dir)) == "." or parent == dir
          dir = parent
          Dir.rmdir(dir)
        end
      rescue Errno::ENOTEMPTY, Errno::EEXIST, Errno::ENOENT
      end
    end
  end
  module_function :rmdir

  def ln(src, dest, force: nil, noop: nil, verbose: nil)
    fu_output_message "ln#{force ? ' -f' : ''} #{[src, dest].flatten.join ' '}" if verbose
    return if noop
    fu_each_src_dest0(src, dest) do |source, target|
      remove_file target, true if force
      File.link source, target
    end
  end
  module_function :ln

  alias link ln
  module_function :link

  # Hard links of every file under each source, in directories made to match.
  def cp_lr(src, dest, noop: nil, verbose: nil, dereference_root: true, remove_destination: false)
    fu_output_message "cp -lr#{remove_destination ? ' --remove-destination' : ''} #{[src, dest].flatten.join ' '}" if verbose
    return if noop
    fu_each_src_dest(src, dest) do |source, target|
      link_entry source, target, dereference_root, remove_destination
    end
  end
  module_function :cp_lr

  def ln_s(src, dest, force: nil, relative: false, target_directory: true, noop: nil, verbose: nil)
    if relative
      return ln_sr(src, dest, force: force, target_directory: target_directory, noop: noop, verbose: verbose)
    end
    fu_output_message "ln -s#{force ? 'f' : ''}#{target_directory ? '' : 'T'} #{[src, dest].flatten.join ' '}" if verbose
    return if noop
    fu_each_src_dest0(src, dest, target_directory) do |source, target|
      remove_file target, true if force
      File.symlink source, target
    end
  end
  module_function :ln_s

  alias symlink ln_s
  module_function :symlink

  def ln_sf(src, dest, noop: nil, verbose: nil)
    ln_s src, dest, force: true, noop: noop, verbose: verbose
  end
  module_function :ln_sf

  # Symbolic links whose targets are written relative to the link, by the
  # shorter of the paths as written and as they resolve.
  def ln_sr(src, dest, target_directory: true, force: nil, noop: nil, verbose: nil)
    command = "ln -s#{force ? 'f' : ''}#{target_directory ? '' : 'T'}" if verbose
    destination_parts = nil
    resolved_destination_parts = nil
    fu_each_src_dest0(src, dest, target_directory) do |source, target|
      if target_directory
        parent = File.dirname(target)
        destination_parts = fu_split_path(parent)
        resolved_destination_parts = fu_split_path(File.realpath(parent))
      else
        destination_parts ||= fu_split_path(dest)
        resolved_destination_parts ||= fu_split_path(File.realdirpath(dest))
      end
      source_parts = fu_split_path(source)
      shared = fu_common_components(source_parts, destination_parts)
      climbs = destination_parts.size - shared
      climbs -= 1 unless target_directory
      written = fu_clean_components(*Array.new([climbs, 0].max, ".."), *source_parts[shared..-1])
      resolved_source_parts = begin
        fu_split_path(File.realdirpath(source))
      rescue StandardError
        nil
      end
      if resolved_source_parts
        shared = fu_common_components(resolved_source_parts, resolved_destination_parts)
        climbs = resolved_destination_parts.size - shared
        climbs -= 1 unless target_directory
        resolved = fu_clean_components(*Array.new([climbs, 0].max, ".."), *resolved_source_parts[shared..-1])
        written = resolved if written.size > resolved.size
      end
      source = File.join(written)
      fu_output_message [command, source, target].flatten.join(" ") if verbose
      next if noop
      remove_file target, true if force
      File.symlink source, target
    end
  end
  module_function :ln_sr

  def link_entry(src, dest, dereference_root = false, remove_destination = false)
    Entry_.new(src, nil, dereference_root).traverse do |entry|
      landing = Entry_.new(dest, entry.rel, false)
      File.unlink landing.path if remove_destination && File.file?(landing.path)
      entry.link landing.path
    end
  end
  module_function :link_entry

  def cp(src, dest, preserve: nil, noop: nil, verbose: nil)
    fu_output_message "cp#{preserve ? ' -p' : ''} #{[src, dest].flatten.join ' '}" if verbose
    return if noop
    fu_each_src_dest(src, dest) do |source, target|
      copy_file source, target, preserve
    end
  end
  module_function :cp

  alias copy cp
  module_function :copy

  def cp_r(src, dest, preserve: nil, noop: nil, verbose: nil, dereference_root: true, remove_destination: nil)
    fu_output_message "cp -r#{preserve ? 'p' : ''}#{remove_destination ? ' --remove-destination' : ''} #{[src, dest].flatten.join ' '}" if verbose
    return if noop
    fu_each_src_dest(src, dest) do |source, target|
      copy_entry source, target, preserve, dereference_root, remove_destination
    end
  end
  module_function :cp_r

  # A file, directory tree, symbolic link or special file copied, with the
  # times, owner and mode of each entry kept when `preserve` is true.
  def copy_entry(src, dest, preserve = false, dereference_root = false, remove_destination = false)
    src = File.realpath(src) if dereference_root
    copy_one = proc do |entry|
      landing = Entry_.new(dest, entry.rel, false)
      if remove_destination && (File.file?(landing.path) || File.symlink?(landing.path))
        File.unlink landing.path
      end
      entry.copy landing.path
    end
    keep_metadata = proc do |entry|
      landing = Entry_.new(dest, entry.rel, false)
      entry.copy_metadata landing.path if preserve
    end
    Entry_.new(src, nil, false).wrap_traverse(copy_one, keep_metadata)
  end
  module_function :copy_entry

  def copy_file(src, dest, preserve = false, dereference = true)
    entry = Entry_.new(src, nil, dereference)
    entry.copy_file dest
    entry.copy_metadata dest if preserve
  end
  module_function :copy_file

  def copy_stream(src, dest)
    IO.copy_stream(src, dest)
  end
  module_function :copy_stream

  def mv(src, dest, force: nil, noop: nil, verbose: nil, secure: nil)
    fu_output_message "mv#{force ? ' -f' : ''} #{[src, dest].flatten.join ' '}" if verbose
    return if noop
    fu_each_src_dest(src, dest) do |source, target|
      landing = Entry_.new(target, nil, true)
      begin
        raise Errno::EEXIST, target if landing.exist? && landing.directory?
        begin
          File.rename source, target
        rescue Errno::EXDEV, Errno::EPERM
          copy_entry source, target, true
          if secure
            remove_entry_secure source, force
          else
            remove_entry source, force
          end
        end
      rescue SystemCallError
        raise unless force
      end
    end
  end
  module_function :mv

  alias move mv
  module_function :move

  def rm(list, force: nil, noop: nil, verbose: nil)
    list = fu_list(list)
    fu_output_message "rm#{force ? ' -f' : ''} #{list.join ' '}" if verbose
    return if noop
    list.each { |path| remove_file path, force }
  end
  module_function :rm

  alias remove rm
  module_function :remove

  def rm_f(list, noop: nil, verbose: nil)
    rm list, force: true, noop: noop, verbose: verbose
  end
  module_function :rm_f

  alias safe_unlink rm_f
  module_function :safe_unlink

  def rm_r(list, force: nil, noop: nil, verbose: nil, secure: nil)
    list = fu_list(list)
    fu_output_message "rm -r#{force ? 'f' : ''} #{list.join ' '}" if verbose
    return if noop
    list.each do |path|
      if secure
        remove_entry_secure path, force
      else
        remove_entry path, force
      end
    end
  end
  module_function :rm_r

  def rm_rf(list, noop: nil, verbose: nil, secure: nil)
    rm_r list, force: true, noop: noop, verbose: verbose, secure: secure
  end
  module_function :rm_rf

  alias rmtree rm_rf
  module_function :rmtree

  # A tree removed so that a program racing this one cannot point the removal
  # elsewhere: the tree is made this process's own and closed to others first.
  # A tree in a directory anyone may write to without the sticky bit is
  # refused.
  def remove_entry_secure(path, force = false)
    unless fu_have_symlink?
      remove_entry path, force
      return
    end
    full_path = File.expand_path(path)
    status = File.lstat(full_path)
    unless status.directory?
      File.unlink full_path
      return
    end
    parent_status = File.stat(File.dirname(full_path))
    unless parent_status.world_writable?
      remove_entry path, force
      return
    end
    unless parent_status.sticky?
      raise ArgumentError, "parent directory is world writable, FileUtils#remove_entry_secure does not work; abort: #{path.inspect} (parent directory mode #{'%o' % parent_status.mode})"
    end
    owner = Process.euid
    dot_file = full_path + "/."
    held = File.lstat(dot_file)
    unless fu_stat_identical_entry?(status, held)
      File.unlink full_path
      return
    end
    File.chown owner, -1, dot_file
    File.chmod 0700, dot_file
    unless fu_stat_identical_entry?(status, File.lstat(full_path))
      File.unlink full_path
      return
    end
    root = Entry_.new(path)
    root.preorder_traverse do |entry|
      if entry.directory?
        entry.chown owner, -1
        entry.chmod 0700
      end
    end
    root.postorder_traverse do |entry|
      begin
        entry.remove
      rescue StandardError
        raise unless force
      end
    end
  rescue StandardError
    raise unless force
  end
  module_function :remove_entry_secure

  def fu_have_symlink?
    File.symlink nil, nil
  rescue NotImplementedError
    false
  rescue TypeError
    true
  end
  private_module_function :fu_have_symlink?

  def fu_stat_identical_entry?(first, second)
    first.dev == second.dev and first.ino == second.ino
  end
  private_module_function :fu_stat_identical_entry?

  def remove_entry(path, force = false)
    Entry_.new(path).postorder_traverse do |entry|
      begin
        entry.remove
      rescue StandardError
        raise unless force
      end
    end
  rescue StandardError
    raise unless force
  end
  module_function :remove_entry

  def remove_file(path, force = false)
    Entry_.new(path).remove_file
  rescue StandardError
    raise unless force
  end
  module_function :remove_file

  def remove_dir(path, force = false)
    raise Errno::ENOTDIR, path unless force or File.directory?(path)
    remove_entry path, force
  end
  module_function :remove_dir

  def compare_file(first, second)
    return false unless File.size(first) == File.size(second)
    File.open(first, "rb") do |first_stream|
      File.open(second, "rb") do |second_stream|
        return compare_stream(first_stream, second_stream)
      end
    end
  end
  module_function :compare_file

  alias identical? compare_file
  alias cmp compare_file
  module_function :identical?
  module_function :cmp

  def compare_stream(first, second)
    block_size = fu_stream_blksize(first, second)
    loop do
      first_block = first.read(block_size).to_s
      second_block = second.read(block_size).to_s
      return true if first_block.empty? && second_block.empty?
      return false unless first_block == second_block
    end
  end
  module_function :compare_stream

  # Copy each source to its destination unless the two already hold the
  # same bytes, making the destination's directory first and applying the
  # mode, owner and group asked for.
  def install(src, dest, mode: nil, owner: nil, group: nil, preserve: nil, noop: nil, verbose: nil)
    if verbose
      message = +"install -c"
      message << " -p" if preserve
      message << " -m " << mode_to_s(mode) if mode
      message << " -o #{owner}" if owner
      message << " -g #{group}" if group
      message << " " << [src, dest].flatten.join(" ")
      fu_output_message message
    end
    return if noop
    uid = fu_get_uid(owner)
    gid = fu_get_gid(group)
    fu_each_src_dest(src, dest) do |source, target|
      status = File.stat(source)
      next if File.exist?(target) and compare_file(source, target)
      remove_file target, true
      if target.end_with?("/")
        mkdir_p target
        copy_file source, target + File.basename(source)
      else
        mkdir_p File.expand_path("..", target)
        copy_file source, target
      end
      File.utime status.atime, status.mtime, target if preserve
      File.chmod fu_mode(mode, status), target if mode
      File.chown uid, gid, target if uid or gid
    end
  end
  module_function :install

  # The bits a "who" letter of a symbolic mode covers.
  def user_mask(target)
    target.each_char.inject(0) do |mask, letter|
      case letter
      when "u" then mask | 04700
      when "g" then mask | 02070
      when "o" then mask | 01007
      when "a" then mask | 07777
      else
        raise ArgumentError, "invalid 'who' symbol in file mode: #{letter}"
      end
    end
  end
  private_module_function :user_mask

  def apply_mask(mode, user_mask, operator, mode_mask)
    case operator
    when "=" then (mode & ~user_mask) | (user_mask & mode_mask)
    when "+" then mode | (user_mask & mode_mask)
    when "-" then mode & ~(user_mask & mode_mask)
    end
  end
  private_module_function :apply_mask

  # A symbolic mode such as "u+x,go-w" applied to the mode `path` has now.
  def symbolic_modes_to_i(mode_symbols, path)
    path = File.stat(path) unless File::Stat === path
    mode_symbols.split(/,/).inject(path.mode & 07777) do |current_mode, clause|
      target, *actions = clause.split(/([=+-])/)
      raise ArgumentError, "invalid file mode: #{mode_symbols}" if actions.empty?
      target = "a" if target.empty?
      who = user_mask(target)
      actions.each_slice(2) do |operator, permissions|
        assigns = operator == "="
        mode_mask = (permissions || "").each_char.inject(0) do |mask, letter|
          case letter
          when "r" then mask | 0444
          when "w" then mask | 0222
          when "x" then mask | 0111
          when "X" then path.directory? ? mask | 0111 : mask
          when "s" then mask | 06000
          when "t" then mask | 01000
          when "u", "g", "o"
            current_mode = apply_mask(current_mode, who, operator, mask) if mask.nonzero?
            assigns = false
            copied = user_mask(letter)
            (current_mode & copied) / (copied & 0111) * (who & 0111)
          else
            raise ArgumentError, "invalid 'perm' symbol in file mode: #{letter}"
          end
        end
        if mode_mask.nonzero? || assigns
          current_mode = apply_mask(current_mode, who, operator, mode_mask)
        end
      end
      current_mode
    end
  end
  private_module_function :symbolic_modes_to_i

  def fu_mode(mode, path)
    mode.is_a?(String) ? symbolic_modes_to_i(mode, path) : mode
  end
  private_module_function :fu_mode

  def mode_to_s(mode)
    mode.is_a?(String) ? mode : "%o" % mode
  end
  private_module_function :mode_to_s

  def chmod(mode, list, noop: nil, verbose: nil)
    list = fu_list(list)
    fu_output_message format("chmod %s %s", mode_to_s(mode), list.join(" ")) if verbose
    return if noop
    list.each { |path| Entry_.new(path).chmod(fu_mode(mode, path)) }
  end
  module_function :chmod

  def chmod_R(mode, list, noop: nil, verbose: nil, force: nil)
    list = fu_list(list)
    fu_output_message format("chmod -R%s %s %s", (force ? "f" : ""), mode_to_s(mode), list.join(" ")) if verbose
    return if noop
    list.each do |root|
      Entry_.new(root).traverse do |entry|
        begin
          entry.chmod(fu_mode(mode, entry.path))
        rescue StandardError
          raise unless force
        end
      end
    end
  end
  module_function :chmod_R

  def chown(user, group, list, noop: nil, verbose: nil)
    list = fu_list(list)
    fu_output_message format("chown %s %s", (group ? "#{user}:#{group}" : user || ":"), list.join(" ")) if verbose
    return if noop
    uid = fu_get_uid(user)
    gid = fu_get_gid(group)
    list.each { |path| Entry_.new(path).chown uid, gid }
  end
  module_function :chown

  def chown_R(user, group, list, noop: nil, verbose: nil, force: nil)
    list = fu_list(list)
    fu_output_message format("chown -R%s %s %s", (force ? "f" : ""), (group ? "#{user}:#{group}" : user || ":"), list.join(" ")) if verbose
    return if noop
    uid = fu_get_uid(user)
    gid = fu_get_gid(group)
    list.each do |root|
      Entry_.new(root).traverse do |entry|
        begin
          entry.chown uid, gid
        rescue StandardError
          raise unless force
        end
      end
    end
  end
  module_function :chown_R

  def fu_get_uid(user)
    return nil unless user
    case user
    when Integer then user
    when /\A\d+\z/ then user.to_i
    else
      require "etc"
      Etc.getpwnam(user) ? Etc.getpwnam(user).uid : nil
    end
  end
  private_module_function :fu_get_uid

  def fu_get_gid(group)
    return nil unless group
    case group
    when Integer then group
    when /\A\d+\z/ then group.to_i
    else
      require "etc"
      Etc.getgrnam(group) ? Etc.getgrnam(group).gid : nil
    end
  end
  private_module_function :fu_get_gid

  # Each file's times set to `mtime` or now, and a file that is not there
  # made empty unless `nocreate` is given.
  def touch(list, noop: nil, verbose: nil, mtime: nil, nocreate: nil)
    list = fu_list(list)
    if verbose
      fu_output_message "touch #{nocreate ? '-c ' : ''}#{mtime ? mtime.strftime('-t %Y%m%d%H%M.%S ') : ''}#{list.join ' '}"
    end
    return if noop
    list.each do |path|
      created = nocreate
      begin
        File.utime(mtime, mtime, path)
      rescue Errno::ENOENT
        raise if created
        File.open(path, "a") { }
        created = true
        retry if mtime
      end
    end
  end
  module_function :touch

  private

  # Reading and comparing streams in blocks the size their files prefer.
  module StreamUtils_
    private

    def fu_windows?
      false
    end

    def fu_copy_stream0(src, dest, _blksize = nil)
      IO.copy_stream(src, dest)
    end

    def fu_stream_blksize(*streams)
      streams.each do |stream|
        next unless stream.respond_to?(:stat)
        size = fu_blksize(stream.stat)
        return size if size
      end
      fu_default_blksize
    end

    def fu_blksize(status)
      size = status.blksize
      return nil if size.nil? || size == 0
      size
    end

    def fu_default_blksize
      1024
    end
  end

  include StreamUtils_
  extend StreamUtils_

  # One entry of a tree being walked: a path, or a prefix and the path
  # relative to it, with the status read once and kept.
  class Entry_
    include StreamUtils_

    def initialize(first, second = nil, dereference = false)
      @prefix = @rel = @path = nil
      if second
        @prefix = first
        @rel = second
      else
        @path = first
      end
      @deref = dereference
      @stat = nil
      @lstat = nil
    end

    def inspect
      "\#<#{self.class} #{path}>"
    end

    def path
      @path ? File.path(@path) : join(@prefix, @rel)
    end

    def prefix
      @prefix || @path
    end

    attr_reader :rel

    def dereference?
      @deref
    end

    def exist?
      lstat
      true
    rescue Errno::ENOENT
      false
    end

    def file?
      status = lstat!
      status and status.file?
    end

    def directory?
      status = lstat!
      status and status.directory?
    end

    def symlink?
      status = lstat!
      status and status.symlink?
    end

    def chardev?
      status = lstat!
      status and status.chardev?
    end

    def blockdev?
      status = lstat!
      status and status.blockdev?
    end

    def socket?
      status = lstat!
      status and status.socket?
    end

    def pipe?
      status = lstat!
      status and status.pipe?
    end

    S_IF_DOOR = 0xD000

    def door?
      status = lstat!
      status and (status.mode & 0xF000 == S_IF_DOOR)
    end

    def entries
      Dir.children(path, encoding: path.encoding).map do |name|
        Entry_.new(prefix, join(rel, name))
      end
    end

    def stat
      return @stat if @stat
      @stat = lstat && lstat.symlink? ? File.stat(path) : lstat
    end

    def stat!
      return @stat if @stat
      @stat = lstat! && lstat!.symlink? ? File.stat(path) : lstat!
    rescue SystemCallError
      nil
    end

    def lstat
      @lstat ||= dereference? ? File.stat(path) : File.lstat(path)
    end

    def lstat!
      lstat
    rescue SystemCallError
      nil
    end

    def chmod(mode)
      if symlink?
        File.lchmod mode, path if have_lchmod?
      else
        File.chmod mode, path
      end
    rescue Errno::EOPNOTSUPP
    end

    def chown(uid, gid)
      if symlink?
        File.lchown uid, gid, path if have_lchown?
      else
        File.chown uid, gid, path
      end
    end

    def link(dest)
      if directory?
        if !File.exist?(dest) and descendant_directory?(dest, path)
          raise ArgumentError, "cannot link directory %s to itself %s" % [path, dest]
        end
        begin
          Dir.mkdir dest
        rescue StandardError
          raise unless File.directory?(dest)
        end
      else
        File.link path, dest
      end
    end

    def copy(dest)
      lstat
      if file?
        copy_file dest
      elsif directory?
        if !File.exist?(dest) and descendant_directory?(dest, path)
          raise ArgumentError, "cannot copy directory %s to itself %s" % [path, dest]
        end
        begin
          Dir.mkdir dest
        rescue StandardError
          raise unless File.directory?(dest)
        end
      elsif symlink?
        File.symlink File.readlink(path), dest
      elsif chardev? || blockdev?
        raise "cannot handle device file"
      elsif socket?
        begin
          require "socket"
        rescue LoadError
          raise "cannot handle socket"
        end
        raise "cannot handle socket" unless defined?(UNIXServer)
        UNIXServer.new(dest).close
        File.chmod lstat.mode, dest
      elsif pipe?
        raise "cannot handle FIFO" unless File.respond_to?(:mkfifo)
        File.mkfifo dest, lstat.mode
      elsif door?
        raise "cannot handle door: #{path}"
      else
        raise "unknown file type: #{path}"
      end
    end

    def copy_file(dest)
      File.open(path) do |source|
        File.open(dest, "wb", source.stat.mode) do |target|
          IO.copy_stream(source, target)
        end
      end
    end

    # The times, owner and mode of this entry given to the one at `path`. An
    # owner this process may not set leaves the setuid and setgid bits out.
    def copy_metadata(path)
      status = lstat
      File.utime status.atime, status.mtime, path unless status.symlink?
      mode = status.mode
      begin
        if status.symlink?
          begin
            File.lchown status.uid, status.gid, path
          rescue NotImplementedError
          end
        else
          File.chown status.uid, status.gid, path
        end
      rescue Errno::EPERM, Errno::EACCES
        mode &= 01777
      end
      if status.symlink?
        begin
          File.lchmod mode, path
        rescue NotImplementedError, Errno::EOPNOTSUPP
        end
      else
        File.chmod mode, path
      end
    end

    def remove
      directory? ? remove_dir1 : remove_file
    end

    def remove_dir1
      Dir.rmdir path.chomp("/")
    end

    def remove_file
      File.unlink path
    end

    def preorder_traverse
      stack = [self]
      while (entry = stack.pop)
        yield entry
        stack.concat entry.entries.reverse if entry.directory?
      end
    end

    alias traverse preorder_traverse

    def postorder_traverse(&block)
      if directory?
        begin
          children = entries
        rescue Errno::EACCES
          yield self
          return
        end
        children.each { |child| child.postorder_traverse(&block) }
      end
      yield self
    end

    def wrap_traverse(before, after)
      before.call self
      entries.each { |child| child.wrap_traverse before, after } if directory?
      after.call self
    end

    private

    def have_lchmod?
      return false unless File.respond_to?(:lchmod)
      File.lchmod 0
      true
    rescue NotImplementedError
      false
    end

    def have_lchown?
      return false unless File.respond_to?(:lchown)
      File.lchown nil, nil
      true
    rescue NotImplementedError
      false
    end

    def join(dir, base)
      return File.path(dir) if not base or base == "."
      return File.path(base) if not dir or dir == "."
      File.join(dir, base)
    end

    DIRECTORY_TERM = "(?=/|\\z)"

    def descendant_directory?(descendant, ascendant)
      if File::FNM_SYSCASE.nonzero?
        File.expand_path(File.dirname(descendant)).casecmp(File.expand_path(ascendant)) == 0
      else
        File.expand_path(File.dirname(descendant)) == File.expand_path(ascendant)
      end
    end
  end

  def fu_list(arg)
    [arg].flatten.map { |path| File.path(path) }
  end
  private_module_function :fu_list

  def fu_each_src_dest(src, dest)
    fu_each_src_dest0(src, dest) do |source, target|
      raise ArgumentError, "same file: #{source} and #{target}" if fu_same?(source, target)
      yield source, target
    end
  end
  private_module_function :fu_each_src_dest

  # Each source with the path it lands at: inside `dest` when `dest` is a
  # directory or several sources are given, and `dest` itself otherwise.
  def fu_each_src_dest0(src, dest, target_directory = true)
    if (sources = Array.try_convert(src))
      unless target_directory or sources.size <= 1
        raise ArgumentError, "extra target #{sources.map { |source| File.path(source) }}"
      end
      sources.each do |source|
        source = File.path(source)
        yield source, (target_directory ? File.join(dest, File.basename(source)) : dest)
      end
    else
      src = File.path(src)
      if target_directory and File.directory?(dest)
        yield src, File.join(dest, File.basename(src))
      else
        yield src, File.path(dest)
      end
    end
  end
  private_module_function :fu_each_src_dest0

  def fu_same?(first, second)
    File.identical?(first, second)
  end
  private_module_function :fu_same?

  # The shell command a call stands for, written to `@fileutils_output` or
  # $stdout, after `@fileutils_label` when one is set.
  def fu_output_message(message)
    output = @fileutils_output if defined?(@fileutils_output)
    output ||= $stdout
    message = @fileutils_label + message if defined?(@fileutils_label)
    output.puts message
  end
  private_module_function :fu_output_message

  def fu_split_path(path)
    path = File.path(path)
    parts = []
    until (parent, base = File.split(path); parent == path or parent == ".")
      if base != ".." and parts.last == ".." and !(fu_have_symlink? && File.symlink?(path))
        parts.pop
      else
        parts << base
      end
      path = parent
    end
    parts << path
    parts.reverse!
  end
  private_module_function :fu_split_path

  def fu_common_components(target, base)
    shared = 0
    shared += 1 while target[shared]&.== base[shared]
    shared
  end
  private_module_function :fu_common_components

  def fu_clean_components(*components)
    components.shift while components.first == "."
    return components if components.empty?
    clean = [components.shift]
    walked = File.join(*clean, "")
    while (component = components.shift)
      if component == ".." and clean.last != ".." and !(fu_have_symlink? && File.symlink?(walked))
        clean.pop
        walked = walked.sub(%r{(?<=\A|/)[^/]+/\z}, "")
      else
        clean << component
        walked += component + "/"
      end
    end
    clean
  end
  private_module_function :fu_clean_components

  def fu_starting_path?(path)
    path&.start_with?("/")
  end
  private_module_function :fu_starting_path?

  # Each public command with the keyword options it takes.
  OPT_TABLE = {}
  (private_instance_methods & methods(false)).each do |name|
    keywords = instance_method(name).parameters.filter_map { |kind, keyword| keyword if kind == :key }
    OPT_TABLE[name.to_s] = keywords
  end

  public

  def self.commands
    OPT_TABLE.keys
  end

  def self.options
    OPT_TABLE.values.flatten.uniq.map(&:to_s)
  end

  def self.have_option?(name, option)
    taken = OPT_TABLE[name.to_s] or raise ArgumentError, "no such method: #{name}"
    taken.include?(option)
  end

  def self.options_of(name)
    OPT_TABLE[name.to_s].map(&:to_s)
  end

  def self.collect_method(option)
    OPT_TABLE.keys.select { |name| OPT_TABLE[name].include?(option) }
  end

  private

  LOW_METHODS = singleton_methods(false) - collect_method(:noop).map(&:intern)

  # The methods that take no `noop:` option, each doing nothing, for the
  # modules that must not touch the file system.
  module LowMethods
    private

    def _do_nothing(*) end

    ::FileUtils::LOW_METHODS.each { |name| alias_method name, :_do_nothing }
  end

  METHODS = singleton_methods - %i[private_module_function commands options have_option? options_of collect_method]

  # Every command, each always given `verbose: true`.
  module Verbose
    include FileUtils

    names = ::FileUtils.collect_method(:verbose)
    names.each do |name|
      module_eval <<~METHOD, __FILE__, __LINE__ + 1
        def #{name}(*args, **options)
          super(*args, **options, verbose: true)
        end
      METHOD
    end
    private(*names)
    extend self
    class << self
      public(*::FileUtils::METHODS)
    end
  end

  # Every command, each always given `noop: true`, and the rest doing nothing.
  module NoWrite
    include FileUtils
    include LowMethods

    names = ::FileUtils.collect_method(:noop)
    names.each do |name|
      module_eval <<~METHOD, __FILE__, __LINE__ + 1
        def #{name}(*args, **options)
          super(*args, **options, noop: true)
        end
      METHOD
    end
    private(*names)
    extend self
    class << self
      public(*::FileUtils::METHODS)
    end
  end

  # Every command printing what it would do, given `noop: true` and
  # `verbose: true`, and the rest doing nothing.
  module DryRun
    include FileUtils
    include LowMethods

    names = ::FileUtils.collect_method(:noop)
    names.each do |name|
      module_eval <<~METHOD, __FILE__, __LINE__ + 1
        def #{name}(*args, **options)
          super(*args, **options, noop: true, verbose: true)
        end
      METHOD
    end
    private(*names)
    extend self
    class << self
      public(*::FileUtils::METHODS)
    end
  end
end
