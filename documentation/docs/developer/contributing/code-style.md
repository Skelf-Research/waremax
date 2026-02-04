# Code Style Guide

Coding conventions for Waremax.

---

## Rust Style

### Formatting

Use `rustfmt` with default settings:

```bash
cargo fmt
```

All code must pass formatting checks:

```bash
cargo fmt --check
```

### Linting

Use `clippy` for linting:

```bash
cargo clippy -- -D warnings
```

Fix all clippy warnings before submitting.

---

## Naming Conventions

### Types

```rust
// Structs: PascalCase
pub struct RobotConfig { ... }

// Enums: PascalCase
pub enum TaskStatus { ... }

// Traits: PascalCase, often adjectives
pub trait Schedulable { ... }

// Type aliases: PascalCase
pub type NodeId = u32;
```

### Functions and Methods

```rust
// Functions: snake_case
fn calculate_distance(from: NodeId, to: NodeId) -> f64 { ... }

// Methods: snake_case
impl Robot {
    fn is_idle(&self) -> bool { ... }
    fn start_task(&mut self, task: Task) { ... }
}

// Constructors: new or descriptive
impl Robot {
    fn new(id: RobotId, config: &RobotConfig) -> Self { ... }
    fn from_config(config: &RobotConfig) -> Self { ... }
}
```

### Variables

```rust
// Variables: snake_case
let robot_count = config.robots.count;
let avg_task_time = total_time / task_count;

// Constants: SCREAMING_SNAKE_CASE
const MAX_QUEUE_SIZE: usize = 100;
const DEFAULT_SPEED: f64 = 1.5;
```

---

## Code Organization

### Module Structure

```
crate/
├── lib.rs          # Public API
├── module.rs       # Single-file module
└── module/         # Multi-file module
    ├── mod.rs      # Module root
    ├── types.rs    # Type definitions
    └── impl.rs     # Implementations
```

### Import Order

```rust
// 1. Standard library
use std::collections::HashMap;

// 2. External crates
use serde::{Deserialize, Serialize};

// 3. Crate-level imports
use crate::config::Scenario;

// 4. Module-level imports
use super::Robot;
```

### File Organization

```rust
// lib.rs structure
//! Crate-level documentation

// Public modules
pub mod config;
pub mod entities;

// Re-exports
pub use config::Scenario;
pub use entities::Robot;

// Private modules
mod internal;

// Tests at end
#[cfg(test)]
mod tests;
```

---

## Documentation

### Module Documentation

```rust
//! Module-level documentation
//!
//! This module provides...
//!
//! # Examples
//!
//! ```
//! use waremax::example;
//! ```
```

### Function Documentation

```rust
/// Calculate the shortest path between two nodes.
///
/// Uses Dijkstra's algorithm to find the path with
/// minimum total edge weight.
///
/// # Arguments
///
/// * `from` - Starting node ID
/// * `to` - Destination node ID
/// * `map` - Warehouse map graph
///
/// # Returns
///
/// `Some(Path)` if a path exists, `None` otherwise.
///
/// # Examples
///
/// ```
/// let path = shortest_path(NodeId(0), NodeId(5), &map);
/// ```
pub fn shortest_path(from: NodeId, to: NodeId, map: &Map) -> Option<Path> {
    // ...
}
```

### Inline Comments

```rust
// Good: Explain WHY
// Use capacity 2 because paths can cross in both directions
let capacity = 2;

// Bad: Explain WHAT (obvious from code)
// Set capacity to 2
let capacity = 2;
```

---

## Error Handling

### Use Result for Recoverable Errors

```rust
// Good
fn parse_config(yaml: &str) -> Result<Config, ConfigError> {
    // ...
}

// Avoid panic for recoverable errors
fn parse_config(yaml: &str) -> Config {
    // Don't panic here!
}
```

### Custom Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },

    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}
```

### Error Propagation

```rust
// Use ? operator
fn load_scenario(path: &Path) -> Result<Scenario, Error> {
    let content = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&content)?;
    let scenario = config.validate()?;
    Ok(scenario)
}
```

---

## Testing

### Test Organization

```rust
// Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // ...
    }
}

// Integration tests in tests/ directory
// tests/integration.rs
```

### Test Naming

```rust
#[test]
fn test_robot_starts_idle() { ... }

#[test]
fn test_task_allocation_selects_nearest() { ... }

#[test]
fn test_invalid_config_returns_error() { ... }
```

### Test Structure

```rust
#[test]
fn test_something() {
    // Arrange
    let input = create_test_input();

    // Act
    let result = function_under_test(input);

    // Assert
    assert_eq!(result, expected);
}
```

---

## Pull Requests

### Commit Messages

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:

```
feat(policies): add weighted allocation policy

fix(routing): handle disconnected graph case

docs(readme): update installation instructions

refactor(metrics): simplify collector interface
```

### PR Description

Include:

1. What changes were made
2. Why the changes were needed
3. How to test the changes
4. Breaking changes (if any)

### Review Process

1. All tests must pass
2. Code must be formatted
3. No clippy warnings
4. At least one approval required

---

## Related

- [Testing Guide](testing.md)
- [Documentation Guide](documentation.md)
