def pair(a, b)
  "#{a}|#{b}"
end

puts pair(<<-EOS, "second")
first line
second line
EOS

x = <<-TXT + " world"
hello
TXT
puts x
