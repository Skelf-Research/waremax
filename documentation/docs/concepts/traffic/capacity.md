# Capacity

Resource limits that control robot density.

---

## Concept

Capacity defines how many robots can occupy a resource simultaneously.

### Node Capacity

Maximum robots at a location:

```yaml
traffic:
  node_capacity_default: 1
```

### Edge Capacity

Maximum robots on a path:

```yaml
traffic:
  edge_capacity_default: 1
```

---

## Why Capacity Matters

### Safety

Prevents physical collisions:

```
Node capacity = 1
  → Only one robot at each location
  → No collision possible
```

### Realism

Models actual constraints:

- Aisle widths
- Intersection sizes
- Station layouts

### Performance

Affects traffic flow:

```
Higher capacity → More throughput
                → More complex coordination
```

---

## Node Capacity

### Single Occupancy (Default)

```
Capacity = 1

Before:  [●]  R1 at node

Request: R2 wants node

Result:  R2 waits
```

### Multi-Occupancy

```yaml
# Intersection with capacity 4
nodes:
  - id: "intersection_1"
    capacity: 4
```

```
Capacity = 4

[● ● ●]  R1, R2, R3 at node (3/4)

Request: R4 wants node

Result:  R4 enters (4/4)
```

### Use Cases for Higher Capacity

| Location | Typical Capacity |
|----------|------------------|
| Regular aisle | 1 |
| Wide intersection | 2-4 |
| Waiting area | 5-10 |
| Station approach | 2-3 |

---

## Edge Capacity

### Single Lane (Default)

```
Capacity = 1

[N1]═══●═══[N2]
       R1

R2 must wait at N1 or N2
```

### Multi-Lane

```yaml
edges:
  - from: "N1"
    to: "N2"
    capacity: 2
```

```
Capacity = 2

[N1]═══●═══[N2]
       R1
    ═══●═══
       R2

Two robots can traverse simultaneously
```

### Use Cases for Higher Capacity

| Path Type | Typical Capacity |
|-----------|------------------|
| Regular aisle | 1 |
| Wide main aisle | 2-3 |
| Highway corridor | 3-4 |

---

## Capacity and Direction

### Bidirectional Edges

Same capacity for both directions:

```
Capacity = 1

[N1]══●══[N2]
      R1 →

R2 ← cannot enter until R1 exits
```

### One-Way Edges

Capacity applies to allowed direction only:

```
Capacity = 2, one-way →

[N1]══●══[N2]
      R1 →
   ══●══
      R2 →

Both traveling same direction
```

---

## Capacity Enforcement

### Blocking

When at capacity, robots wait:

```
1. R1 at node N2
2. R2 requests N2
3. N2 at capacity (1/1)
4. R2 waits at current location
5. R1 leaves N2
6. R2 enters N2
```

### Queue Formation

Multiple waiting robots form queues:

```
[N1]  [N2]  [N3]
 R3    R2    R1
 ←     ←     ●

R1 at destination
R2 waiting for R1
R3 waiting for R2
```

---

## Capacity Configuration

### Global Defaults

```yaml
traffic:
  node_capacity_default: 1
  edge_capacity_default: 1
```

### Per-Resource Overrides

```yaml
nodes:
  - id: "main_intersection"
    capacity: 4
  - id: "station_S1_approach"
    capacity: 3

edges:
  - from: "A1"
    to: "A2"
    capacity: 2  # Wide aisle
```

---

## Capacity Planning

### Bottleneck Identification

Low capacity + high traffic = bottleneck:

```
Traffic: ████████ (high)
Capacity: █ (low)
         ↓
    Bottleneck ⚠️
```

### Solutions

1. **Increase capacity**: Widen aisles, expand intersections
2. **Add routes**: Parallel paths reduce load
3. **Reduce traffic**: Better task distribution

### Simulation-Based Tuning

1. Run simulation with baseline capacity
2. Identify congestion hotspots
3. Increase capacity at hotspots
4. Verify improvement

---

## Effective Throughput

### Theoretical Maximum

```
Max throughput = capacity × 3600 / avg_traversal_time
```

### Example

Edge with:
- Capacity: 2
- Length: 6m
- Speed: 1.5 m/s
- Traversal time: 4s

```
Max throughput = 2 × 3600 / 4 = 1800 robots/hour
```

### Real Throughput

Always lower due to:
- Entry/exit delays
- Coordination overhead
- Uneven traffic patterns

---

## Best Practices

### Conservative Defaults

Start with capacity = 1:
- Simplest to reason about
- Safest (no collisions)
- Increase selectively

### Match Physical Layout

```
3m aisle → capacity = 1
6m aisle → capacity = 2
```

### Test Changes

Small capacity changes can have large effects on congestion patterns.

---

## Related

- [Nodes & Edges](../warehouse/nodes-edges.md)
- [Congestion](congestion.md)
- [Traffic Configuration](../../configuration/traffic.md)
