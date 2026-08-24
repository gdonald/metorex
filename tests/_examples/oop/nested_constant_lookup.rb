module Catalog
  LIMIT = 3

  module Storage
    class Shelf
      def same_kind?(other)
        other.kind_of? Shelf
      end

      def limit
        LIMIT
      end

      def own_name
        Shelf
      end
    end
  end
end

shelf = Catalog::Storage::Shelf.new

puts shelf.same_kind?(Catalog::Storage::Shelf.new)
puts shelf.same_kind?(Object.new)
puts shelf.limit
puts shelf.own_name
