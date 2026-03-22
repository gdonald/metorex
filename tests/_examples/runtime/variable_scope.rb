# Variable Scope and Resolution Examples for Metorex
# This file demonstrates how variable scoping works in Metorex

# ==========================================
# 1. Global Scope Variables
# ==========================================

# Variables declared at the top level are in global scope
global_var = 42
puts("Global variable: ", global_var)

# ==========================================
# 2. Function Scope
# ==========================================

def outer_function
  # This variable is local to outer_function
  outer_local = "I'm local to outer_function"

  # Can access global variables from within functions
  puts("Accessing global from function: ", global_var)

  def inner_function
    # This is local to inner_function
    inner_local = "I'm local to inner_function"

    # Can access variables from outer scopes
    puts("Inner accessing outer: ", outer_local)
    puts("Inner accessing global: ", global_var)
  end

  inner_function()
  puts("Outer local: ", outer_local)
end

outer_function()

# ==========================================
# 3. Variable Shadowing
# ==========================================

x = 10  # Outer x

def test_shadowing
  # This x shadows the outer x
  x = 20
  puts("Inside function x: ", x)  # Prints 20

  def nested
    # This x shadows both outer x values
    x = 30
    puts("Inside nested x: ", x)  # Prints 30
  end

  nested()
  puts("Back in function x: ", x)  # Prints 20
end

test_shadowing()
puts("Outer x: ", x)  # Prints 10

# ==========================================
# 4. Block Scope
# ==========================================

y = 100

if true
  # Variables declared in if blocks are in their own scope
  block_var = "I'm in a block"
  puts("Block variable: ", block_var)

  # Can access outer scope
  puts("Accessing y from block: ", y)
end

# ==========================================
# 5. Loop Variable Scope
# ==========================================

# For loop variables are scoped to the loop
for i in [1, 2, 3]
  puts("Loop variable i: ", i)

  # Can declare variables inside the loop
  loop_local = i * 2
  puts("Loop local: ", loop_local)
end

# i is not accessible here (would be undefined)
# puts(i)  # Error: Undefined variable 'i'

# ==========================================
# 6. Parameter Scope
# ==========================================

def with_params(a, b, c = 10)
  # Parameters are in the function's scope
  puts("Parameter a: ", a)
  puts("Parameter b: ", b)
  puts("Parameter c (with default): ", c)

  # Can modify parameter values
  a = a + 1
  puts("Modified a: ", a)
end

with_params(1, 2)
with_params(5, 10, 20)

# ==========================================
# 7. Lambda/Block Closures
# ==========================================

def make_counter
  count = 0

  # This lambda captures 'count' from the outer scope
  counter = lambda do
    count = count + 1
    return count
  end

  return counter
end

counter1 = make_counter()
puts("Counter1 call 1: ", counter1.call())  # 1
puts("Counter1 call 2: ", counter1.call())  # 2

counter2 = make_counter()
puts("Counter2 call 1: ", counter2.call())  # 1 (separate closure)

# ==========================================
# 8. Class Scope
# ==========================================

class ScopeExample
  # Class body has its own scope

  def initialize(value)
    # Instance variables are prefixed with @
    @instance_var = value
  end

  def show_value
    # Can access instance variables
    puts("Instance variable: ", @instance_var)

    # Local variables in methods
    local_var = "I'm local to show_value"
    puts("Local variable: ", local_var)
  end

  def modify_value(new_value)
    @instance_var = new_value
  end
end

obj = ScopeExample.new(100)
obj.show_value()
obj.modify_value(200)
obj.show_value()

# ==========================================
# 9. Undefined Variable Error
# ==========================================

# The resolver detects undefined variables at compile time
# This would cause an error during resolution:
# puts(undefined_variable)

# ==========================================
# 10. Variable Reuse
# ==========================================

def variable_reuse
  # Same variable name can be reassigned
  result = 1
  puts("Result 1: ", result)

  result = 2
  puts("Result 2: ", result)

  result = result + 10
  puts("Result 3: ", result)

  return result
end

final = variable_reuse()
puts("Final result: ", final)

# ==========================================
# 11. Nested Scopes
# ==========================================

def level1
  l1_var = "Level 1"

  def level2
    l2_var = "Level 2"

    def level3
      l3_var = "Level 3"

      # Can access all outer scopes
      puts("L3 sees L1: ", l1_var)
      puts("L3 sees L2: ", l2_var)
      puts("L3 sees L3: ", l3_var)
    end

    level3()
  end

  level2()
end

level1()

# ==========================================
# 12. While Loop Scope
# ==========================================

counter = 0
while counter < 3
  # Loop scope
  loop_msg = "Iteration " + counter.to_s()
  puts(loop_msg)
  counter = counter + 1
end

puts("Final counter: ", counter)

puts("\n✓ Variable scope examples completed!")
