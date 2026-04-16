module StringExt
  refine(String) do
    def shout
      upcase + "!"
    end
  end
end

using StringExt

puts "hello".shout
