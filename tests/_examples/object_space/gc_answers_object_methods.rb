# A method the program defines on Object is one GC and ObjectSpace answer
# too, as every object does, while a name neither module keeps an account of
# still reads as nil.
class Object
  def described_by_program
    "answered for #{inspect}"
  end
end

p(GC.described_by_program)
p(ObjectSpace.described_by_program)
p(GC.respond_to?(:described_by_program))
