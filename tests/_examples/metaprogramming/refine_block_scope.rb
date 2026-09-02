inside = nil

refinement = Module.new do
  refine String do
    def shout
      upcase
    end

    inside = "hello".shout
  end

  refine Integer do
    def described
      "int #{to_s}"
    end
  end

  refine Array do
    def described
      map { |item| item.described }.join(", ")
    end
  end
end

puts inside

result = nil
Module.new do
  using refinement
  result = [1, 2].described
end
puts result

begin
  Module.new { refine String }
rescue ArgumentError => error
  puts error.message
end

begin
  Module.new { refine "nope" do; end }
rescue TypeError => error
  puts error.message
end

module Countable
end

Module.new do
  refine Countable do
    def counted
      "counted"
    end
  end
end
puts "refined a module"
