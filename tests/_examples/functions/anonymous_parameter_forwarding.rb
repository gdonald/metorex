def target(*args, **options, &given)
  [args, options, given.nil?].inspect
end

def forward_rest(*)
  target(*)
end
puts forward_rest(1, 2)

def forward_keywords(**)
  target(**)
end
puts forward_keywords(a: 1)

def forward_block(&)
  target(&)
end
puts forward_block { :ignored }

def forward_everything(...)
  target(...)
end
puts forward_everything(1, a: 2)

["*", "**", "&"].each do |anonymous|
  begin
    eval("def outer(#{anonymous}); proc { |#{anonymous}| target(#{anonymous}) }; end")
    puts "no error"
  rescue SyntaxError => refused
    puts refused.message.include?("also used within block")
  end
end
