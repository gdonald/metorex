# `ruby2_keywords` keeps the keyword hash a splat gathered marked, so the
# splat can pass it on as keywords again.
class Relay
  ruby2_keywords def gather(*args)
    args
  end

  def keywords(**given)
    given
  end

  def positional(one)
    one
  end
end

relay = Relay.new
gathered = relay.gather(1, name: "ada")
p(gathered)
marked = gathered.last
p(Hash.ruby2_keywords_hash?(marked))

original = { name: "ada" }
p(Hash.ruby2_keywords_hash?(original))
p(relay.keywords(*relay.gather(**original)))
p(Hash.ruby2_keywords_hash?(relay.positional(*relay.gather(**original))))

plain = -> *args { args }
p(plain.call(2, name: "ada").last.class)
marking = -> *args { args }
marking.ruby2_keywords
p(Hash.ruby2_keywords_hash?(marking.call(2, name: "ada").last))
