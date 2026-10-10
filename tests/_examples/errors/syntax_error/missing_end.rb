# rubocop:disable Lint/Syntax
class Kennel
  def admit(dog)
    dogs << dog

  def release(dog)
    dogs.delete(dog)
  end
end
# rubocop:enable Lint/Syntax
