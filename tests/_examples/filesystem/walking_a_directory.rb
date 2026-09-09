# An open directory is walked one name at a time, and the position it is left
# at decides what the next read finds.
folder = File.join(Dir.pwd, "tmp_walking")
Dir.mkdir(folder)
File.write(File.join(folder, "one.txt"), "1")
File.write(File.join(folder, "two.txt"), "2")
Dir.mkdir(File.join(folder, "inside"))

opened = Dir.open(folder)
p opened.class
p opened.path == folder
p opened.to_path == folder
p opened.inspect == "#<Dir:" + folder + ">"
p opened.closed?
p Dir.include?(Enumerable)

p opened.pos
first = opened.read
p first.class
p opened.pos
opened.rewind
p opened.pos
p opened.read == first

# A position taken before a read leads back to the same name.
mark = opened.pos
before = opened.read
opened.seek(mark)
p opened.read == before

p opened.entries.sort
p opened.children.sort

walked = []
opened.each { |name| walked.push(name) }
p walked.sort
p opened.read

collected = []
opened.each_child { |name| collected.push(name) }
p collected.sort

p opened.close
p opened.closed?
p opened.path == folder

begin
  opened.read
rescue IOError => problem
  p problem.message
end

# Reading the whole directory at once needs no handle.
p Dir.entries(folder).sort
p Dir.children(folder).sort
p Dir.exist?(folder)

Dir.open(folder) { |held| p held.children.length }

File.delete(File.join(folder, "one.txt"), File.join(folder, "two.txt"))
Dir.rmdir(File.join(folder, "inside"))
Dir.rmdir(folder)
p Dir.exist?(folder)
