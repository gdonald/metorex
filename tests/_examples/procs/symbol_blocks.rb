labels = [1, 2].map(&:to_s)
p(labels)
p([3, 4].map(&:succ))
p(:+.to_proc.call(1, 2))

def handle = yield

begin
  handle(&:upcase)
rescue ArgumentError => error
  p(error.message)
end

begin
  :upcase.to_proc.call
rescue ArgumentError => error
  p(error.message)
end

begin
  [1].map(&:missing_name)
rescue NoMethodError => error
  p(error.backtrace.last.sub(/\A.*:(\d+):/, 'line \1:'))
end

shout = :upcase.to_proc
begin
  shout.call(1)
rescue NoMethodError => error
  p(error.backtrace.map { |line| line.sub(/\A.*:(\d+):/, 'line \1:') })
end

flag = true
p(flag &:anything)
mask = 6
bits = 3
p(mask &bits)
