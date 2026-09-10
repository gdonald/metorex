# A String changes what it holds, and every reference to it sees the change.
held = "abc"
alias_of = held
held << "de"
held.concat("f", "g")
held << 33
p(alias_of)

held.replace("hello")
held.prepend("oh, ")
held.insert(4, "!")
p(held)
p(held.slice!(0, 5))
p(held)

p(held.upcase!)
p(held.upcase!)
p(held.sub!("LLO", "y"))
p(held.reverse!)
held.clear
p(held)

# A frozen String refuses every change.
settled = "kept".freeze
p(settled.frozen?)
begin
  settled << "more"
rescue FrozenError => problem
  p problem.message
end

# `+str` asks for one that changes and `-str` for one that does not.
p (+settled).frozen?
p (-"loose").frozen?

# Case mapping takes options.
p("CÅR".downcase(:ascii))
p("İ".downcase(:turkic))
p("ß".downcase(:fold))
p("ßet".capitalize)

# A run of characters that are not letters or digits counts up as bytes.
p("(\xFF".succ.bytes)
