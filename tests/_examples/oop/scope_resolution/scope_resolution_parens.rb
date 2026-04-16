# Scope resolution with :: (with parentheses)
class Config
  VERSION = 1
  MAX_SIZE = 100
end

puts(Config::VERSION)
puts(Config::MAX_SIZE)
