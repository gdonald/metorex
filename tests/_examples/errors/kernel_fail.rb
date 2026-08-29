begin
  fail
rescue RuntimeError => error
  puts error.class
end

begin
  fail "the duck is not irish."
rescue => error
  puts error.message
end

class MissingWidget < RuntimeError
end

begin
  fail MissingWidget
rescue MissingWidget => error
  puts error.class
end

begin
  fail MissingWidget, "no widget here"
rescue MissingWidget => error
  puts error.message
end

class Builder
  def exception(message)
    StandardError.new message
  end
end

begin
  fail Builder.new, "built by hand"
rescue StandardError => error
  puts "#{error.class}: #{error.message}"
end

class Sender
  def go
    send :fail, "sent along"
  end
end

begin
  Sender.new.go
rescue RuntimeError => error
  puts error.message
end

puts Kernel.private_instance_methods.include?(:fail)
