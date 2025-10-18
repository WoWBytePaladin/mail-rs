---
name: Bug Report
about: Create a report to help us improve
title: '[BUG] '
labels: bug
assignees: ''

---

## Bug Description

A clear and concise description of what the bug is.

## To Reproduce

Steps to reproduce the behavior:

1. Create a message with '...'
2. Call method '....'
3. See error

## Expected Behavior

A clear and concise description of what you expected to happen.

## Actual Behavior

What actually happened instead.

## Code Sample

```rust
// Minimal code example that reproduces the issue
use mail_builder::MessageBuilder;

let message = MessageBuilder::new()
    .from("test@example.com")
    .to("recipient@example.com")
    .build();
```

## Environment

- **OS**: [e.g. Ubuntu 22.04, Windows 11, macOS 13]
- **Rust version**: [e.g. 1.75.0] (run `rustc --version`)
- **mail-rs version**: [e.g. 0.1.0]
- **mail-core version**: [e.g. 0.1.0]
- **mail-smtp version**: [e.g. 0.1.0]
- **mail-builder version**: [e.g. 0.1.0]

## Additional Context

Add any other context about the problem here, such as:

- Error messages or stack traces
- Relevant configuration
- Related issues
- Workarounds attempted

## Logs

```text
Paste any relevant logs here
```
