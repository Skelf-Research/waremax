# waremax validate

Validate a scenario file without running the simulation.

---

## Synopsis

```bash
waremax validate --scenario <PATH>
```

---

## Description

The `validate` command checks a scenario file for errors and configuration issues without executing the simulation. It verifies:

- YAML syntax
- Required fields
- Valid parameter values
- File references (map, storage)
- Station node assignments
- Policy configurations

---

## Options

| Option | Description |
|--------|-------------|
| `--scenario`, `-s` | Path to the scenario YAML file to validate |

---

## Examples

### Validate a scenario

```bash
waremax validate --scenario my_scenario.yaml
```

### Validate before running

```bash
waremax validate --scenario my_scenario.yaml && \
waremax run --scenario my_scenario.yaml
```

---

## Output

### Valid Scenario

```
Validating scenario: my_scenario.yaml
Scenario valid!
  Seed: 42
  Duration: 30 minutes
  Warmup: 5 minutes
  Robot count: 10
  Stations: 4
```

### Valid with Warnings

```
Validating scenario: my_scenario.yaml
Scenario valid!
  Seed: 42
  Duration: 30 minutes
  Warmup: 5 minutes
  Robot count: 10
  Stations: 4

Warnings (2):
  - Warmup period is less than 10% of duration
  - Station S2 has no queue capacity limit
```

### Invalid Scenario

```
Validating scenario: bad_scenario.yaml
Validation failed with 3 error(s):
  - Missing required field: simulation.duration_minutes
  - Invalid station type: 'picking' (expected: pick, drop, inbound, outbound)
  - Robot count must be positive, got: 0
```

### File Not Found

```
Validating scenario: nonexistent.yaml
Failed to load scenario: IO error: No such file or directory
```

---

## Validation Checks

### Required Fields

- `seed`
- `simulation.duration_minutes`
- `robots.count`
- `stations` (at least one)
- `orders.arrival_process`
- `orders.lines_per_order`
- `orders.sku_popularity`

### Value Constraints

| Field | Constraint |
|-------|------------|
| `seed` | Positive integer |
| `duration_minutes` | > 0 |
| `warmup_minutes` | >= 0 |
| `robots.count` | > 0 |
| `robots.max_speed_mps` | > 0 |
| `robots.max_payload_kg` | > 0 |
| `station.concurrency` | > 0 |
| `arrival_process.rate_per_min` | > 0 |

### File References

- Map file exists (warning if not found)
- Storage file exists (warning if not found)

### Cross-References

- Station nodes should exist in map
- Charging station nodes should exist in map
- Maintenance station nodes should exist in map

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Scenario is valid |
| 1 | Validation failed |

---

## Use Cases

### Pre-flight Check

```bash
# Validate before expensive simulation
if waremax validate --scenario production.yaml; then
  waremax run --scenario production.yaml \
    --output-dir ./results
fi
```

### CI/CD Pipeline

```bash
# Validate all scenarios in directory
for f in scenarios/*.yaml; do
  echo "Validating $f"
  waremax validate --scenario "$f" || exit 1
done
echo "All scenarios valid"
```

### Debugging Configuration

```bash
# Check specific scenario
waremax validate --scenario problematic.yaml
# Fix errors based on output
# Re-validate
```

---

## See Also

- [run](run.md) - Run simulations
- [generate](generate.md) - Generate scenarios
- [Scenario Files](../user-guide/scenario-files.md) - Configuration reference
