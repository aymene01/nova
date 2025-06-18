# Contributing to Nova

## Quick Start

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Run tests: `make all`
5. Commit with conventional format: `git commit -m "feat: your feature"`
6. Push and create a pull request

## Development

```bash
# Run simulation
cargo run -- start

# Run tests
make all

# Format code
cargo fmt

# Check linting
cargo clippy
```

## Commit Format

Use conventional commits:

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation
- `test:` - Tests
- `chore:` - Maintenance

## Testing

All changes must pass the existing test suite (87 tests). Add tests for new functionality.
