require 'etc'

# The account this process runs as, read from the password database.
account = Etc.getpwuid
p account.class
p account.name.class
p account.uid == Process.uid
p account.gid == Process.gid
p account.dir.class
p account.shell.class
p Etc.getpwnam(account.name).uid == account.uid

# The group it runs as, read from the group database.
company = Etc.getgrgid Process.gid
p company.class
p company.name.class
p company.gid == Process.gid
p company.mem.class

# What the system says about itself.
named = Etc.uname
p named.class
p named.keys
p Etc.nprocessors >= 1
p Etc.sysconf(Etc::SC_OPEN_MAX).is_a?(Integer)
p Etc.confstr(Etc::CS_PATH).class
p Etc.sysconfdir
p Etc.systmpdir.class

# A walk through the whole database, and the refusal of one inside another.
counted = 0
Etc.passwd do |entry|
  counted += 1
end
p counted > 0

begin
  Etc.group do |outer|
    Etc.group do |inner|
    end
  end
rescue RuntimeError => error
  p error.class
end

begin
  Etc.getpwuid "me"
rescue TypeError => error
  p error.message
end
