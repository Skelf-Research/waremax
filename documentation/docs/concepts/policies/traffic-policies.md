# Traffic Policies

Policies governing robot movement and congestion.

---

## Overview

Traffic policies control how robots navigate, handle congestion, and resolve conflicts.

| Policy Area | Decisions |
|-------------|-----------|
| Routing | Which path to take |
| Congestion | Wait or reroute |
| Deadlock | How to resolve |
| Priority | Who goes first |

---

## Routing Policies

### Shortest Path

Always take minimum distance route:

```yaml
routing:
  policy: shortest_path
```

```
A ──3── B ──2── C
│               │
4               3
│               │
D ──────5────── E

A to E: A→B→C→E (8m) ✓
Not:    A→D→E (9m)
```

**Pros**: Minimum travel distance
**Cons**: May route through congestion

---

### Congestion-Aware

Avoid congested areas:

```yaml
routing:
  policy: congestion_aware
  congestion_weight: 1.5
```

**Cost calculation:**

```
path_cost = distance + congestion_weight × congestion_score
```

```
Path 1: 8m, congestion=5  → cost = 8 + 1.5×5 = 15.5
Path 2: 10m, congestion=1 → cost = 10 + 1.5×1 = 11.5 ✓
```

**Pros**: Avoids delays
**Cons**: Longer distances, requires congestion data

---

### Time-Optimal

Minimize expected travel time:

```yaml
routing:
  policy: time_optimal
```

Considers:
- Distance
- Current congestion
- Historical patterns
- Expected wait times

---

## Congestion Response

### Wait

Robot waits when path blocked:

```yaml
traffic:
  congestion_response: wait
```

```
R1 → [blocked] → destination

R1 waits at current position
```

**Pros**: Simple, no replanning
**Cons**: May wait long time

---

### Reroute

Find alternative path:

```yaml
traffic:
  congestion_response: reroute
  reroute_threshold_s: 5.0
```

```
R1 → [blocked] → destination
          ↓
R1 → [alternative path] → destination
```

**Pros**: Avoids long waits
**Cons**: Longer distance, replanning overhead

---

### Hybrid

Combine strategies:

```yaml
traffic:
  congestion_response: hybrid
  wait_threshold_s: 3.0
  reroute_threshold_s: 10.0
```

```
Wait < 3s:  Wait
3s-10s:     Continue waiting
Wait > 10s: Reroute
```

---

## Deadlock Policies

### Priority Resolution

Higher priority robot proceeds:

```yaml
traffic:
  deadlock_resolution: priority
```

```
R1 (priority 7) ←→ R2 (priority 3)

R2 backs off, R1 proceeds
```

---

### Time-Based Resolution

Robot waiting longer proceeds:

```yaml
traffic:
  deadlock_resolution: time_based
```

```
R1 waiting 5s ←→ R2 waiting 2s

R1 proceeds (waited longer)
```

---

### Backoff Resolution

One robot retreats:

```yaml
traffic:
  deadlock_resolution: backoff
  backoff_selection: random
```

Options: `random`, `lower_priority`, `further_from_goal`

---

## Traffic Flow Policies

### One-Way Aisles

Enforce directional flow:

```yaml
traffic:
  one_way_enforcement: true
  aisle_direction:
    aisle_1: forward  # →
    aisle_2: backward # ←
```

```
Aisle 1: →→→→→→→→→
Aisle 2: ←←←←←←←←←
```

---

### Lane Discipline

Multiple lanes with rules:

```yaml
traffic:
  lanes:
    main_aisle:
      lanes: 2
      policy: keep_right
```

```
Lane 1: →→→→→ (rightward traffic)
Lane 2: ←←←←← (leftward traffic)
```

---

### Speed Zones

Variable speed by area:

```yaml
traffic:
  speed_zones:
    main_aisle:
      max_speed_m_s: 2.0
    intersection:
      max_speed_m_s: 0.5
```

---

## Priority in Traffic

### Right-of-Way

Higher priority robots have precedence:

```yaml
traffic:
  right_of_way: priority_based
```

```
Intersection conflict:
  R1 (priority 7) vs R2 (priority 3)
  → R1 goes first
```

---

### Emergency Priority

Emergency tasks override all:

```yaml
traffic:
  emergency_priority: true
  emergency_threshold: 9
```

Priority 9+ robots:
- Skip queues
- Others yield immediately
- Reroute lower priority robots

---

## Configuration Example

```yaml
routing:
  policy: congestion_aware
  congestion_weight: 1.5
  replan_on_congestion: true
  replan_threshold_s: 10.0

traffic:
  # Congestion response
  congestion_response: hybrid
  wait_threshold_s: 3.0
  reroute_threshold_s: 15.0

  # Deadlock handling
  deadlock_detection: true
  deadlock_resolution: priority
  deadlock_timeout_s: 30.0

  # Flow control
  one_way_enforcement: false
  right_of_way: priority_based

  # Speed
  default_speed_m_s: 1.5
  intersection_speed_m_s: 0.8
```

---

## Policy Interactions

### Routing + Congestion

```
Congestion-aware routing → Fewer blocks → Less rerouting
```

### Priority + Deadlock

```
Priority routing → Fewer conflicts → Simpler deadlock resolution
```

### Batching + Traffic

```
Batching → Fewer trips → Less congestion → Better throughput
```

---

## Comparing Policies

### Simulation Comparison

```bash
waremax compare scenario.yaml \
  --param routing.policy=shortest_path \
  --param routing.policy=congestion_aware
```

### Metrics to Watch

| Metric | Indicates |
|--------|-----------|
| Travel time | Route efficiency |
| Wait time | Congestion handling |
| Deadlocks | Resolution effectiveness |
| Throughput | Overall efficiency |

---

## Best Practices

### Start Simple

Begin with basic policies:

```yaml
routing:
  policy: shortest_path

traffic:
  congestion_response: wait
```

### Add Complexity as Needed

If congestion is a problem:

```yaml
routing:
  policy: congestion_aware
```

If deadlocks occur:

```yaml
traffic:
  deadlock_detection: true
  deadlock_resolution: priority
```

### Monitor and Tune

Track key metrics and adjust:

- High wait time → Enable rerouting
- Frequent deadlocks → Improve prevention
- Uneven flow → Consider one-way aisles

---

## Related

- [Traffic Concepts](../traffic/index.md)
- [Congestion](../traffic/congestion.md)
- [Deadlock](../traffic/deadlock.md)
- [Routing Configuration](../../configuration/routing.md)
