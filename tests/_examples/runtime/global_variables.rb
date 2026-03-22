# Global variables with $ prefix
$count = 0

def increment
  $count = $count + 1
end

increment
increment
increment
puts $count

$message = "hello"
puts $message
