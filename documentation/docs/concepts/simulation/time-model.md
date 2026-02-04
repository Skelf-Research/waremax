# Time Model

How time is represented and advances in Waremax.

---

## Time Representation

### SimTime

Time is represented as a floating-point number of seconds:

```
SimTime = f64 (seconds since simulation start)
```

### Precision

- Resolution: microsecond-level
- Range: 0 to simulation duration
- No fixed time steps

---

## Time Advancement

### Event-Driven

Time only advances when processing events:

```
current_time = 0.0s
process(event at 10.5s) → current_time = 10.5s
process(event at 10.5s) → current_time = 10.5s (same time)
process(event at 23.8s) → current_time = 23.8s
```

### No Idle Steps

If nothing happens for 30 seconds:

- Time-stepping: 30+ iterations
- DES: 0 iterations, jump directly

---

## Simulation Phases

### Initialization (t = 0)

- Create robots at starting positions
- Initialize stations
- Schedule first order arrival
- Set up initial state

### Warmup Period

```
0 ────────────── warmup_end ────────────── duration
    Warmup            |       Measurement
    (no metrics)      |       (collect metrics)
```

During warmup:

- Events processed normally
- Metrics not collected
- System reaches steady state

### Measurement Period

After warmup:

- All metrics collected
- KPIs calculated
- Results meaningful

### Termination

Simulation ends when:

- `current_time >= duration`
- Event queue is empty

---

## Duration Configuration

```yaml
simulation:
  duration_minutes: 60    # Total simulation time
  warmup_minutes: 10      # Warmup period
```

### Effective Duration

```
Effective = duration_minutes - warmup_minutes
```

Example:

- Duration: 60 minutes
- Warmup: 10 minutes
- Effective: 50 minutes of metrics

---

## Time Calculations

### Travel Time

```
travel_time = edge_length / robot_speed
```

Example:

- Edge: 15 meters
- Speed: 1.5 m/s
- Time: 15 / 1.5 = 10 seconds

### Service Time

Depends on distribution:

```
# Constant
time = base + per_item × items

# Lognormal
time = sample_lognormal(base, stddev) + per_item × items

# Exponential
time = sample_exponential(mean)
```

### Battery Consumption

```
energy_per_second = power_w / 3600
time_at_idle = energy_consumed_wh / (idle_power_w / 3600)
```

---

## Event Ordering

When events have the same time:

1. **Priority order** by event type
2. **FIFO** for same type

This ensures:

- Deterministic behavior
- Logical event ordering
- Consistent results

### Type Priorities

```
1. SystemEvents (highest)
2. OrderArrival
3. TaskAssignment
4. RobotDepart
5. RobotArrive
6. ServiceStart
7. ServiceEnd
8. MetricsSample (lowest)
```

---

## Time Units in Configuration

Configuration uses minutes for readability:

```yaml
simulation:
  duration_minutes: 60      # Minutes
  warmup_minutes: 10        # Minutes
```

Internally converted to seconds:

```
duration_s = duration_minutes × 60
warmup_s = warmup_minutes × 60
```

---

## Common Time Scales

| Activity | Typical Duration |
|----------|------------------|
| Edge traversal | 2-30 seconds |
| Station service | 5-60 seconds |
| Charging (full) | 30-120 minutes |
| Maintenance | 5-15 minutes |
| Order cycle | 30-180 seconds |

---

## Related

- [Discrete Event Simulation](discrete-event.md) - How DES works
- [Event Types](events.md) - Specific events
- [Simulation Configuration](../../configuration/simulation.md) - Duration settings
