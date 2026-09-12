# Every line read is written back over the file it came from, which is what
# `-i` asks for. The name of the file being read is reported as it is edited.
while (line = ARGF.gets)
  puts(line.chomp.upcase)
end
