# Configuration language DSL
# Demonstrates using a builder to define structured application configuration

class Config
  def initialize
    @entries = []
  end

  def set(key, value)
    @entries.push([key, value])
    self
  end

  def get(key)
    found = nil
    @entries.each do |pair|
      if pair[0] == key
        found = pair[1]
      end
    end
    found
  end
end

class AppConfig
  def initialize
    @config = Config.new
  end

  def server(host, port)
    @config.set("host", host)
    @config.set("port", "#{port}")
    self
  end

  def database(host, port)
    @config.set("db_host", host)
    @config.set("db_port", "#{port}")
    self
  end

  def debug(enabled)
    @config.set("debug", "#{enabled}")
    self
  end

  def get(key)
    @config.get(key)
  end
end

config = AppConfig.new
config.server("localhost", 8080)
config.database("db.local", 5432)
config.debug(true)

puts config.get("host")
puts config.get("port")
puts config.get("db_host")
puts config.get("db_port")
puts config.get("debug")
