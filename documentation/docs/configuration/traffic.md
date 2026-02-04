# Traffic Configuration

Configuration for traffic management and congestion handling.

---

## Schema

```yaml
traffic:
  policy: <string>                 # Default: "wait_at_node"
  edge_capacity_default: <integer> # Default: 1
  node_capacity_default: <integer> # Default: 1
  wait_threshold_s: <float>       # Default: 2.0
  max_reroute_attempts: <integer> # Default: 3
  deadlock_detection: <boolean>   # Default: false
  deadlock_resolver: <string>     # Default: "youngest_backs_up"
  deadlock_check_interval_s: <float> # Default: 0
  reservation_enabled: <boolean>  # Default: false
  reservation_lookahead_s: <float> # Default: 30.0
```

---

## Traffic Policy

### wait_at_node

Robots wait at current node when path is blocked.

```yaml
traffic:
  policy: wait_at_node
```

**Behavior**: Simple waiting, no rerouting
**Best for**: Low traffic scenarios, testing

### reroute_on_wait

Robots attempt to find alternate paths when blocked.

```yaml
traffic:
  policy: reroute_on_wait
  wait_threshold_s: 2.0
  max_reroute_attempts: 3
```

**Behavior**: Wait briefly, then try alternate route
**Best for**: High traffic scenarios

### adaptive_traffic

Dynamically adjusts behavior based on congestion.

```yaml
traffic:
  policy: adaptive_traffic
```

**Behavior**: Combines waiting and rerouting based on conditions
**Best for**: Variable traffic scenarios

---

## Capacity Settings

### edge_capacity_default

**Type**: integer
**Default**: 1

Maximum robots on an edge simultaneously.

```yaml
traffic:
  edge_capacity_default: 2  # Allow 2 robots on each edge
```

### node_capacity_default

**Type**: integer
**Default**: 1

Maximum robots at a node simultaneously.

```yaml
traffic:
  node_capacity_default: 2
```

**Note**: Station nodes may have higher effective capacity based on station concurrency.

---

## Rerouting Parameters

### wait_threshold_s

**Type**: float
**Default**: 2.0

Seconds to wait before attempting reroute.

```yaml
traffic:
  policy: reroute_on_wait
  wait_threshold_s: 3.0  # Wait 3 seconds before rerouting
```

### max_reroute_attempts

**Type**: integer
**Default**: 3

Maximum reroute attempts before giving up.

```yaml
traffic:
  max_reroute_attempts: 5
```

---

## Deadlock Detection

### deadlock_detection

**Type**: boolean
**Default**: false

Enable deadlock detection.

```yaml
traffic:
  deadlock_detection: true
```

### deadlock_resolver

**Type**: string
**Default**: "youngest_backs_up"
**Options**: `youngest_backs_up`, `lowest_priority_aborts`, `wait_and_retry`, `tiered`

Strategy for resolving detected deadlocks.

```yaml
traffic:
  deadlock_detection: true
  deadlock_resolver: youngest_backs_up
```

| Resolver | Description |
|----------|-------------|
| `youngest_backs_up` | Most recently assigned robot backs up |
| `lowest_priority_aborts` | Lowest priority task aborts |
| `wait_and_retry` | Wait and retry with jitter |
| `tiered` | Combined approach |

### deadlock_check_interval_s

**Type**: float
**Default**: 0 (only check on wait)

Interval for periodic deadlock checks.

```yaml
traffic:
  deadlock_detection: true
  deadlock_check_interval_s: 5.0  # Check every 5 seconds
```

---

## Reservation System

### reservation_enabled

**Type**: boolean
**Default**: false

Enable path reservation system.

```yaml
traffic:
  reservation_enabled: true
```

### reservation_lookahead_s

**Type**: float
**Default**: 30.0

Seconds to look ahead when reserving path.

```yaml
traffic:
  reservation_enabled: true
  reservation_lookahead_s: 20.0
```

---

## Complete Examples

### Simple Traffic

```yaml
traffic:
  policy: wait_at_node
  edge_capacity_default: 1
  node_capacity_default: 1
```

### Medium Traffic

```yaml
traffic:
  policy: reroute_on_wait
  edge_capacity_default: 1
  node_capacity_default: 1
  wait_threshold_s: 2.0
  max_reroute_attempts: 3
```

### High Traffic

```yaml
traffic:
  policy: reroute_on_wait
  edge_capacity_default: 2
  node_capacity_default: 2
  wait_threshold_s: 1.5
  max_reroute_attempts: 5
  deadlock_detection: true
  deadlock_resolver: youngest_backs_up
```

### Reservation-Based

```yaml
traffic:
  policy: wait_at_node
  edge_capacity_default: 1
  node_capacity_default: 1
  deadlock_detection: true
  deadlock_resolver: tiered
  reservation_enabled: true
  reservation_lookahead_s: 25.0
```

---

## Configuration Guidelines

### Capacity Selection

| Robot Density | Edge Capacity | Node Capacity |
|---------------|---------------|---------------|
| Low (< 0.1 robot/node) | 1 | 1 |
| Medium (0.1-0.3) | 1-2 | 1-2 |
| High (> 0.3) | 2-3 | 2-3 |

### When to Enable Deadlock Detection

- Many robots in small area
- Complex map topology
- Observing stuck robots

### When to Use Reservations

- Predictable paths
- Time-critical operations
- Avoiding conflicts proactively

---

## Related

- [Traffic Management Concepts](../concepts/traffic/index.md)
- [Congestion Handling](../concepts/traffic/congestion.md)
- [Deadlock Detection](../concepts/traffic/deadlock.md)
