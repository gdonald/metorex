keywords = ->(named:, optional: 2, **rest) { [named, optional, rest] }
puts(keywords.call(named: 1, extra: 3).inspect)
begin
  keywords.call
rescue ArgumentError => error
  puts(error.message)
end

none = ->(**nil) { :ok }
puts(none.call.to_s)
begin
  none.call(a: 1)
rescue ArgumentError => error
  puts(error.message)
end

spread = ->(first, second = 1, *middle, last) { [first, second, middle, last] }
puts(spread.call(1, 2).inspect)
puts(spread.call(1, 2, 3, 4).inspect)

grouped = ->((first, second, *middle, last), (*leading, next_to_last, final)) do
  [first, second, middle, last, leading, next_to_last, final]
end
puts(grouped.call(1, 2).inspect)
puts(grouped.call([1, 2, 3], [4, 5, 6, 7]).inspect)

taking_a_block = ->(&given) { given.nil? ? :none : given.call }
puts(taking_a_block.().to_s)
puts(taking_a_block.() { :given }.to_s)
