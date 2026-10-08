# Inside a block, `local_variables` lists the block's own locals and then
# those of the scopes it was written in, out to the method or file around it.
def listing
  outer = 1
  [outer].map do |item|
    inside = item
    [1].map { innermost = inside; local_variables }.first
  end.first
end

def with_parameters(first, second = 2, *rest, key:, &block)
  local_variables
end

p(listing)
p(with_parameters(1, key: 3))
p(proc { |argument; shadowed| local_variables }.call(1))
