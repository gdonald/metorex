class Recipe
  def initialize
    @name = "stew"
    @servings = 4
    @vegetarian = false
  end
end

recipe = Recipe.new
puts recipe.instance_variables.inspect

recipe.instance_variable_set(:@rating, 5)
puts recipe.instance_variables.inspect

puts Object.new.instance_variables.inspect
puts nil.instance_variables.inspect
puts 42.instance_variables.inspect
