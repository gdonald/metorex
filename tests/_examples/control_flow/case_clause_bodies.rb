def classify(code)
  case code
  when 200, 201
    if code == 200
      "ok"
    else
      "created"
    end
  when :"!", :+, :[]
    "operator"
  when :return
    "keyword"
  else
    "other"
  end
end
p(classify(200))
p(classify(201))
p(classify(:"!"))
p(classify(:[]))
p(classify(:return))
p(classify(404))

def label(status)
  text = case status
         when :up
           prefix = "service"
           "#{prefix} is up"
         when :down
           prefix = "service"
           suffix = "down"
           "#{prefix} is #{suffix}"
         else
           note = "unknown"
           note.upcase
         end
  text
end
p(label(:up))
p(label(:down))
p(label(:paused))
