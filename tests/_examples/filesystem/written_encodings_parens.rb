# A write puts the bytes the string is made of on the stream, so text outside
# ASCII goes out in the encoding the string carries rather than one byte per
# character.
require "tmpdir"

Dir.mktmpdir do |held|
  path = File.join(held, "linea.txt")

  File.open(path, "w") do |stream|
    stream.write("línea\n")
  end
  p(File.binread(path).bytes)
  p(File.read(path))

  # A string holding bytes of its own is written as those bytes, whatever
  # they spell.
  raw = "\xC3\xA9".dup.force_encoding("BINARY")
  File.open(path, "wb") do |stream|
    stream.write(raw)
  end
  p(File.binread(path).bytes)

  # Reading it back through a pipe gives the same bytes, so a separator made
  # of more than one byte is found across what was written.
  reader, writer = IO.pipe
  writer.write("uno·dos·tres")
  writer.close
  p(reader.gets("·"))
  p(reader.read)
  reader.close
end
