limit = 50
primes = []

(2...limit).each do |candidate|
  is_prime = true
  primes.each do |prime|
    if candidate % prime == 0
      is_prime = false
      break
    end
  end
  if is_prime
    primes.push(candidate)
  end
end

puts primes
