def register(event, handler)
  puts event.to_s
  puts handler
end

register :start, "handler_one"
register :finish, "handler_two"
