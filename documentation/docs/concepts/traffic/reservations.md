# Reservations

Space reservation systems for coordinated movement.

---

## Concept

Reservations allow robots to claim resources before arriving, enabling coordinated movement and preventing conflicts.

### Without Reservations

Robots discover conflicts on arrival:

```
t=0: R1 starts toward N5
t=0: R2 starts toward N5
t=3: R1 arrives at N5
t=3: R2 arrives at N5 → Conflict!
```

### With Reservations

Conflicts detected in advance:

```
t=0: R1 reserves N5 for t=3
t=0: R2 tries to reserve N5 for t=3 → Denied
t=0: R2 waits or chooses alternative
```

---

## Reservation Types

### Spatial Reservations

Reserve physical location:

```
Reserve: Node N5
Duration: 2 seconds
Holder: Robot R1
```

### Temporal Reservations

Reserve location for time window:

```
Reserve: Node N5
Window: t=10s to t=15s
Holder: Robot R1
```

### Path Reservations

Reserve entire path:

```
Reserve: N1 → N2 → N3 → N4
Windows: N1: t=0-2, N2: t=2-4, N3: t=4-6, N4: t=6-8
Holder: Robot R1
```

---

## Space-Time Reservations

### Time Windows

Reserve resources for specific time intervals:

```
Resource: N5
Time:     ████████████████████████████████
          0    5    10   15   20   25   30

R1:            ████
R2:                      ████
R3:                                 ████

No overlap → No conflict
```

### Reservation Table

```
Node  | t=0-5  | t=5-10 | t=10-15 | t=15-20
------|--------|--------|---------|--------
N1    | R1     |        | R3      |
N2    |        | R1     | R2      |
N3    |        | R2     | R1      |
N4    | R2     |        |         | R3
N5    |        | R1,R2  |         |
            conflict ↑
```

---

## Reservation Process

### Request

Robot requests reservation:

```
Request {
  robot: R1
  resource: N5
  start_time: 10.0
  end_time: 15.0
}
```

### Check

System checks availability:

```
Is N5 free from t=10 to t=15?
  → Check existing reservations
  → No conflicts found
  → Approve
```

### Grant or Deny

```
Approved:
  Reservation granted
  Robot proceeds with plan

Denied:
  Conflict exists
  Robot must wait or replan
```

### Release

After use:

```
Robot leaves N5 at t=14
Reservation released
Resource available for others
```

---

## Conflict Resolution

### First-Come-First-Served

Earlier requests win:

```
t=0: R1 requests N5 for t=10-15 → Granted
t=1: R2 requests N5 for t=12-17 → Denied (conflict)
```

### Priority-Based

Higher priority wins:

```
t=0: R1 (priority 3) requests N5 for t=10-15 → Granted
t=1: R2 (priority 5) requests N5 for t=12-17 → Granted
     R1's reservation revoked or adjusted
```

### Sliding Windows

Adjust times to avoid conflict:

```
R1 requests N5 for t=10-15 → Granted
R2 requests N5 for t=12-17
  → Conflict at t=12-15
  → Slide R2 to t=15-20 → Granted
```

---

## Path Planning with Reservations

### Cooperative Path Planning

All robots plan together:

1. Collect all robot destinations
2. Find conflict-free paths for all
3. Execute coordinated plan

### Prioritized Planning

Plan one at a time:

```
1. Plan path for R1 (highest priority)
   → Reserve all resources
2. Plan path for R2 (avoid R1's reservations)
   → Reserve remaining resources
3. Continue for all robots...
```

### Windowed Planning

Replan periodically:

```
t=0:  Plan for t=0-30
t=30: Replan for t=30-60
t=60: Replan for t=60-90
...
```

---

## Implementation Approaches

### Centralized

Single coordinator manages all reservations:

```
      ┌─────────────┐
      │ Coordinator │
      └──────┬──────┘
             │
    ┌────────┼────────┐
    │        │        │
   R1       R2       R3
```

Pros:
- Global optimization
- No conflicts

Cons:
- Single point of failure
- Scalability limits

### Distributed

Robots negotiate directly:

```
R1 ←──────→ R2
│ ╲        ╱ │
│  ╲      ╱  │
│   ╲    ╱   │
│    ╲  ╱    │
R3 ←──────→ R4
```

Pros:
- Scalable
- Robust

Cons:
- Complex protocols
- Suboptimal solutions

---

## Configuration

```yaml
traffic:
  # Enable reservations
  reservations: true
  reservation_type: space_time

  # Time parameters
  reservation_horizon_s: 30.0
  reservation_buffer_s: 1.0

  # Conflict resolution
  reservation_priority: first_come

routing:
  # Consider reservations in pathfinding
  reservation_aware: true
```

---

## Reservation Metrics

### Utilization

```
Reservation utilization = Time used / Time reserved
```

### Conflicts

| Metric | Description |
|--------|-------------|
| Conflict rate | Conflicts per time period |
| Resolution time | Time to resolve conflict |
| Replanning rate | How often paths change |

### Efficiency

```
Planning overhead = Planning time / Total time
```

---

## Trade-offs

### Reservation Horizon

**Short horizon:**
- Less planning overhead
- More reactive
- May miss opportunities

**Long horizon:**
- Better optimization
- More planning overhead
- Predictions may be wrong

### Granularity

**Fine (small time windows):**
- Precise scheduling
- High overhead
- Complex management

**Coarse (large time windows):**
- Simple management
- Resource waste
- Less flexibility

---

## Best Practices

### Buffer Time

Add safety margins:

```
Actual travel: 4.0s
Reservation: 4.5s (+0.5s buffer)
```

### Timeout Reservations

Cancel stale reservations:

```yaml
traffic:
  reservation_timeout_s: 60.0
```

### Monitor Performance

Track:
- Reservation utilization
- Conflict frequency
- Planning time

---

## Related

- [Deadlock](deadlock.md)
- [Congestion](congestion.md)
- [Routing Configuration](../../configuration/routing.md)
