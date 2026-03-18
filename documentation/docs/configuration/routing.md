# Routing Configuration

Configuration for path finding and routing algorithms.

---

## Schema

```yaml
routing:
  algorithm: <string>              # Default: "dijkstra"
  congestion_aware: <boolean>     # Default: false
  cache_routes: <boolean>         # Default: true
  congestion_weight: <float>      # Default: 0.5
```

---

## Fields

### algorithm

**Type**: string
**Default**: "dijkstra"
**Options**: `dijkstra`, `astar`

Routing algorithm to use.

```yaml
routing:
  algorithm: dijkstra
```

| Algorithm | Description | Best For |
|-----------|-------------|----------|
| `dijkstra` | Shortest path | General use, guaranteed optimal |
| `astar` | Heuristic-guided | Large maps, faster for long paths |

### congestion_aware

**Type**: boolean
**Default**: false

Enable congestion-aware routing.

```yaml
routing:
  algorithm: astar
  congestion_aware: true
```

When enabled, routing considers current congestion levels when choosing paths.

### cache_routes

**Type**: boolean
**Default**: true

Cache computed routes for reuse.

```yaml
routing:
  cache_routes: true
```

**Benefits**:

- Faster repeated queries
- Reduced computation

**Disable when**:

- Congestion-aware routing (routes should adapt)
- Very dynamic maps

### congestion_weight

**Type**: float
**Default**: 0.5

Weight factor for congestion in path cost.

```yaml
routing:
  congestion_aware: true
  congestion_weight: 0.6
```

| Weight | Effect |
|--------|--------|
| 0.0 | Ignore congestion (shortest path only) |
| 0.5 | Balance distance and congestion |
| 1.0 | Heavily weight congestion avoidance |

---

## Examples

### Default Routing

```yaml
routing:
  algorithm: dijkstra
  cache_routes: true
```

### A* for Large Maps

```yaml
routing:
  algorithm: astar
  cache_routes: true
```

### Congestion-Aware

```yaml
routing:
  algorithm: astar
  congestion_aware: true
  congestion_weight: 0.6
  cache_routes: false  # Routes should adapt to congestion
```

### Maximum Congestion Avoidance

```yaml
routing:
  algorithm: astar
  congestion_aware: true
  congestion_weight: 0.8
  cache_routes: false
```

---

## Algorithm Details

### Dijkstra's Algorithm

- Finds shortest path based on edge weights
- Guaranteed optimal solution
- Explores all directions equally
- O(V log V + E) complexity

**Best for**:

- Small to medium maps
- When optimality is required
- Static routing scenarios

### A* Algorithm

- Uses heuristic to guide search toward goal
- Typically faster than Dijkstra for long paths
- Same optimality guarantee with admissible heuristic
- More efficient for large maps

**Best for**:

- Large maps
- Long-distance routes
- When performance matters

---

## Congestion-Aware Routing

### How It Works

Path cost becomes:

```
cost = base_distance + (congestion_weight × congestion_score × edge_length)
```

Where:

- `base_distance`: Physical edge length
- `congestion_score`: Current congestion level (0-1)
- `edge_length`: Length of the edge
- `congestion_weight`: User-configured weight

### Example Impact

For a path with two options:

| Route | Distance | Congestion | Effective Cost (weight=0.5) |
|-------|----------|------------|----------------------------|
| A | 100m | 0.1 | 100 + 0.5 × 0.1 × 100 = 105m |
| B | 120m | 0.0 | 120 + 0 = 120m |

Route A is chosen (shorter effective cost).

With high congestion on Route A:

| Route | Distance | Congestion | Effective Cost (weight=0.5) |
|-------|----------|------------|----------------------------|
| A | 100m | 0.8 | 100 + 0.5 × 0.8 × 100 = 140m |
| B | 120m | 0.0 | 120 + 0 = 120m |

Route B is now chosen.

---

## Performance Considerations

### Route Caching

**With caching enabled**:

- Routes computed once and reused
- Fast repeated queries
- May use suboptimal routes if conditions change

**Without caching**:

- Routes computed fresh each time
- Adapts to current conditions
- Higher computation cost

### Recommendation

| Scenario | Caching | Congestion-Aware |
|----------|---------|------------------|
| Low traffic | Yes | No |
| Medium traffic | Yes | Optional |
| High traffic | No | Yes |
| Dynamic conditions | No | Yes |

---

## Integration with Traffic

Routing works with traffic configuration:

```yaml
traffic:
  policy: reroute_on_wait
  edge_capacity_default: 2

routing:
  algorithm: astar
  congestion_aware: true
  congestion_weight: 0.6
  cache_routes: false
```

When `reroute_on_wait` is used with congestion-aware routing:

1. Robot blocked at capacity limit
2. Waits for `wait_threshold_s`
3. Reroutes using congestion-aware algorithm
4. New route avoids congested areas

---

## Related

- [Traffic Configuration](traffic.md)
- [Movement & Routing Concepts](../concepts/robots/movement.md)
