# Simple test framework DSL
# Demonstrates using blocks to build a minimal testing framework

class TestSuite
  def initialize(name)
    @name = name
    @tests = []
    @passed = 0
    @failed = 0
  end

  def test(description, &block)
    @tests.push([description, block])
  end

  def run
    puts "Suite: #{@name}"
    @tests.each do |pair|
      desc = pair[0]
      blk = pair[1]
      result = blk.call
      if result
        @passed = @passed + 1
        puts "  PASS: #{desc}"
      else
        @failed = @failed + 1
        puts "  FAIL: #{desc}"
      end
    end
    puts "Results: #{@passed} passed, #{@failed} failed"
  end
end

suite = TestSuite.new("Math Tests")
suite.test("1 + 1 equals 2") { 1 + 1 == 2 }
suite.test("10 / 2 equals 5") { 10 / 2 == 5 }
suite.test("3 * 4 equals 12") { 3 * 4 == 12 }
suite.test("5 - 3 equals 2") { 5 - 3 == 2 }
suite.test("impossible math") { 1 + 1 == 3 }
suite.run
