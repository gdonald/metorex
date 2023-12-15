# Contributing to Metorex

Thank you for your interest in contributing to Metorex! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Rust 2024 edition or later
- Cargo (comes with Rust)
- Git

### Getting Started

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/yourusername/metorex.git
   cd metorex
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

## Project Structure

Metorex is organized as a single Rust crate with the following structure:

- **`src/`** - Source code
  - `lib.rs` - Library entry point
  - `main.rs` - CLI binary entry point
  - `error.rs` - Error handling and reporting
  - `runtime.rs` - Runtime execution environment
- **`tests/`** - Integration tests
  - `error_test.rs` - Error handling tests
  - `test_runner.rs` - Example file test harness
  - `version_test.rs` - Version tests
- **`examples/`** - Example `.mx` files demonstrating language features

## Development Guidelines

### Code Quality

1. **Run tests before committing:**
   ```bash
   cargo test
   ```

2. **Run clippy to catch common mistakes:**
   ```bash
   cargo clippy
   ```

3. **Format code with rustfmt:**
   ```bash
   cargo fmt
   ```

### Testing Requirements

- **Always write new tests for new functionality**
- **Tests must be in the `tests/` directory, not in implementation files**
- **Run `scripts/misplaced_tests.sh` to verify test placement**
- **Code coverage should be 100%** - measure with:
  ```bash
  cargo tarpaulin --out Stdout
  ```
- **All tests must pass before submitting a pull request**

### Code Style

- Follow Rust naming conventions
- Write clear, descriptive variable and function names
- Add comments for complex logic
- Keep functions focused and small
- Use meaningful commit messages

### Test Placement

All tests must be in the `tests/` directory. Do not add:
- `#[cfg(test)]` modules in implementation files
- `#[test]` functions in `src/` files
- Doc tests in implementation files

Use the provided script to check for misplaced tests:
```bash
./scripts/misplaced_tests.sh
```

## Pull Request Process

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes and commit:**
   ```bash
   git add .
   git commit -m "Add feature: description of your changes"
   ```

3. **Ensure all checks pass:**
   - All tests pass (`cargo test`)
   - No clippy warnings (`cargo clippy`)
   - Code is formatted (`cargo fmt`)
   - No misplaced tests (`./scripts/misplaced_tests.sh`)
   - Code coverage is maintained or improved

4. **Push your branch:**
   ```bash
   git push origin feature/your-feature-name
   ```

5. **Open a pull request** with a clear description of your changes

## Reporting Issues

When reporting issues, please include:

- Rust version (`rustc --version`)
- Operating system and version
- Steps to reproduce the issue
- Expected behavior
- Actual behavior
- Any error messages or logs

## Questions?

If you have questions about contributing, feel free to:
- Open an issue for discussion
- Reach out to the maintainers

## License

By contributing to Metorex, you agree that your contributions will be licensed under the MIT License.
