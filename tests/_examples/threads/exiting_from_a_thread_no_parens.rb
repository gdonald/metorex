# The same run written without parentheses where they can be left off.
#
# A thread that ends the program hands the word to the main thread rather than
# dying of it, so nothing is written about a thread that failed. A thread that
# dies of anything else is reported.

recorded = []
ready = false

Thread.new do
  Thread.pass until ready
  begin
    exit 42
  rescue SystemExit => ending
    recorded << :in_the_thread
    raise ending
  end
end

begin
  ready = true
  sleep
rescue SystemExit => ending
  recorded << :in_the_main_thread
  recorded << ending.status
end

p recorded

quiet = Thread.new do
  Thread.current.report_on_exception = false
  raise "kept to itself"
end

begin
  quiet.join
rescue RuntimeError => refused
  p refused.message
end
