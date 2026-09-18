# A read that has nothing to hand over yet waits, and waiting is where every
# other thread gets its turn.
reading, writing = IO.pipe
held = Thread.new { reading.read(5) }
writing.write "hello"
writing.close
p(held.value)
reading.close

# `IO.select` waits the same way, so another thread can make a stream ready
# while the caller is in it.
reading, writing = IO.pipe
main = Thread.current
closer = Thread.new do
  Thread.pass until main.stop?
  writing.close
end
ready, = IO.select([reading])
p(ready == [reading])
closer.join
reading.close

# A pipe carries the encodings it was opened with, which belong to the end
# the text comes out of.
reading, writing = IO.pipe(Encoding::UTF_16BE, Encoding::UTF_8)
p(reading.external_encoding)
p(reading.internal_encoding)
reading.close
writing.close

# A stream another thread closes while this one waits on it is closed.
reading, writing = IO.pipe
watcher = Thread.new do
  begin
    reading.read(1)
  rescue IOError => error
    error.class
  end
end
Thread.pass until watcher.stop?
reading.close
p(watcher.value)
writing.close
