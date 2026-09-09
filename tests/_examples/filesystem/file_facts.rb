# What the operating system keeps about a file, read through File::Stat and
# the questions File asks on top of it.
here = "README.md"
held = File.stat(here)

p held.class
p held.ftype
p held.file?
p held.directory?
p held.symlink?
p (held.mode & 0777).to_s(8)
p held.size > 0
p held.size == File.size(here)
p held.uid == Process.uid
p held.gid == Process.gid
p held.owned?
p held.nlink >= 1
p held.blksize > 0
p held.atime.class
p held.mtime.class
p held.zero?
p held.readable?
p held.world_writable?

p File.stat("src").ftype
p File.zero?(here)
p File.zero?("no_such_file_at_all")
p File.ftype("src")
p File.identical?(here, here)
p File.identical?(here, "src")
p File.setuid?(here)
p File.sticky?(here)
p File.blockdev?(here)
p File.chardev?(here)
p File.pipe?(here)
p File.socket?(here)

# A symbolic link is a name pointing at another name, and lstat reads the
# link itself where stat follows it.
folder = File.join(Dir.pwd, "tmp_file_facts")
Dir.mkdir(folder)
target = folder + "/target.txt"
link = folder + "/link.txt"
File.write(target, "hello")
File.symlink(target, link)

p File.symlink?(link)
p File.symlink?(target)
p File.readlink(link) == target
p File.lstat(link).ftype
p File.stat(link).ftype
p File.stat(link).size
p File.identical?(link, target)

copy = folder + "/copy.txt"
File.link(target, copy)
p File.stat(copy).nlink
p File.identical?(copy, target)

File.utime(1000000000, 1200000000, target)
p File.stat(target).mtime.to_i
p (File.stat(target) <=> File.stat(copy))

File.delete(link, copy, target)
Dir.rmdir(folder)
p File.exist?(folder)
