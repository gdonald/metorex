# Built-in testing framework (with parentheses)

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

  def run
    puts("#{@description}")
    @tests.each do |pair|
      desc = pair[0]
      test_block = pair[1]

      @before_hooks.each { |hook| hook.call }

      begin
        test_block.call
        @passed += 1
        puts("  PASS: #{desc}")
      rescue => e
        @failed += 1
        puts("  FAIL: #{desc} - #{e.message}")
      end

      @after_hooks.each { |hook| hook.call }
    end
    puts("#{@passed} passed, #{@failed} failed")
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
