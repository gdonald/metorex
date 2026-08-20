module Basic
end

module Super
  include Basic
end

class Parent
end

class Child < Parent
  include Super
end

puts (Child > Parent).inspect
puts (Parent > Child).inspect
puts (Child > Child).inspect
puts (Parent > Basic).inspect

puts (Child < Parent).inspect
puts (Parent < Child).inspect
puts (Child <= Child).inspect
puts (Child >= Child).inspect
puts (Basic >= Super).inspect
puts (Super <= Basic).inspect
puts (Basic < Parent).inspect

begin
  Parent > Object.new
rescue TypeError => error
  puts error.message
end
