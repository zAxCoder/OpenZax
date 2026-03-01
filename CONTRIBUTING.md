# Contributing to OpenZax

Thank you for your interest in contributing to OpenZax! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/openzax.git`
3. Create a branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Run formatting: `cargo fmt`
7. Run linting: `cargo clippy`
8. Commit your changes: `git commit -m "Add your feature"`
9. Push to your fork: `git push origin feature/your-feature-name`
10. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.82+ (2024 edition)
- Git

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Style

We use `rustfmt` and `clippy` to maintain code quality:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

## Project Structure

```
openzax/
├── crates/
│   ├── core/          # Core engine
│   ├── shell/         # Terminal interface
│   ├── sdk/           # SDK for extensions
│   └── cli/           # CLI tool
├── docs/              # Documentation
└── .github/           # CI/CD workflows
```

## Pull Request Guidelines

- Keep PRs focused on a single feature or fix
- Write clear commit messages
- Add tests for new functionality
- Update documentation as needed
- Ensure all CI checks pass

## Reporting Issues

When reporting issues, please include:

- OpenZax version
- Operating system
- Rust version
- Steps to reproduce
- Expected vs actual behavior
- Error messages or logs

## Feature Requests

Feature requests are welcome! Please:

- Check if the feature is already planned in the roadmap
- Describe the use case clearly
- Explain why it would benefit the project

## Questions?

Feel free to open an issue for questions or join our community discussions.

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).
