# Contributing to Waremax

Thank you for your interest in contributing to Waremax! This document provides guidelines and information for contributors.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates. When filing a bug report, include:

- A clear, descriptive title
- Steps to reproduce the issue
- Expected behavior vs actual behavior
- Your environment (OS, Rust version, Waremax version)
- Relevant configuration files (with sensitive data removed)
- Any error messages or logs

### Suggesting Features

Feature requests are welcome. Please provide:

- A clear description of the feature
- The problem it solves or use case it enables
- Any implementation ideas you have

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Follow the code style** (see below)
3. **Add tests** for new functionality
4. **Update documentation** as needed
5. **Ensure all tests pass** before submitting
6. **Write a clear PR description** explaining your changes

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/waremax.git
cd waremax

# Build the project
cargo build

# Run tests
cargo test

# Run clippy for linting
cargo clippy --all-targets --all-features

# Format code
cargo fmt
```

## Code Style

### Rust Guidelines

- Follow standard Rust naming conventions
- Use `cargo fmt` before committing
- Address all `cargo clippy` warnings
- Write documentation comments for public APIs
- Keep functions focused and reasonably sized

### Commit Messages

- Use clear, descriptive commit messages
- Start with a verb in present tense (e.g., "Add", "Fix", "Update")
- Reference issue numbers when applicable

Example:
```
Add congestion-aware routing policy

Implements a new routing strategy that considers current traffic
conditions when planning robot paths.

Fixes #123
```

### Testing

- Write unit tests for new functionality
- Include integration tests for complex features
- Use descriptive test names that explain what is being tested
- Aim for meaningful test coverage, not just high percentages

## Project Structure

```
waremax/
├── src/                    # CLI binary entry point
├── crates/
│   ├── waremax-core        # Core simulation engine
│   ├── waremax-config      # Configuration parsing
│   ├── waremax-map         # Map and pathfinding
│   ├── waremax-entities    # Domain entities
│   ├── waremax-policies    # Allocation policies
│   ├── waremax-metrics     # Metrics collection
│   ├── waremax-sim         # Simulation orchestration
│   ├── waremax-testing     # Test utilities
│   └── waremax-analysis    # Statistical analysis
├── docs/                   # Additional documentation
├── examples/               # Example configurations
└── documentation/          # MkDocs documentation site
```

## Review Process

1. All PRs require at least one maintainer review
2. CI checks must pass (tests, formatting, clippy)
3. Changes may require documentation updates
4. Large changes should be discussed in an issue first

## Getting Help

- Open an issue for bugs or feature discussions
- Check existing documentation and issues first
- Be patient and respectful in all interactions

## License

By contributing to Waremax, you agree that your contributions will be licensed under the MIT License.
