def hands_over
  yield(1)
end
puts hands_over { |taken| taken * 2 }

["class Holder; yield; end", "module Held; yield; end", "1.times { yield }"].each do |source|
  begin
    eval source
    puts "no error"
  rescue SyntaxError => error
    puts error.message.include?("Invalid yield").to_s
  end
end
