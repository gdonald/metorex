# `alias` takes a setter's name with its `=`, written bare or as a symbol.
class Schedule
  attr_reader :window

  def set_window value
    @window = value
  end

  alias window= set_window
  alias :slot= :set_window
end

schedule = Schedule.new
schedule.window = "02:00"
p schedule.window
schedule.slot = "03:00"
p schedule.window
p Schedule.instance_method(:window=).original_name
