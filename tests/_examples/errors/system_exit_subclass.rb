class CustomExit < SystemExit
end

puts "before raise"
raise CustomExit.new(8)
puts "never reached"
