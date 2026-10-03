# A class that undefines initialize_copy cannot finish a copy: dup and
# clone call it on the copy, and it is not there to answer.
class Uncopyable
  undef_method :initialize_copy
end

[:dup, :clone].each do |copying|
  begin
    Uncopyable.new.public_send copying
  rescue NoMethodError => error
    puts error.message
  end
end
