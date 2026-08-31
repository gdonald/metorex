result = catch(:done) do
  10.times do |i|
    throw(:done, i) if i == 3
  end
  :never
end
puts result.inspect

puts catch(:plain) { throw :plain }.inspect

begin
  throw(:one, :two, :three)
rescue ArgumentError => error
  puts "#{error.class}: #{error.message}"
end

begin
  throw
rescue ArgumentError => error
  puts error.class
end

begin
  throw(:no_catch_for_this)
rescue UncaughtThrowError => error
  puts error.class
end

class Widget
  def name
    "widget"
  end
end

widget = Widget.new
tapped = widget.tap { |w| w.name }
puts tapped.equal? widget
puts widget.tap { :ignored }.equal?(widget)

begin
  3.tap
rescue LocalJumpError => error
  puts "#{error.class}: #{error.message}"
end

puts Object.new.respond_to? :taint
puts Object.new.respond_to? :tainted?
