# A logger given a named shift age rotates on the calendar, moving the file
# aside under the period it covered.
require "logger"
require "tmpdir"

dir = File.join(Dir.tmpdir, "metorex_log_rotation_parens")
Dir.entries(dir).each { |name| File.delete(File.join(dir, name)) unless name.start_with?(".") } if Dir.exist?(dir)
Dir.mkdir(dir) unless Dir.exist?(dir)

path = File.join(dir, "service.log")
logger = Logger.new(path, "daily", shift_period_suffix: "%Y-%m-%d")
today = Time.now

logger.add(Logger::INFO, "first")

tomorrow = Time.at(today.to_i + 60 * 60 * 24)
Time.define_singleton_method(:now) { tomorrow }

logger.add(Logger::INFO, "second")
logger.close

shifted = "#{path}.#{today.strftime('%Y-%m-%d')}"
puts(File.exist?(shifted))
puts(File.read(shifted).include?("first"))
puts(File.read(path).include?("second"))

Dir.entries(dir).each { |name| File.delete(File.join(dir, name)) unless name.start_with?(".") }
Dir.rmdir(dir)
