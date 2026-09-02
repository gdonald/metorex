begin
  exit
rescue SystemExit => error
  puts error.message
  puts error.status
  puts error.success?
end

begin
  exit 42
rescue SystemExit => error
  puts error.status
  puts error.success?
end

begin
  exit(-1)
rescue SystemExit => error
  puts error.status
  puts error.success?
end

begin
  exit false
rescue SystemExit => error
  puts error.status
  puts error.success?
end

begin
  exit(true)
rescue SystemExit => error
  puts error.status
  puts error.success?
end

def leaves
  exit 7
ensure
  puts "ensure ran"
end

begin
  leaves
rescue SystemExit => error
  puts error.status
end

begin
  begin
    exit 3
  rescue StandardError
    puts "should not rescue here"
  end
rescue SystemExit => error
  puts "SystemExit is not a StandardError"
  puts error.class
end
