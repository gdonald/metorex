PORT_NUMBER = 8080
module Settings
end

[
  -> { class PORT_NUMBER; end },
  -> { class Settings; end },
  -> { class Settings::Missing < Class.new.singleton_class; end },
].each do |attempt|
  begin
    attempt.call
  rescue TypeError => error
    puts error.message
  end
end

class Record
end
class AuditRecord < Record
end
[Object, BasicObject].each do |other|
  begin
    eval "class AuditRecord < #{other}; end"
  rescue TypeError => error
    puts "#{other}: #{error.message}"
  end
end
puts AuditRecord.superclass

class Counter
  def self.remember(count)
    @count = count
  end
end
Counter.remember(3)
p Counter.instance_variables
Counter.remove_instance_variable(:@count)
p Counter.instance_variables

built = class Report; :body_value; end
p built
p(class Report; class << self; :singleton_value; end; end)
named = module Formats; :module_value; end
p named

holder = Class.new do
  class ScopedLog
  end
  LOG_LEVEL = :info

  def self.level
    LOG_LEVEL
  end
end
p ScopedLog.name
p holder.level
p holder.const_defined?(:LOG_LEVEL, false)

class Report
  def self.first_line
    class << self
      return :from_singleton_body
    end
    :after
  end
end
p Report.first_line
