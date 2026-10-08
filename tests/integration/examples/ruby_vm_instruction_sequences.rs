// Examples of RubyVM::InstructionSequence, which compiles through a parser
// written in Ruby. They run in a test binary of their own so no one binary
// takes long enough under coverage to reach tarpaulin's time limit.

use super::run_example;

/// The expected output of both `ruby_vm/instruction_sequences` variants.
const INSTRUCTION_SEQUENCES_OUTPUT: &str = concat!(
    "<RubyVM::InstructionSequence:<compiled>@<compiled>:1>\n",
    "[\"<compiled>\", \"<compiled>\", \"<compiled>\", \"<compiled>\", 1]\n",
    "21\n",
    "[[1, :line], [2, :line]]\n",
    "== disasm: #<ISeq:<compiled>@<compiled>:1 (1,0)-(2,9)>\n",
    "[\"<main>\", true, true, 1]\n",
    "[<RubyVM::InstructionSequence:<compiled>@report.rb:12>, \"report.rb\", \"/reports/report.rb\", 12, 42]\n",
    "[\"area\", \"area\", 19, true]\n",
    "\"Not a toplevel InstructionSequence\"\n",
    "\"block in <main>\"\n",
    "\"block (2 levels) in builder\"\n",
    "\"block in <module:Reports>\"\n",
    "nil\n",
    "[[\"a\", 1], [\"<class:B>\", 2], [\"block in <compiled>\", 3]]\n",
    "21\n",
    "\"<compiled>:1: syntax error, unexpected end-of-input\"\n",
    "\"<compiled>:1: syntax error, unexpected end-of-input\\ntotal = price *\\n               ^\\n\"\n",
    "\"loop.rb:1: Invalid break\\nbreak if ready\\n^~~~~\\n\"\n",
    "\"allocator undefined for RubyVM::InstructionSequence\"\n",
    "0\n",
);

#[test]
fn test_ruby_vm_instruction_sequences_execution() {
    let output = run_example("ruby_vm/instruction_sequences.rb");
    assert_eq!(output, INSTRUCTION_SEQUENCES_OUTPUT);
}

#[test]
fn test_ruby_vm_instruction_sequences_no_parens_execution() {
    let output = run_example("ruby_vm/instruction_sequences_no_parens.rb");
    assert_eq!(output, INSTRUCTION_SEQUENCES_OUTPUT);
}
