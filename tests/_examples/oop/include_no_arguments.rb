begin
  Class.new do
    include
  end
rescue ArgumentError => error
  puts error.message
end
