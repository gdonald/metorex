at_exit { puts("handler should not run") }

begin
  puts("before")
  exit!(21)
ensure
  puts("ensure should not run")
end
