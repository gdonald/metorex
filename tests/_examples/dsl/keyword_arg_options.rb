def receive_options(name, options = nil)
  if options.nil?
    puts "options is nil"
  else
    puts "options.class = #{options.class}"
    puts "options[:shared] = #{options[:shared]}"
    puts "options[\"shared\"] = #{options["shared"]}"
  end
end

receive_options("a", shared: true)
receive_options("b", { shared: true })
receive_options("c")
