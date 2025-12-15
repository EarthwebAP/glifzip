# Changelog

All notable changes to GLifzip will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-12-15

### Added - Week 1 (Core Implementation)
- Multi-threaded Zstd compression with configurable levels (1-22)
- Multi-threaded LZ4 decompression for ultra-fast extraction
- GLIF file format with 116-byte header and JSON sidecar metadata
- SHA256 verification for both payload and archive integrity
- Deterministic compression for reproducible builds
- CLI tool with `create`, `extract`, and `verify` commands
- Comprehensive test suite with 34 passing tests
- Benchmark framework using Criterion
- Cross-platform support (Linux, macOS, Windows)
- Chunk-based processing (128 MB chunks) for optimal parallelization
- Rayon-based thread pool with work-stealing parallelism
- Adler-32 checksum for header validation
- Big-endian byte ordering for cross-platform compatibility

### Week 2 (Planned)
- Directory compression support with recursive file handling
- Advanced CLI features (progress bars, verbose output)
- File listing in archives
- Exclude patterns for selective compression
- Metadata preservation (permissions, timestamps)
- Enhanced error messages and user feedback

### Week 3 (Planned)
- Windows file association registration
- Windows Explorer context menu integration
- Windows installer (MSI/EXE)
- macOS Finder integration
- Icon design and branding
- Code signing for release binaries
- Desktop environment integration (Linux)

### Week 4 (Planned)
- Performance profiling and optimization
- Hot path optimization for compression/decompression
- Benchmark suite with real-world data
- Performance regression tests
- Production deployment guide
- Comprehensive API documentation
- Production-ready error handling

## [0.2.0] - Planned (Week 2 Target)

### Planned
- Directory and multi-file compression
- Progress indicators
- Enhanced CLI experience

## [0.1.0] - 2025-12-14

### Added
- Initial prototype implementation
- Basic compression/decompression functionality
- File format specification
- Core library structure

## Project Milestones

### Week 1: Foundation (COMPLETED)
**Focus**: Core compression engine and file format
- Rust project structure
- GLIF format specification
- Multi-threaded compression/decompression
- Verification system
- Test suite

### Week 2: Features (UPCOMING)
**Focus**: User experience and directory support
- Directory compression
- CLI enhancements
- Progress feedback
- File metadata

### Week 3: Platform Integration (UPCOMING)
**Focus**: OS integration and distribution
- Windows integration
- macOS integration
- Linux desktop integration
- Installers and packages

### Week 4: Production Readiness (UPCOMING)
**Focus**: Performance and quality
- Performance optimization
- Benchmarking
- Documentation
- Release preparation

## Performance Targets

| Cores | Compression (Target) | Decompression (Target) |
|-------|---------------------|------------------------|
| 1     | ≥1.0 GB/s           | ≥2.0 GB/s              |
| 2     | ≥2.0 GB/s           | ≥4.0 GB/s              |
| 4     | ≥4.0 GB/s           | ≥8.0 GB/s              |
| 8     | ≥8.0 GB/s           | ≥16.0 GB/s             |

## Known Issues

### v1.0.0
- Performance testing limited to WSL environment (native testing planned)
- No directory compression yet (Week 2)
- No progress indicators yet (Week 2)
- Windows integration not yet implemented (Week 3)

## Technical Details

### Compression Algorithm
- Primary: Zstd (levels 1-22)
- Secondary: LZ4 wrapper for fast decompression
- Chunk size: 128 MB per thread
- Thread pool: Rayon work-stealing

### File Format
- Magic number: "GLIF01"
- Header size: 116 bytes fixed + variable sidecar
- Checksum: Adler-32 (header), SHA256 (payload/archive)
- Byte order: Big-endian for portability

### Dependencies
- zstd 0.13 - Multi-threaded compression
- lz4 1.24 - Ultra-fast decompression
- rayon 1.7 - Parallelism framework
- sha2 0.10 - Cryptographic hashing
- serde_json 1.0 - Metadata serialization
- clap 4.0 - CLI argument parsing
- criterion 0.5 - Benchmarking

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development workflow and guidelines.

## License

GLifzip is licensed under the MIT License. See [LICENSE](LICENSE) for details.
