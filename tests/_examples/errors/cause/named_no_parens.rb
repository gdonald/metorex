# The same run written without parentheses where they can be left off.
#
# `raise` takes a `cause:` naming what the new error follows. It settles the
# matter: an error raised with one is not chained onto whatever was being
# handled, and an error raised a second time keeps the cause it was first
# raised with.

def show
  yield
rescue Exception => raised
  puts "#{raised.class}: #{raised.message} <- #{raised.cause.inspect}"
end

# A cause named outright, in place of the one being handled.
show do
  begin
    raise "being handled"
  rescue
    raise "the new one", cause: StandardError.new("named instead")
  end
end

# `cause: nil` asks for none at all.
show do
  begin
    raise "being handled"
  rescue
    raise "the new one", cause: nil
  end
end

# Naming cause alone says nothing about what to raise.
show { raise cause: StandardError.new("only this") }

# A cause has to be an exception.
show { raise "the new one", cause: Object.new }

# An exception is not its own cause.
held = StandardError.new("itself")
show { raise held, cause: held }

# A chain that would run back on itself is refused.
show do
  begin
    raise "one"
  rescue => first
    begin
      raise "two"
    rescue
      begin
        raise "three"
      rescue => third
        raise first, cause: third
      end
    end
  end
end

# An error raised again keeps the cause it was raised with the first time.
show do
  begin
    raise "one"
  rescue => first
    begin
      raise "two"
    rescue
      raise first
    end
  end
end
