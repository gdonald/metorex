# The date library's reading and writing of times, added to Time itself.
require 'date'

class Time
  def self.parse(text, now = Time.now)
    DateTime.parse(text).to_time
  end

  def self.iso8601(text)
    DateTime.iso8601(text).to_time
  end

  def self.xmlschema(text)
    DateTime.iso8601(text).to_time
  end

  def self.rfc2822(text)
    DateTime.rfc2822(text).to_time
  end

  def self.rfc822(text)
    DateTime.rfc2822(text).to_time
  end

  def self.httpdate(text)
    DateTime.httpdate(text).to_time
  end

  def iso8601(places = 0)
    written = self.strftime("%Y-%m-%dT%H:%M:%S")
    if places > 0
      written = written + "." + (self.subsec * 10 ** places).floor.to_s.rjust(places, "0")
    end
    written + (self.utc? ? "Z" : self.to_datetime.zone)
  end

  def xmlschema(places = 0)
    self.iso8601(places)
  end

  def rfc2822
    self.strftime("%a, %-d %b %Y %H:%M:%S %z")
  end

  def rfc822
    self.rfc2822
  end

  def httpdate
    self.getutc.strftime("%a, %d %b %Y %H:%M:%S GMT")
  end
end
