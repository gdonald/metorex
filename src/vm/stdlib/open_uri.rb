# `open-uri` reads a URI the way a file is read. Ruby stopped having it
# redefine `Kernel#open` in 3.0, so the name a program calls to open a URI is
# `URI.open`, and `Kernel#open` is left as it was.

require "uri"
require "net/http"

module URI
  # Read what a URI names. An http or https URI is fetched, and anything else
  # is handed to `Kernel#open` as a name in the file system.
  def self.open(name, *rest, &block)
    parsed = name.is_a?(URI::Generic) ? name : URI.parse(name.to_s)
    unless parsed.is_a?(URI::HTTP)
      return Kernel.open(name, *rest, &block)
    end
    body = Net::HTTP.get parsed
    held = StringIO.new body
    return held if block.nil?
    begin
      block.call held
    ensure
      held.close
    end
  end

  class Generic
    def open(*rest, &block)
      URI.open self, *rest, &block
    end
  end
end
