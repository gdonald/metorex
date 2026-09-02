at_exit { puts("last handler") }
at_exit { exit(3) }
at_exit { puts("first handler") }
puts("main body")
exit(2)
