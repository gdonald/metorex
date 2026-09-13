answered = eval(<<~RUBY)
  class WithEnsure
    :body
  ensure
    :ensure
  end
RUBY
puts answered.to_s

module WithRescue
  raise "stopped"
rescue RuntimeError => error
  puts error.message
ensure
  puts "module ensured"
end

held = Object.new
class << held
  raise "singleton stopped"
rescue RuntimeError => error
  puts error.message
end

puts (class << true; self; end).to_s
puts (class << nil; self; end).to_s
begin
  class << 1
    :never
  end
rescue TypeError => error
  puts error.message
end
