path = "/tmp/metorex_example_lines.txt"
File.open(path, "w") do |file|
  file.puts "alpha"
  file.puts "beta"
end

collected = []
File.open(path) do |file|
  file.each_line do |line|
    collected << line.chomp!
  end
end
puts collected.inspect

File.open(path) do |file|
  puts file.readlines.length.inspect
end

File.open(path) do |file|
  puts file.read.length.inspect
end

words = ["alpha", "beta"]
puts words.find { |word| word.start_with?("b") }.inspect
puts words.detect { |word| word.start_with?("z") }.inspect
