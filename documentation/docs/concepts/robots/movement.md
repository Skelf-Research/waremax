# Robot Movement

How robots navigate through the warehouse.

---

## Movement Model

### Edge-Based Travel

Robots move along edges between nodes:

```
[Node A] ──── edge (6m) ────> [Node B]
   │                              │
   t=0                         t=4s

At speed 1.5 m/s: 6m / 1.5 = 4 seconds
```

### Speed

Constant speed during travel:

```yaml
robots:
  speed_m_s: 1.5
```

### Travel Time Calculation

```
travel_time = edge_length / robot_speed
```

---

## Pathfinding

### Shortest Path

Default: Find path with minimum total distance.

```
A ──3── B ──2── C
│               │
4               3
│               │
D ──────5────── E

Shortest A→E: A→B→C→E (8m)
Not: A→D→E (9m)
```

### A* Algorithm

Efficient pathfinding using heuristic:

```
f(n) = g(n) + h(n)
```

Where:

- `g(n)`: Actual distance from start
- `h(n)`: Estimated distance to goal (Euclidean)

### Congestion-Aware Routing

Optional: Avoid congested areas:

```yaml
routing:
  policy: congestion_aware
  congestion_weight: 1.5
```

Cost becomes:

```
cost = distance + weight × congestion_score
```

---

## Movement Events

### Timeline

```
t=0:   Robot at Node A, assigned task at Node D
t=0:   Path calculated: A → B → C → D
t=0:   Start moving on edge A→B
t=3:   Arrive at Node B
t=3:   Start moving on edge B→C
t=5:   Arrive at Node C
t=5:   Start moving on edge C→D
t=9:   Arrive at Node D (destination)
```

### Event Types

| Event | Description |
|-------|-------------|
| `StartEdgeTraversal` | Begin moving on edge |
| `EndEdgeTraversal` | Arrive at destination node |
| `PathBlocked` | Cannot proceed (congestion) |
| `PathRerouted` | Alternative route taken |

---

## Traffic Interaction

### Waiting for Access

When next node/edge is occupied:

1. Robot waits at current node
2. Monitors for availability
3. Proceeds when clear

### Rerouting

When path is blocked and rerouting enabled:

1. Detect blockage
2. Calculate alternative path
3. Resume movement on new path

---

## Speed Variations

### Constant Speed (Default)

```yaml
robots:
  speed_m_s: 1.5
```

All robots same speed, always.

### Speed Distribution

Variable speeds across fleet:

```yaml
robots:
  speed_m_s:
    distribution: normal
    mean: 1.5
    stddev: 0.2
```

### Loaded vs. Unloaded

Different speeds when carrying items:

```yaml
robots:
  speed_m_s: 1.5
  loaded_speed_m_s: 1.2
```

---

## Turn Handling

### Simplified Model

Turns are instantaneous (no turn time).

### Turn Costs (Optional)

Add time penalty for turns:

```yaml
routing:
  turn_penalty_s: 0.5
```

Affects path selection - may prefer straighter routes.

---

## Example Movement Trace

```
[10:00:00.000] Robot R1: Assigned task T1, destination N15
[10:00:00.001] Robot R1: Path calculated [N1→N5→N10→N15], distance 12.0m
[10:00:00.002] Robot R1: Starting edge N1→N5 (length 4.0m)
[10:00:02.668] Robot R1: Arrived at N5
[10:00:02.669] Robot R1: Starting edge N5→N10 (length 4.0m)
[10:00:05.336] Robot R1: Arrived at N10
[10:00:05.337] Robot R1: Starting edge N10→N15 (length 4.0m)
[10:00:08.004] Robot R1: Arrived at N15 (destination reached)
```

---

## Performance Factors

### Travel Efficiency

- **Path quality**: Shorter paths = faster completion
- **Congestion**: Waiting time reduces efficiency
- **Layout**: Well-designed maps minimize travel

### Bottleneck Identification

Look for:

- High-traffic edges
- Frequently blocked nodes
- Long average wait times

---

## Related

- [Nodes & Edges](../warehouse/nodes-edges.md)
- [Graph-Based Maps](../warehouse/maps.md)
- [Congestion](../traffic/congestion.md)
