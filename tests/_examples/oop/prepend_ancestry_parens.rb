module Logging
  CHANNEL = "logging"

  def describe
    "logged: #{super}"
  end
end

class Report
  prepend Logging

  CHANNEL = "report"

  def describe
    "report"
  end

  def self.channel
    CHANNEL
  end
end

report = Report.new
puts(report.describe)
puts(Report.ancestors.first)
puts(Report.ancestors.take(2).last)
puts(report.is_a?(Logging))
puts(Report.channel)
puts(Report.constants.include?(:CHANNEL))

class Detailed < Report
  def describe
    "detailed: #{super}"
  end
end

puts(Detailed.new.describe)

module Shared
end

class Base
  include Shared
end

class Derived < Base
  prepend Shared
end

puts(Derived.ancestors.select { |ancestor| ancestor == Shared }.length)

begin
  Class.new { prepend }
rescue ArgumentError
  puts("ArgumentError")
end

class Alone
  prepend Logging
end

begin
  Alone.new.describe
rescue NoMethodError => error
  puts(error.message)
end
