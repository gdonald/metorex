puts ("hello" !~ /ell/).inspect
puts ("hello" !~ /xyz/).inspect
puts (/ell/ !~ "hello").inspect

class Matcher
  def =~ other
    other == :expected
  end
end

matcher = Matcher.new
puts (matcher !~ :expected).inspect
puts (matcher !~ :other).inspect

class Overridden
  def !~ other
    :custom
  end
end

puts (Overridden.new !~ :anything).inspect

begin
  Object.new !~ :foo
rescue NoMethodError => error
  puts "#{error.class}: #{error.message}"
end

begin
  42 !~ 99
rescue NoMethodError => error
  puts error.message
end
