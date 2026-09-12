reader, writer = IO.pipe

puts(reader.fileno > 2)
puts(writer.fileno > 2)
puts(reader.class)
puts(reader.closed?)

writer.write("over the pipe\n")
writer.close

ready = IO.select([reader], nil, nil, 1)
puts(ready[0].size)
puts(reader.gets)
puts(reader.eof?)

reader.close
puts(reader.closed?)

spare, sender = IO.pipe
spare.close_read
puts(spare.closed?)
sender.close
