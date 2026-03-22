# Case with multiple values in when clauses
# Demonstrates OR matching: when 1, 2, 3 matches if value equals 1 OR 2 OR 3

# Example 1: Basic multi-value matching
result1 = case 2
when 1, 2, 3
  "small"
when 4, 5
  "medium"
else
  "large"
end

puts result1  # Output: small

# Example 2: Match different value
result2 = case 4
when 1, 2, 3
  "small"
when 4, 5
  "medium"
else
  "large"
end

puts result2  # Output: medium

# Example 3: Fall through to else
result3 = case 10
when 1, 2, 3
  "small"
when 4, 5
  "medium"
else
  "large"
end

puts result3  # Output: large

# Example 4: Multi-value with strings
grade = case "B"
when "A", "A+"
  "Excellent"
when "B", "B+"
  "Good"
when "C", "C+"
  "Average"
else
  "Below average"
end

puts grade  # Output: Good

# Example 5: Inline syntax
status = case 200 when 200, 201, 204 then "success" when 400, 404 then "client error" else "other" end
puts status  # Output: success
