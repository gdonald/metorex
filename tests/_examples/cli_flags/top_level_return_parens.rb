# A `return` written at the top level ends the file, and the value written
# after it goes nowhere, which Ruby says so on standard error.
puts("before")
return 10
puts("after")
