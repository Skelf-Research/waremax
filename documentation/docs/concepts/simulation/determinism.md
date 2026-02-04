# Determinism & Seeds

How Waremax ensures reproducible simulations.

---

## What is Determinism?

A deterministic simulation produces identical results when run with the same inputs.

**Same seed + Same configuration = Same results**

Every time.

---

## Why Determinism Matters

### Reproducibility

- Debug specific scenarios
- Share exact test cases
- Verify bug fixes

### Fair Comparisons

- Compare policies with same random events
- Isolate effect of changes
- Statistical validity

### Testing

- Regression testing
- CI/CD pipelines
- Validation

---

## Random Seed

### What is a Seed?

A seed initializes the random number generator (RNG).

```yaml
seed: 42  # Any positive integer
```

### How Seeds Work

```
Seed → RNG State → Sequence of Random Numbers
```

Same seed = same sequence = same simulation

### Seed in Configuration

```yaml
seed: 12345

simulation:
  duration_minutes: 60
```

### Override at Runtime

```bash
waremax run --scenario scenario.yaml --seed 99999
```

---

## What's Random?

### Order Arrivals

Poisson process uses RNG for inter-arrival times.

### Service Times

Variable distributions (lognormal, exponential) use RNG.

### SKU Selection

Zipf distribution samples use RNG.

### Failures

MTBF-based failures use RNG for timing.

---

## Determinism Guarantees

### Guaranteed Identical

- Order arrival times
- Order contents (lines, SKUs)
- Service durations
- Failure times
- Route choices (same congestion state)

### Execution Independent

- Results don't depend on:
  - CPU speed
  - System load
  - Time of day
  - Previous runs

---

## Multiple Replications

For statistical analysis, run with different seeds:

```bash
# Replication 1
waremax run --scenario scenario.yaml --seed 1

# Replication 2
waremax run --scenario scenario.yaml --seed 2

# Replication 3
waremax run --scenario scenario.yaml --seed 3
```

Each replication is deterministic individually.

### Parameter Sweeps

```bash
waremax sweep \
  --base scenario.yaml \
  --sweep "robots:5,10,15" \
  --replications 5  # Different seeds for each
```

---

## Debugging with Seeds

### Reproduce a Problem

```bash
# Run with specific seed
waremax run --scenario scenario.yaml --seed 42

# Problem occurs
# Fix code
# Verify with same seed
waremax run --scenario scenario.yaml --seed 42
```

### Find Problematic Seeds

```bash
for seed in $(seq 1 100); do
  waremax run --scenario scenario.yaml --seed $seed > /dev/null
  if [ $? -ne 0 ]; then
    echo "Problem with seed: $seed"
  fi
done
```

---

## Implementation Details

### RNG Algorithm

Waremax uses ChaCha8 (via `rand_chacha`):

- Cryptographically secure
- Fast
- Deterministic
- Platform-independent

### Seeding Strategy

Single seed seeds all random processes:

```
Main Seed → RNG
         → Order arrivals
         → Service times
         → SKU selection
         → Failures
         → etc.
```

### Thread Safety

Single-threaded simulation ensures determinism.

---

## Best Practices

### Document Seeds

```yaml
# seed: 42 - baseline scenario
# seed: 12345 - stress test case
# seed: 99999 - edge case testing
seed: 42
```

### Use Different Seeds for Testing

```yaml
# Development seed
seed: 1

# But test with multiple seeds before release
```

### Named Seeds

```python
SEEDS = {
    "baseline": 42,
    "stress": 12345,
    "edge_case": 99999,
    "production_scenario": 54321,
}
```

---

## Troubleshooting

### Results Differ Between Runs

Check:

1. Same configuration file?
2. Same seed?
3. Same Waremax version?
4. Same input files (map, storage)?

### Results Differ Between Machines

Should not happen with same:

- Configuration
- Seed
- Waremax version (from same build)

If it does, report as bug.

---

## Related

- [Simulation Configuration](../../configuration/simulation.md)
- [Time Model](time-model.md)
- [Events](events.md)
