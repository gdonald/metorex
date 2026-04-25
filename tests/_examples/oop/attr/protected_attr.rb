c = Class.new do
  protected
  attr_accessor :foo
end

obj = c.new
begin
  obj.foo
  puts "BUG: foo getter should have raised"
rescue NoMethodError => e
  puts "OK reader raised: #{e.message}"
end

begin
  obj.foo = 1
  puts "BUG: foo setter should have raised"
rescue NoMethodError => e
  puts "OK writer raised: #{e.message}"
end
