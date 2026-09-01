# A backtrace entry reads `file:line:in 'label'`. `full_message` renders the
# backtrace with the detailed message, and appends the cause chain.

def inner
  raise "boom"
end

def outer
  inner
end

begin
  outer
rescue => error
  trace = error.backtrace
  puts trace[0].end_with?(":in 'Object#inner'").to_s
  puts trace[1].end_with?(":in 'Object#outer'").to_s
  puts trace[2].end_with?(":in '<main>'").to_s
  puts trace[-1].end_with?(":in '<main>'").to_s
end

reported = RuntimeError.new("Some runtime error")
reported.set_backtrace(["a.rb:1", "b.rb:2"])
puts reported.full_message(highlight: false, order: :top).lines.first
puts reported.full_message(highlight: false, order: :bottom).lines.first

# The cause chain follows the exception's own report.
begin
  begin
    raise "the cause"
  rescue
    raise "main exception"
  end
rescue => error
  whole = error.full_message(highlight: false)
  puts whole.include?("main exception").to_s
  puts whole.include?("the cause").to_s
end

# A block can take arguments without naming them.
anonymous = lambda { |**| "keywords ignored" }
puts anonymous.call
splatted = lambda { |*| "positionals ignored" }
puts splatted.call(1, 2)

# An index past either end answers nil, and a negative one counts back.
values = ["first", "second"]
puts values[-1]
puts values[-2]
puts values[-3].inspect
puts values[9].inspect
