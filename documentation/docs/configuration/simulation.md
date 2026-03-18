# Simulation Settings

Configuration for simulation timing and parameters.

---

## Schema

```yaml
simulation:
  duration_minutes: <float>        # Required
  warmup_minutes: <float>          # Default: 0
  time_unit: <string>              # Default: "seconds"
```

---

## Fields

### duration_minutes

**Type**: float
**Required**: Yes

Total simulation duration in minutes.

```yaml
simulation:
  duration_minutes: 60  # 1-hour simulation
```

**Guidelines**:

- Short (1-10 min): Quick tests, debugging
- Medium (30-60 min): Standard experiments
- Long (120+ min): Statistical stability, rare events

### warmup_minutes

**Type**: float
**Default**: 0

Warmup period before metrics collection begins.

```yaml
simulation:
  duration_minutes: 60
  warmup_minutes: 10  # 10 minutes warmup, 50 minutes measured
```

**Purpose**:

- Allows system to reach steady state
- Eliminates startup transients from metrics
- Provides realistic initial conditions

**Guidelines**:

| Scenario | Recommended Warmup |
|----------|-------------------|
| Quick tests | 0-1 min |
| Standard | 5-10 min |
| Baseline comparisons | 10-20 min |
| Long-duration | 10-20% of duration |

### time_unit

**Type**: string
**Default**: "seconds"
**Options**: "seconds", "minutes"

Internal time unit for the simulation.

```yaml
simulation:
  duration_minutes: 60
  time_unit: seconds  # Default
```

!!! note
    This is primarily internal. Most user-facing values are already in appropriate units.

---

## Examples

### Quick Test

```yaml
simulation:
  duration_minutes: 5
  warmup_minutes: 1
```

### Standard Experiment

```yaml
simulation:
  duration_minutes: 60
  warmup_minutes: 10
```

### Reproducible Baseline

```yaml
simulation:
  duration_minutes: 120
  warmup_minutes: 60
```

### Long-Running Analysis

```yaml
simulation:
  duration_minutes: 480  # 8 hours
  warmup_minutes: 60
```

---

## Seed

The random seed is specified at the top level, not in the simulation section:

```yaml
seed: 42  # Random seed for reproducibility

simulation:
  duration_minutes: 60
  warmup_minutes: 10
```

**Seed behavior**:

- Same seed + same config = identical results
- Different seeds = different random sequences
- Override at runtime with `--seed` flag

---

## Effective Duration

Metrics are only collected during the effective measurement period:

```
Effective duration = duration_minutes - warmup_minutes
```

Example:

```yaml
simulation:
  duration_minutes: 60
  warmup_minutes: 10
# Effective measurement: 50 minutes
```

---

## Performance Considerations

### Duration Impact

| Duration | Typical Use | Run Time (Release Build) |
|----------|-------------|-------------------------|
| 1 min | Unit tests | < 1 second |
| 5 min | Quick iteration | 1-5 seconds |
| 30 min | Standard test | 5-30 seconds |
| 60 min | Full simulation | 10-60 seconds |
| 480 min | Long analysis | 1-5 minutes |

*Actual times depend on scenario complexity and hardware.*

### Statistical Stability

Longer durations provide more stable statistics:

- **Short runs**: High variance, quick iteration
- **Long runs**: Stable means, reliable for comparison

For statistical comparisons, ensure adequate duration or use multiple replications.

---

## Related

- [Determinism & Seeds](../concepts/simulation/determinism.md)
- [Time Model](../concepts/simulation/time-model.md)
