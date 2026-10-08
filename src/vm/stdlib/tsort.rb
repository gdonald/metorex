# Topological sorting and strongly connected components of a graph that an
# object describes through `tsort_each_node` and `tsort_each_child`, or that
# two callables describe for the module functions.
module TSort
  VERSION = "0.2.0"

  # Raised when the graph has a cycle, so no order puts every node after
  # the nodes it depends on.
  class Cyclic < StandardError
  end

  def tsort
    TSort.tsort(method(:tsort_each_node), method(:tsort_each_child))
  end

  def tsort_each(&block)
    TSort.tsort_each(method(:tsort_each_node), method(:tsort_each_child), &block)
  end

  def strongly_connected_components
    TSort.strongly_connected_components(method(:tsort_each_node), method(:tsort_each_child))
  end

  def each_strongly_connected_component(&block)
    TSort.each_strongly_connected_component(
      method(:tsort_each_node), method(:tsort_each_child), &block
    )
  end

  def each_strongly_connected_component_from(node, id_map = {}, stack = [], &block)
    TSort.each_strongly_connected_component_from(
      node, method(:tsort_each_child), id_map, stack, &block
    )
  end

  def tsort_each_node
    raise NotImplementedError
  end

  def tsort_each_child(node)
    raise NotImplementedError
  end

  def self.tsort(each_node, each_child)
    tsort_each(each_node, each_child).to_a
  end

  # Each node after every node it reaches. A cycle raises Cyclic naming the
  # nodes in it.
  def self.tsort_each(each_node, each_child)
    return to_enum(__method__, each_node, each_child) unless block_given?

    each_strongly_connected_component(each_node, each_child) do |component|
      raise Cyclic, "topological sort failed: #{component.inspect}" unless component.size == 1
      yield component.first
    end
  end

  def self.strongly_connected_components(each_node, each_child)
    each_strongly_connected_component(each_node, each_child).to_a
  end

  # Each strongly connected component, every one after the components it
  # reaches, found by Tarjan's algorithm from each node in turn.
  def self.each_strongly_connected_component(each_node, each_child)
    return to_enum(__method__, each_node, each_child) unless block_given?

    id_map = {}
    stack = []
    each_node.call do |node|
      next if id_map.include?(node)
      each_strongly_connected_component_from(node, each_child, id_map, stack) do |component|
        yield component
      end
    end
    nil
  end

  # The components reachable from `node`. `id_map` numbers the nodes seen
  # so far and `stack` holds the ones whose component is still open. The
  # answer is the lowest number the walk from `node` reached.
  def self.each_strongly_connected_component_from(node, each_child, id_map = {}, stack = [], &block)
    unless block
      return to_enum(__method__, node, each_child, id_map, stack)
    end

    minimum_id = node_id = id_map[node] = id_map.size
    stack_length = stack.length
    stack << node
    each_child.call(node) do |child|
      if id_map.include?(child)
        child_id = id_map[child]
        minimum_id = child_id if child_id && child_id < minimum_id
      else
        reached = each_strongly_connected_component_from(child, each_child, id_map, stack, &block)
        minimum_id = reached if reached < minimum_id
      end
    end
    if node_id == minimum_id
      component = stack.slice!(stack_length..-1)
      component.each { |member| id_map[member] = nil }
      block.call(component)
    end
    minimum_id
  end
end
