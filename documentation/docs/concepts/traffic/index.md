# Traffic Management

How robots share space and avoid conflicts.

---

## Overview

Traffic management ensures robots navigate efficiently without collisions or deadlocks. It's critical for warehouse performance.

| Topic | Description |
|-------|-------------|
| [Capacity](capacity.md) | Node and edge capacity limits |
| [Congestion](congestion.md) | Traffic buildup and measurement |
| [Deadlock](deadlock.md) | Detection and prevention |
| [Reservations](reservations.md) | Space reservation systems |

---

## The Traffic Problem

### Shared Resources

Robots share limited space:

- **Nodes**: Discrete locations
- **Edges**: Paths between nodes

### Conflicts

What happens when resources are contested:

```
Robot A wants: N1 → N2 → N3
Robot B wants: N4 → N2 → N5
                    ↑
                Conflict at N2
```

---

## Traffic Model

### Capacity-Based

Each node and edge has maximum occupancy:

```yaml
traffic:
  node_capacity_default: 1
  edge_capacity_default: 1
```

### Collision Prevention

When at capacity:

- New robots **wait**
- Or **reroute** (if configured)

---

## Traffic States

### Free Flow

All robots move without delay:

```
Resources < Capacity everywhere
Wait time ≈ 0
```

### Congested

Some delays occur:

```
Some resources at capacity
Wait time > 0
Throughput reduced
```

### Gridlock

Movement stops:

```
Circular waiting
Deadlock potential
Intervention needed
```

---

## Traffic Visualization

### Light Traffic

```
[N1]     [N2]     [N3]
  ●───────────────●
        R1 →
```

### Moderate Traffic

```
[N1]     [N2]     [N3]
  ●───────●───────●
     R1 →    R2 →
```

### Heavy Traffic

```
[N1]     [N2]     [N3]
  ●───────●───────●
     R1    R2 R3
       ↑
    Waiting
```

---

## Traffic Management Strategies

### Reactive

Respond to congestion:

- Wait when blocked
- Reroute when path congested
- Queue at destinations

### Proactive

Prevent congestion:

- Reserve paths ahead
- Route around busy areas
- Coordinate movements

### Hybrid

Combine approaches:

- Reserve critical sections
- React to unexpected congestion
- Balance efficiency and safety

---

## Configuration Overview

```yaml
traffic:
  # Capacity limits
  node_capacity_default: 1
  edge_capacity_default: 1

  # Congestion handling
  wait_on_blocked: true
  reroute_on_congestion: false
  reroute_threshold: 3.0

  # Deadlock prevention
  deadlock_detection: true
  deadlock_resolution: priority
```

---

## Performance Impact

### Throughput vs. Fleet Size

```
Throughput
    │
    │         ┌──── Congestion limit
    │        /│
    │       / │
    │      /  │
    │     /   │
    │    /    │
    │   /     │
    │  /      │
    │ /       │
    │/        │
    └─────────┴─────── Fleet Size
              ↑
        Optimal size
```

### Key Relationships

- More robots → More congestion
- Better traffic management → Higher optimal fleet size
- Poor layout → Lower congestion threshold

---

## Related

- [Traffic Configuration](../../configuration/traffic.md)
- [Routing Configuration](../../configuration/routing.md)
- [Robot Movement](../robots/movement.md)
