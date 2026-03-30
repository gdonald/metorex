# Data Processing Pipeline: Process a list of student records
# Demonstrates: arrays, hashes, iteration, filtering, mapping

# Student data as hashes
students = [
  {"name" => "Alice", "grade" => 92, "subject" => "Math"},
  {"name" => "Bob", "grade" => 78, "subject" => "Science"},
  {"name" => "Charlie", "grade" => 95, "subject" => "Math"},
  {"name" => "Diana", "grade" => 88, "subject" => "Science"},
  {"name" => "Eve", "grade" => 67, "subject" => "Math"},
  {"name" => "Frank", "grade" => 91, "subject" => "Science"}
]

# Step 1: Filter passing students (grade >= 70)
passing = students.filter do |s|
  s["grade"] >= 70
end

puts "Passing students:"
passing.each do |s|
  puts "  #{s["name"]}: #{s["grade"]}"
end

# Step 2: Get names of honor roll students (grade >= 90)
honor_roll = students.filter do |s|
  s["grade"] >= 90
end

puts "Honor roll:"
honor_roll.each do |s|
  puts "  #{s["name"]}: #{s["grade"]}"
end

# Step 3: Calculate average grade
total = 0
students.each do |s|
  total += s["grade"]
end
average = total / students.length
puts "Average grade: #{average}"

# Step 4: Count students per subject
math_count = 0
science_count = 0
students.each do |s|
  if s["subject"] == "Math"
    math_count += 1
  else
    science_count += 1
  end
end
puts "Math students: #{math_count}"
puts "Science students: #{science_count}"

# Step 5: Extract just the names
names = students.map do |s|
  s["name"]
end
puts "All students: #{names}"
