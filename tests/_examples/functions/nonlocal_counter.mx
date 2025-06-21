def make_counter
  count = 0

  increment = lambda do
    count = count + 1
    puts count
  end

  get_count = lambda do
    puts count
  end

  reset = lambda do
    count = 0
  end

  increment.call
  increment.call
  increment.call
  get_count.call
  reset.call
  get_count.call
  increment.call
end

make_counter
