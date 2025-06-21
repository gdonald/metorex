
def risky_operation()
  puts 'risky operation!'
  raise 'Oops...'
end

def cleanup()
  puts 'cleanup'
end

begin
  risky_operation()
rescue StandardError => e
  puts "General error: #{e.message}"
ensure
  cleanup()
end
