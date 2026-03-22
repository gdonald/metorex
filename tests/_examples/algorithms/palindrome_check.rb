text = "racecar"
chars = []

text.each_char do |char|
  chars.push(char)
end

reversed_text = ""

chars.each do |char|
  reversed_text = char + reversed_text
end

is_palindrome = text == reversed_text
puts is_palindrome
