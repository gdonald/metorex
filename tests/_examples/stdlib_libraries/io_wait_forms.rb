# IO#wait with an event mask answers the events that came ready, and with a
# timeout and modes answers the stream. Bad arguments are refused.
reading, writing = IO.pipe

p([IO::READABLE, IO::PRIORITY, IO::WRITABLE])
p(reading.wait(IO::READABLE, 0))
p(writing.wait(IO::WRITABLE, 0))
writing.write("ready")
p(reading.wait(IO::READABLE, 0))
p(reading.wait(0, :read).equal?(reading))
p(reading.wait(:r, 0, :w).equal?(reading))
p(writing.wait(0.5, :writable).equal?(writing))

def refused
  yield
rescue ArgumentError, TypeError, IOError => error
  p([error.class, error.message])
end

refused { writing.wait(0, 0) }
refused { writing.wait(-1, 0) }
refused { reading.wait(0, :sideways) }
refused { reading.wait(0, 1, :r) }
refused { reading.wait(:r, -1) }
refused { reading.wait(nil, 0) }
reading.close
refused { reading.wait(IO::READABLE, 0) }
refused { reading.wait(0, :r) }
writing.close
