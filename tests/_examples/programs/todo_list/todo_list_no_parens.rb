# Todo List without parentheses (where possible)
# Note: method calls on objects require parentheses for arguments

class TodoItem
  def initialize(title)
    @title = title
    @done = false
  end

  def title
    @title
  end

  def done?
    @done
  end

  def mark_done
    @done = true
  end

  def to_s
    if @done
      "[x] #{@title}"
    else
      "[ ] #{@title}"
    end
  end
end

class TodoList
  def initialize(name)
    @name = name
    @items = []
  end

  def name
    @name
  end

  def add(title)
    @items.push(TodoItem.new(title))
  end

  def complete(index)
    if index >= 0 && index < @items.length
      @items[index].mark_done
    end
  end

  def count
    @items.length
  end

  def pending_count
    count = 0
    @items.each do |item|
      if !item.done?
        count += 1
      end
    end
    count
  end

  def display
    puts "=== #{@name} ==="
    if @items.length == 0
      puts "(empty)"
      return nil
    end
    i = 0
    @items.each do |item|
      puts "#{i}. #{item.to_s}"
      i += 1
    end
    puts "#{self.pending_count} pending, #{self.count} total"
  end
end

list = TodoList.new("My Tasks")

list.add("Buy groceries")
list.add("Write tests")
list.add("Read a book")
list.add("Exercise")

puts "Before completing tasks:"
list.display

list.complete(0)
list.complete(1)

puts ""
puts "After completing tasks:"
list.display
