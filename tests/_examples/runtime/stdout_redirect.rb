path = "/tmp/metorex_stdout_redirect_#{Process.pid}.txt"

File.open(path, "w") do |file|
  saved = $stdout
  $stdout = file
  p "captured"
  puts "and this"
  $stdout = saved
end

File.open(path) do |file|
  puts file.read(9).inspect
  puts file.read
end

File.delete path

class Described
  def inspect
    "<described>"
  end
end

p [Described.new, "text", :symbol, nil]
p Described.new
