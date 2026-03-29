# Built-in testing framework (with parentheses)
# Supports before/after hooks, test filtering, and colored output

class Expectation
  def initialize(value)
    @value = value
  end

  def to_equal(expected)
    assert_equal(expected, @value)
  end

  def to_be_truthy
    assert(@value, "Expected truthy value")
  end

  def to_be_falsy
    assert(!@value, "Expected falsy value")
  end

  def to_be_nil
    assert(@value == nil, "Expected nil")
  end

  def to_be_a(klass)
    assert(@value.is_a?(klass), "Type mismatch")
  end
end

class TestSuite
  def initialize(description)
    @description = description
    @tests = []
    @passed = 0
    @failed = 0
    @before_hooks = []
    @after_hooks = []
    @filter = nil
  end

  def before(&block)
    @before_hooks.push(block)
  end

  def after(&block)
    @after_hooks.push(block)
  end

  def it(description, &block)
    @tests.push([description, block])
  end

  def only(pattern)
    @filter = pattern
  end

  def run
    puts("\e[1m#{@description}\e[0m")
    @tests.each do |pair|
      desc = pair[0]
      test_block = pair[1]

      skip = false
      if @filter != nil
        if !desc.include?(@filter)
          skip = true
        end
      end

      if !skip
        @before_hooks.each { |hook| hook.call }

        begin
          test_block.call
          @passed += 1
          puts("  \e[32mPASS\e[0m: #{desc}")
        rescue => e
          @failed += 1
          puts("  \e[31mFAIL\e[0m: #{desc} - #{e.message}")
        end

        @after_hooks.each { |hook| hook.call }
      end
    end
    if @failed == 0
      puts("\e[32m#{@passed} passed\e[0m, #{@failed} failed")
    else
      puts("#{@passed} passed, \e[31m#{@failed} failed\e[0m")
    end
  end
end

def describe(name, &block)
  suite = TestSuite.new(name)
  block.call(suite)
  suite.run
end

def expect(value)
  Expectation.new(value)
end

# === Example usage ===

describe("Math operations") do |t|
  t.it("adds numbers") { expect(1 + 1).to_equal(2) }
  t.it("multiplies numbers") { expect(3 * 4).to_equal(12) }
  t.it("divides numbers") { expect(10 / 2).to_equal(5) }
end

describe("String operations") do |t|
  t.it("concatenates strings") { expect("hello" + " world").to_equal("hello world") }
  t.it("gets length") { expect("test".length).to_equal(4) }
end

describe("Type checking") do |t|
  t.it("checks integer type") { expect(42).to_be_a(Integer) }
  t.it("checks truthiness") { expect(true).to_be_truthy }
  t.it("checks nil") { expect(nil).to_be_nil }
end

describe("Assertions") do |t|
  t.it("assert_equal catches mismatches") { assert_raises { assert_equal(1, 2) } }
  t.it("assert catches false") { assert_raises { assert(false, "should fail") } }
end

describe("Filtered suite") do |t|
  t.only("add")
  t.it("add test") { expect(1 + 1).to_equal(2) }
  t.it("multiply test") { expect(3 * 4).to_equal(12) }
end
