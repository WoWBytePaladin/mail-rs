# Contributing to mail-rs

Thank you for considering contributing to mail-rs! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful and considerate of others. We're all here to learn and improve.

## How to Contribute

### Reporting Bugs

If you find a bug:

1. Check if the bug has already been reported in [Issues](https://github.com/mail-rs/mail-rs/issues)
2. If not, create a new issue with:
   - Clear title and description
   - Steps to reproduce
   - Expected vs actual behavior
   - Rust version and OS
   - Minimal code example if possible

### Suggesting Features

For feature requests:

1. Check if the feature has been suggested before
2. Create a new issue describing:
   - What problem it solves
   - How it should work
   - Example usage
   - Why it fits the project goals

### Pull Requests

1. **Fork** the repository
2. **Create a branch** for your feature:
   ```bash
   git checkout -b feature/my-new-feature
   ```
3. **Make your changes**:
   - Follow the code style
   - Add tests for new functionality
   - Update documentation
   - Ensure all tests pass
4. **Commit** with clear messages:
   ```bash
   git commit -m "feat: add support for X"
   ```
   Use conventional commits format:
   - `feat:` - New feature
   - `fix:` - Bug fix
   - `docs:` - Documentation only
   - `style:` - Code style changes
   - `refactor:` - Code refactoring
   - `test:` - Adding tests
   - `chore:` - Maintenance tasks
5. **Push** to your fork:
   ```bash
   git push origin feature/my-new-feature
   ```
6. **Open a Pull Request** on GitHub

## Development Setup

### Prerequisites

- Rust 1.90.0 or later
- Git

### Building

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/mail-rs.git
cd mail-rs

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --workspace -- -D warnings
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p mail-core
cargo test -p mail-smtp
cargo test -p mail-builder

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_message_simple
```

### Running Examples

```bash
cargo run --example simple
cargo run --example html_with_attachments
cargo run --example bulk_send
```

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use meaningful variable names
- Add doc comments for public APIs
- Keep functions focused and small
- Prefer explicit types when it improves clarity

### Documentation

- Document all public APIs with `///` doc comments
- Include examples in doc comments
- Update README.md when adding features
- Keep CHANGELOG.md updated

Example:
```rust
/// Creates a new email address with display name.
///
/// # Examples
///
/// ```
/// use mail_core::Address;
///
/// let addr = Address::with_name("user@example.com", "John Doe");
/// assert_eq!(addr.format(), "John Doe <user@example.com>");
/// ```
pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
    // ...
}
```

## Project Structure

```
mail-rs/
├── mail-core/          # Core types and RFC implementations
│   ├── src/
│   │   ├── lib.rs
│   │   ├── message.rs
│   │   ├── header.rs
│   │   ├── address.rs
│   │   ├── encoding.rs
│   │   └── error.rs
│   └── Cargo.toml
├── mail-smtp/          # SMTP client
│   ├── src/
│   │   ├── lib.rs
│   │   ├── client.rs
│   │   ├── transport.rs
│   │   ├── auth.rs
│   │   └── error.rs
│   └── Cargo.toml
├── mail-builder/       # High-level API
│   ├── src/
│   │   └── lib.rs
│   ├── examples/
│   └── Cargo.toml
├── Cargo.toml          # Workspace config
└── README.md
```

## Testing Guidelines

- Write tests for new functionality
- Test edge cases and error conditions
- Use descriptive test names
- Keep tests focused and independent

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_with_name() {
        let addr = Address::with_name("user@example.com", "User Name");
        assert_eq!(addr.email, "user@example.com");
        assert_eq!(addr.name, Some("User Name".to_string()));
    }

    #[test]
    fn test_address_format_special_chars() {
        let addr = Address::with_name("user@example.com", "User, Name");
        assert_eq!(addr.format(), "\"User, Name\" <user@example.com>");
    }
}
```

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Examples:
```
feat(smtp): add OAuth2 authentication support
fix(core): correct boundary generation in multipart messages
docs(readme): update installation instructions
test(smtp): add integration tests for TLS connection
```

## Release Process

1. Update version in `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Create a git tag
4. Push to GitHub
5. Publish to crates.io (if applicable)

## Questions?

Feel free to open an issue for any questions or discussions!

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
