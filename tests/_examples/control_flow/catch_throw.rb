puts catch(:done) { throw :done, "thrown value" }
puts catch(:done) { "block value" }

catch :outer do
  catch :inner do
    throw :outer
  end
  puts "not reached"
end
puts "unwound to the outer catch"

def find_first_negative(rows)
  catch :found do
    rows.each do |row|
      row.each do |value|
        throw :found, value if value < 0
      end
    end
    nil
  end
end

puts find_first_negative([[1, 2], [3, -4], [5]])
puts find_first_negative([[1, 2], [3, 4]]).inspect

fresh = catch { |tag| tag }
puts fresh.class

label = "key"
puts catch(label) { throw label, "matched by identity" }

begin
  catch("key") { throw "key" }
rescue ArgumentError => error
  puts error.class
end

begin
  catch(:a) { throw :b }
rescue UncaughtThrowError => error
  puts error.message
end

begin
  catch :nothing
rescue LocalJumpError => error
  puts error.class
end
