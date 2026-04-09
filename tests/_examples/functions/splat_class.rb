class Logger
  def initialize(prefix)
    @prefix = prefix
  end

  def log(*messages)
    messages.each do |m|
      puts(@prefix + ": " + m)
    end
  end
end

l = Logger.new("APP")
l.log("start", "running", "stop")

# Splat forwarding
def forward(*args)
  puts(args.length)
  args.each do |a|
    puts(a)
  end
end

forward(1, 2, 3)
