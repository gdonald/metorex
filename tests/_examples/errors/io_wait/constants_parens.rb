puts(IO::EAGAINWaitReadable.superclass)
puts(IO::EAGAINWaitReadable.ancestors.include?(IO::WaitReadable))
puts(IO::EAGAINWaitReadable.equal?(IO::EWOULDBLOCKWaitReadable))

puts(IO::EAGAINWaitWritable.superclass)
puts(IO::EAGAINWaitWritable.ancestors.include?(IO::WaitWritable))
puts(IO::EAGAINWaitWritable.equal?(IO::EWOULDBLOCKWaitWritable))

puts(IO::EAGAINWaitReadable.ancestors.include?(IO::WaitWritable))
puts(Errno::EAGAIN.equal?(Errno::EWOULDBLOCK))

error = IO::EAGAINWaitReadable.new("would block")
puts(error.class)
puts(error.is_a?(Errno::EAGAIN))
puts(error.is_a?(IO::WaitReadable))
