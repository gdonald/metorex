# The password and group databases, and the values the system reports about
# itself. The lookups are carried out by the C library underneath.
module Etc
  Passwd = Struct.new(:name, :passwd, :uid, :gid, :gecos, :dir, :shell)
  Group = Struct.new(:name, :passwd, :gid, :mem)

  __config_names__.each do |name, value|
    const_set(name, value)
  end

  module_function

  # An entry read back from the database, or nil where there was none.
  def __passwd_from__(fields)
    return nil if fields.nil?
    Passwd.new(fields[:name], fields[:passwd], fields[:uid], fields[:gid],
               fields[:gecos], fields[:dir], fields[:shell])
  end

  def __group_from__(fields)
    return nil if fields.nil?
    Group.new(fields[:name], fields[:passwd], fields[:gid], fields[:mem])
  end

  def getpwuid(uid = nil)
    uid = Process.uid if uid.nil?
    unless uid.is_a?(Integer)
      raise TypeError, "no implicit conversion of #{uid.class} into Integer"
    end
    found = __passwd_from__(__pw_by_uid__(uid))
    raise ArgumentError, "can't find user for #{uid}" if found.nil?
    found
  end

  def getpwnam(name)
    unless name.is_a?(String)
      raise TypeError, "no implicit conversion of #{name.class} into String"
    end
    found = __passwd_from__(__pw_by_name__(name))
    raise ArgumentError, "can't find user for #{name}" if found.nil?
    found
  end

  def getgrgid(gid = nil)
    gid = Process.gid if gid.nil?
    unless gid.is_a?(Integer)
      raise TypeError, "no implicit conversion of #{gid.class} into Integer"
    end
    found = __group_from__(__gr_by_gid__(gid))
    raise ArgumentError, "can't find group for #{gid}" if found.nil?
    found
  end

  def getgrnam(name)
    unless name.is_a?(String)
      raise TypeError, "no implicit conversion of #{name.class} into String"
    end
    found = __group_from__(__gr_by_name__(name))
    raise ArgumentError, "can't find group for #{name}" if found.nil?
    found
  end

  def getpwent
    __passwd_from__(__pw_next__)
  end

  def getgrent
    __group_from__(__gr_next__)
  end

  def setpwent
    __pw_rewind__
    nil
  end

  def endpwent
    __pw_end__
    nil
  end

  def setgrent
    __gr_rewind__
    nil
  end

  def endgrent
    __gr_end__
    nil
  end

  # Without a block, one entry at a time. With a block, the whole database,
  # and a walk inside another walk is refused the way Ruby refuses it.
  def passwd
    return getpwent unless block_given?
    raise RuntimeError, "parallel iteration" if @walking_passwd
    @walking_passwd = true
    begin
      setpwent
      while (entry = getpwent)
        yield entry
      end
    ensure
      endpwent
      @walking_passwd = false
    end
    nil
  end

  def group
    return getgrent unless block_given?
    raise RuntimeError, "parallel iteration" if @walking_group
    @walking_group = true
    begin
      setgrent
      while (entry = getgrent)
        yield entry
      end
    ensure
      endgrent
      @walking_group = false
    end
    nil
  end

  def getlogin
    __login__ || ENV["USER"]
  end

  def uname
    __uname__
  end

  def nprocessors
    __nprocessors__
  end

  def sysconf(name)
    __sysconf__(name)
  end

  def confstr(name)
    __confstr__(name)
  end

  def sysconfdir
    "/etc"
  end

  def systmpdir
    ENV["TMPDIR"] || "/tmp"
  end
end
