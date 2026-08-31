recipe = { "name" => "stew", "servings" => 4, "vegetarian" => false }
puts recipe.keys.inspect
puts recipe.values.inspect

recipe["rating"] = 5
puts recipe.keys.inspect

recipe["name"] = "soup"
puts recipe.keys.inspect

recipe.delete("servings")
puts recipe.keys.inspect

collected = []
recipe.each do |key, value|
  collected << key
end
puts collected.inspect

pairs = [1, 2, first: "a", second: "b"]
puts pairs.length
puts pairs.last.keys.inspect

arrow_pairs = ["x" => 1, "y" => 2]
puts arrow_pairs.length
puts arrow_pairs.first.keys.inspect

only_keywords = [alpha: 1, beta: 2]
puts only_keywords.first.keys.inspect
