# A method that calls itself without end raises SystemStackError, which the
# program can rescue.
def deeper(level)
  deeper(level + 1)
end

begin
  deeper(0)
rescue SystemStackError => error
  p error.class
  p error.message
end
p :carried_on
