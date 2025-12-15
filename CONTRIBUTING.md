# Contributing to GLifzip

Thank you for your interest in contributing to GLifzip! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Code Style](#code-style)
- [Performance Considerations](#performance-considerations)

## Code of Conduct

GLifzip is part of the GlyphOS ecosystem. We are committed to providing a welcoming and inclusive environment for all contributors. Please be respectful, constructive, and collaborative.

## Getting Started

### Prerequisites

- Rust 1.70 or later (2021 edition)
- Cargo
- Git
- (Optional) QEMU for testing GlyphOS integration

### Setting Up Development Environment

1. Clone the repository:
```bash
git clone https://github.com/EarthwebAP/glifzip.git
cd glifzip
```

2. Build the project:
```bash
cargo build --release
```

3. Run tests to verify setup:
```bash
cargo test --release
```

4. Run benchmarks (optional):
```bash
cargo bench
```

## Development Workflow

### Branch Strategy

- `main` - Stable release branch
- `develop` - Active development branch
- `feature/*` - New features
- `bugfix/*` - Bug fixes
- `release/*` - Release preparation

### Creating a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### Making Changes

1. Write code following the project's style guidelines
2. Add tests for new functionality
3. Update documentation as needed
4. Ensure all tests pass
5. Run `cargo fmt` to format code
6. Run `cargo clippy` to check for common issues

## Testing

### Running Tests

```bash
# Run all tests
cargo test --release

# Run specific test
cargo test --release test_name

# Run with output
cargo test --release -- --nocapture

# Run integration tests only
cargo test --release --test integration_tests
```

### Writing Tests

- Place unit tests in the same file as the code being tested
- Place integration tests in the `tests/` directory
- Use descriptive test names: `test_<functionality>_<scenario>`
- Test edge cases, error conditions, and happy paths

Example:
```rust
#[test]
fn test_compress_empty_data() {
    let data = vec![];
    let config = CompressionConfig::default();
    let result = compress(&data, &config);
    assert!(result.is_ok());
}
```

### Benchmarking

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench compression

# Generate HTML report
cargo bench -- --save-baseline my-baseline
```

## Commit Guidelines

### Commit Message Format

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Build process or auxiliary tool changes
- `ci`: CI/CD changes

### Examples

```
feat(compression): add support for compression level 22

Implement support for maximum Zstd compression level.
This provides better compression ratios for archival use cases.

Closes #123
```

```
fix(decompression): handle corrupted chunk headers

Add validation for chunk metadata to prevent crashes
when processing corrupted archives.

Fixes #456
```

## Pull Request Process

### Before Submitting

1. Ensure all tests pass: `cargo test --release`
2. Format code: `cargo fmt`
3. Check for issues: `cargo clippy`
4. Update documentation if needed
5. Update CHANGELOG.md with your changes
6. Rebase on latest `develop` branch

### Creating a Pull Request

1. Push your branch to GitHub:
```bash
git push origin feature/your-feature-name
```

2. Create a Pull Request on GitHub with:
   - Clear title describing the change
   - Detailed description of what changed and why
   - Reference to related issues
   - Screenshots/benchmarks if applicable

3. Request review from maintainers

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Performance improvement
- [ ] Documentation update
- [ ] Other (please describe)

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows project style guidelines
- [ ] All tests pass
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No breaking changes (or documented if necessary)
```

## Code Style

### Rust Style Guidelines

- Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `cargo fmt` with default settings
- Maximum line length: 100 characters
- Use meaningful variable names
- Add comments for complex logic

### Code Organization

```rust
// Imports grouped and sorted
use std::fs;
use std::io::{Read, Write};

// Module-level constants
const MAGIC_NUMBER: &[u8] = b"GLIF01";
const CHUNK_SIZE: usize = 128 * 1024 * 1024;

// Public API first
pub fn compress(data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>> {
    // Implementation
}

// Private helpers after
fn compress_chunk(chunk: &[u8], level: i32) -> Result<Vec<u8>> {
    // Implementation
}

// Tests at the end
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Test implementation
    }
}
```

### Documentation

- Add doc comments for all public APIs
- Use examples in doc comments
- Document error conditions
- Keep comments up-to-date with code changes

Example:
```rust
/// Compresses data using multi-threaded Zstd compression.
///
/// # Arguments
///
/// * `data` - The data to compress
/// * `config` - Compression configuration
///
/// # Returns
///
/// Returns compressed data in GLIF format or an error.
///
/// # Examples
///
/// ```
/// use glifzip::{compress, CompressionConfig};
///
/// let data = b"Hello, GLifzip!";
/// let config = CompressionConfig::default();
/// let compressed = compress(data, &config)?;
/// ```
///
/// # Errors
///
/// Returns an error if compression fails or if the data is too large.
pub fn compress(data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>> {
    // Implementation
}
```

## Performance Considerations

### Optimization Guidelines

1. **Profile before optimizing**: Use `cargo bench` and profiling tools
2. **Avoid premature optimization**: Focus on correctness first
3. **Use appropriate data structures**: Vec, HashMap, etc.
4. **Minimize allocations**: Reuse buffers when possible
5. **Leverage parallelism**: Use Rayon for CPU-bound tasks

### Performance Targets

When contributing performance improvements, aim to maintain or exceed these targets:

| Cores | Compression    | Decompression  |
|-------|----------------|----------------|
| 1     | ≥1.0 GB/s      | ≥2.0 GB/s      |
| 2     | ≥2.0 GB/s      | ≥4.0 GB/s      |
| 4     | ≥4.0 GB/s      | ≥8.0 GB/s      |
| 8     | ≥8.0 GB/s      | ≥16.0 GB/s     |

### Benchmarking Changes

```bash
# Create baseline before changes
cargo bench -- --save-baseline before

# Make your changes

# Compare against baseline
cargo bench -- --baseline before
```

## Project Structure

```
glifzip/
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Core library
│   ├── format/                 # GLIF format handling
│   │   ├── mod.rs
│   │   ├── header.rs           # Header parsing/writing
│   │   └── sidecar.rs          # JSON metadata
│   ├── compression/            # Compression engines
│   │   ├── mod.rs
│   │   ├── zstd_compressor.rs  # Zstd compression
│   │   └── lz4_decompressor.rs # LZ4 decompression
│   └── verification/           # Hash verification
│       ├── mod.rs
│       └── sha256.rs           # SHA256 functions
├── tests/                      # Integration tests
├── benches/                    # Benchmarks
├── Cargo.toml                  # Project manifest
├── README.md                   # Project overview
├── CHANGELOG.md               # Version history
└── CONTRIBUTING.md            # This file
```

## Areas for Contribution

### High Priority

- **Week 2 Features**: Directory compression, CLI enhancements
- **Performance**: Optimization of compression/decompression hot paths
- **Testing**: Additional test coverage, edge cases
- **Documentation**: API docs, usage examples, tutorials

### Medium Priority

- **Week 3 Features**: Windows/macOS integration
- **Error Handling**: Improved error messages
- **Platform Support**: Testing on different OSes
- **Benchmarks**: Real-world data benchmarks

### Low Priority

- **Code Quality**: Refactoring, cleanup
- **Tooling**: Development scripts, CI improvements
- **Examples**: Sample applications using GLifzip

## Getting Help

- **Issues**: Open an issue on GitHub for bugs or questions
- **Discussions**: Use GitHub Discussions for general questions
- **Email**: Contact the GlyphOS team at 969dwi@gmail.com

## Recognition

Contributors will be recognized in:
- CHANGELOG.md
- GitHub contributors page
- Release notes

Thank you for contributing to GLifzip!

---

**Last Updated**: December 15, 2025
**Project**: GLifzip v1.0.0
**Part of**: GlyphOS Ecosystem
