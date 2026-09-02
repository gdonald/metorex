path = "/tmp/metorex_kernel_open_#{Process.pid}.txt"

File.open(path, "w") do |file|
  file.write "first line\nsecond line\n"
end

handle = open path
puts handle.class
puts handle.gets
puts handle.gets
puts handle.gets.inspect
handle.close

puts open(path, "r") { |file| file.gets }

class Pathish
  def to_path
    ENV["METOREX_OPEN_PATH"]
  end
end

ENV["METOREX_OPEN_PATH"] = path
puts open(Pathish.new, "r") { |file| file.gets }

class Openable
  def to_open(*arguments)
    arguments
  end
end

p open(Openable.new, 1, 2, 3)
p(open(Openable.new) { |value| value })

begin
  open
rescue ArgumentError => error
  puts error.message
end

begin
  open path, "r", 0, 0
rescue ArgumentError => error
  puts error.message
end

puts File::CREAT
puts Kernel.private_instance_methods.include? :open
File.delete path
