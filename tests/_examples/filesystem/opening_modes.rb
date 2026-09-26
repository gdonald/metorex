# A mode string is `r`, `w` or `a`, then any of `+`, `b` or `t` but not both
# of the last two, and `x` only after `w`. Anything else is refused.
path = "/tmp/metorex_opening_modes_#{Process.pid}.txt"
File.write path, "hello file\n"

def refusal
  yield
rescue StandardError => problem
  [problem.class, problem.message]
end

p refusal { File.open path, "q" }
p refusal { File.open path, "rbt" }
p refusal { File.open path, "rx" }
p refusal { File.open path, "rb", newline: :universal }

# `x` asks for a file of its own.
p refusal { File.open path, "wx" }[0]

# A numeric mode says for itself whether the file is written and cut down.
File.open(path, File::WRONLY) { |file| p refusal { file.gets } }
p File.read(path)
File.open(path, File::TRUNC) { |file| p file.gets }
p File.size(path)
p refusal { File.open "#{path}.missing", File::WRONLY }[0]

# A nil mode reads, and `b` in the mode puts the stream in binary mode.
File.open(path, nil, nil) { |file| p file.class }
File.write path, "testing\n"
File.open(path, "rb+") { |file| p [file.binmode?, file.external_encoding, file.pos] }

# With a block the file is closed through `close` after the block, and what
# that `close` raises is raised, except for a stream already closed.
closes = []
File.open path do |file|
  file.define_singleton_method(:close) do
    super()
    closes << :closed
    raise StandardError, "from close"
  end
end rescue closes << $!.message
p closes

File.delete path
