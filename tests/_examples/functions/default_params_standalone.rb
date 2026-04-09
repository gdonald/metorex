def say(msg, times = 1)
  i = 0
  while i < times
    puts(msg)
    i = i + 1
  end
end

say("once")
say("twice", 2)
