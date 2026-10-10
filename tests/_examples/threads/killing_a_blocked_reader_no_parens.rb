# A thread waiting to read a pipe or a socket is asleep, and killing it ends
# the wait: its value is nil.
require "socket"

reader, writer = IO.pipe
piped = Thread.new { reader.readpartial 10 }
Thread.pass until piped.stop?
p piped.status
piped.kill
p piped.value
p piped.status

near, far = UNIXSocket.pair
socketed = Thread.new { near.read 10 }
Thread.pass until socketed.stop?
socketed.kill
p socketed.value
far.close
writer.close
