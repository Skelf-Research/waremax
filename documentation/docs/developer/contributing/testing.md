# Testing Guide

Testing requirements and best practices.

---

## Test Categories

### Unit Tests

Test individual functions and methods:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_distance() {
        let result = calculate_distance(0.0, 0.0, 3.0, 4.0);
        assert!((result - 5.0).abs() < 0.001);
    }
}
```

Location: Same file as code being tested.

### Integration Tests

Test multiple components together:

```rust
// tests/simulation.rs
use waremax::prelude::*;

#[test]
fn test_full_simulation() {
    let scenario = Scenario::from_preset(Preset::Minimal);
    let result = run_simulation(scenario);

    assert!(result.is_ok());
    assert!(result.unwrap().tasks_completed > 0);
}
```

Location: `tests/` directory.

### Property-Based Tests

Test invariants across many inputs:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_distance_is_non_negative(x1: f64, y1: f64, x2: f64, y2: f64) {
        let dist = calculate_distance(x1, y1, x2, y2);
        prop_assert!(dist >= 0.0);
    }
}
```

---

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Crate

```bash
cargo test -p waremax-core
```

### Specific Test

```bash
cargo test test_robot_starts_idle
```

### With Output

```bash
cargo test -- --nocapture
```

### Ignored Tests

```bash
cargo test -- --ignored
```

---

## Test Requirements

### Coverage Goals

- New code should have tests
- Bug fixes should include regression tests
- Critical paths should have integration tests

### What to Test

| Component | Test Type |
|-----------|-----------|
| Pure functions | Unit tests |
| State machines | State transition tests |
| Policies | Decision outcome tests |
| Configuration | Parsing + validation tests |
| Simulation | Integration tests |

### What NOT to Test

- Trivial getters/setters
- External library code
- Implementation details (test behavior)

---

## Test Patterns

### Setup and Teardown

```rust
fn create_test_scenario() -> Scenario {
    ScenarioBuilder::new()
        .duration(300)
        .robots(5)
        .stations(2)
        .build()
}

#[test]
fn test_something() {
    let scenario = create_test_scenario();
    // ...
}
```

### Test Fixtures

```rust
// tests/fixtures.rs
pub fn minimal_map() -> WarehouseMap {
    MapBuilder::grid(3, 3).build()
}

pub fn test_robot() -> Robot {
    Robot::new(RobotId(1), NodeId(0), &RobotConfig::default())
}
```

### Parameterized Tests

```rust
#[test_case(0, 0, 3, 4, 5.0 ; "3-4-5 triangle")]
#[test_case(0, 0, 1, 0, 1.0 ; "horizontal")]
#[test_case(0, 0, 0, 1, 1.0 ; "vertical")]
fn test_distance(x1: f64, y1: f64, x2: f64, y2: f64, expected: f64) {
    let result = calculate_distance(x1, y1, x2, y2);
    assert!((result - expected).abs() < 0.001);
}
```

---

## Determinism Testing

### Reproducibility

```rust
#[test]
fn test_simulation_is_deterministic() {
    let scenario = Scenario::with_seed(12345);

    let result1 = run_simulation(scenario.clone());
    let result2 = run_simulation(scenario.clone());

    assert_eq!(result1.tasks_completed, result2.tasks_completed);
    assert_eq!(result1.events, result2.events);
}
```

### Fixed Seeds

```rust
#[test]
fn test_with_known_seed() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    // Use rng for all randomness
}
```

---

## Performance Testing

### Benchmarks

```rust
// benches/simulation.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_pathfinding(c: &mut Criterion) {
    let map = create_large_map();

    c.bench_function("shortest_path", |b| {
        b.iter(|| {
            shortest_path(
                black_box(NodeId(0)),
                black_box(NodeId(100)),
                black_box(&map),
            )
        })
    });
}

criterion_group!(benches, benchmark_pathfinding);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench
```

### Regression Guards

```rust
#[test]
fn test_simulation_performance() {
    let scenario = Scenario::from_preset(Preset::Standard);
    let start = Instant::now();

    run_simulation(scenario);

    let elapsed = start.elapsed();
    // Standard scenario should complete in under 10 seconds
    assert!(elapsed < Duration::from_secs(10));
}
```

---

## Mocking

### Simple Mocks

```rust
struct MockPolicy;

impl TaskAllocationPolicy for MockPolicy {
    fn allocate(&self, _task: &Task, robots: &[Robot], _map: &Map) -> Option<RobotId> {
        // Always return first idle robot
        robots.iter().find(|r| r.is_idle()).map(|r| r.id())
    }
}
```

### Using mockall

```rust
use mockall::automock;

#[automock]
trait PathFinder {
    fn find_path(&self, from: NodeId, to: NodeId) -> Option<Path>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockPathFinder::new();
    mock.expect_find_path()
        .returning(|_, _| Some(Path::direct()));

    // Use mock in test
}
```

---

## Test Documentation

### Test Names

```rust
// Good: Descriptive
#[test]
fn test_robot_returns_to_idle_after_task_completion() { ... }

// Bad: Vague
#[test]
fn test_robot() { ... }
```

### Test Comments

```rust
#[test]
fn test_deadlock_detection() {
    // Setup: Create two robots that will deadlock
    //   R1 at N1, wants N2
    //   R2 at N2, wants N1

    let r1 = Robot::new(RobotId(1), NodeId(1), ...);
    let r2 = Robot::new(RobotId(2), NodeId(2), ...);

    // Act: Attempt simultaneous movement
    r1.move_to(NodeId(2));
    r2.move_to(NodeId(1));

    // Assert: Deadlock should be detected
    assert!(detect_deadlock(&[r1, r2]));
}
```

---

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all
```

### Required Checks

- All tests pass
- No warnings
- Code formatted
- Documentation builds

---

## Related

- [Code Style](code-style.md)
- [Documentation](documentation.md)
