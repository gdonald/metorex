# ARGF reads a list of files as though they were one.
one = "/tmp/metorex_argf_plain_one.txt"
two = "/tmp/metorex_argf_plain_two.txt"
File.write one, "alpha\nbeta\n"
File.write two, "gamma\ndelta\n"

stream = ARGF.class.new one, two
p stream.to_s
p stream.file.path == one
p stream.argv
p stream.gets
p stream.lineno
p stream.eof?
p stream.gets
p stream.eof?
p stream.gets
p stream.file.path == two
p stream.readlines
# The last file has been read to the end, so the stream is closed and refuses
# to say whether it is at one.
begin
  stream.eof?
rescue IOError => problem
  p problem.message
end

reader = ARGF.class.new one
p reader.read(2)
p reader.pos
reader.rewind
p reader.pos
p reader.getc
p reader.read

File.delete one
File.delete two
