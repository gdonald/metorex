# A stream told an encoding writes text carried into it, one piece at a
# time, while `syswrite` writes the bytes as they stand. A stream told none
# writes bytes, and nothing written asks nothing of the stream.
path = "/tmp/metorex_converted_writes_#{Process.pid()}.txt"

File.open(path, "w", encoding: Encoding::UTF_32LE) { |file| p(file.write("h", "i")) }
p(File.binread(path).bytes())
File.open(path, "w", external_encoding: Encoding::UTF_16BE) { |file| file.syswrite("hi") }
p(File.binread(path).bytes())
File.open(path, "w") { |file| file.write("\x87".b(), "ą") }
p(File.binread(path).bytes())
File.open(path, "r") { |file| p(file.write("")) }

def refusal()
  yield()
rescue StandardError => problem
  [problem.class(), problem.message()]
end

File.open(path, "w", external_encoding: Encoding::UTF_16BE) do |file|
  p(refusal() { file.write("été".b()) })
end

# `IO.write` passes its options to the open, or `open_args:` in their place,
# and answers how many bytes it wrote.
p(IO.write(path, "hi", mode: "w", encoding: Encoding::UTF_32LE))
p(IO.write(path, "hi", open_args: ["w", nil, {encoding: Encoding::UTF_32LE}]))
p(IO.write(path, "hi", 2, mode: "r", open_args: ["w"]))
p(File.read(path))
p(refusal() { IO.write(path, "hi", open_args: [{encoding: Encoding::UTF_32LE}]) })
p(refusal() { IO.write(path, "hi", mode: "w:UTF-16BE", encoding: Encoding::UTF_32LE) })

File.delete(path)
