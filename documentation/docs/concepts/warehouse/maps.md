# Graph-Based Maps

How warehouse topology is represented.

---

## Graph Structure

A warehouse map is a weighted graph G = (V, E):

- **V**: Set of nodes (vertices)
- **E**: Set of edges connecting nodes
- **w**: Edge weights (lengths)

---

## Why Graphs?

### Flexibility

- Any layout can be represented
- Non-rectangular warehouses
- Multiple floors (separate graphs)

### Efficiency

- Standard graph algorithms
- Fast pathfinding
- Compact representation

### Realism

- Actual travel paths
- Accurate distances
- Physical constraints

---

## Map Components

### Nodes (Vertices)

```json
{
  "id": 0,
  "name": "N0",
  "x": 0.0,
  "y": 0.0,
  "type": "aisle"
}
```

Properties:

- Unique identifier
- Position coordinates
- Type classification

### Edges

```json
{
  "id": 0,
  "from": 0,
  "to": 1,
  "length": 3.0,
  "bidirectional": true
}
```

Properties:

- Source and destination nodes
- Physical length
- Directionality

---

## Map Topology

### Connected Graph

All nodes must be reachable:

- No isolated nodes
- No disconnected regions
- Robots can reach any location

### Bidirectional vs One-Way

**Bidirectional** (default):

- Robots can travel both directions
- Most flexible

**One-way**:

- Traffic flow control
- Reduces congestion
- More complex routing

---

## Pathfinding

### Shortest Path

Default routing finds shortest path by distance.

### A* Algorithm

Uses heuristic for faster long-distance routing:

```
f(n) = g(n) + h(n)
```

Where:

- g(n): Actual cost from start
- h(n): Estimated cost to goal (Euclidean distance)

### Congestion-Aware

Optional weighting by current congestion:

```
cost = distance + congestion_weight × congestion_score
```

---

## Example Layouts

### Simple Grid

```
5x5 grid with 3m spacing:

[N0]--[N1]--[N2]--[N3]--[N4]
 |     |     |     |     |
[N5]--[N6]--[N7]--[N8]--[N9]
 |     |     |     |     |
...
```

### Zone-Based

```
Zone A          Zone B
[A1]--[A2]  |  [B1]--[B2]
 |     |    |   |     |
[A3]--[A4]--+--[B3]--[B4]
            |
        Main Aisle
```

---

## Best Practices

### Node Density

- More nodes = finer granularity
- Fewer nodes = simpler routing
- Balance accuracy vs. complexity

### Edge Lengths

- Use actual physical distances
- Consistent units (meters)
- Account for corners/turns

### Capacity Planning

- Identify bottleneck paths
- Add parallel routes if needed
- Consider traffic flow

---

## Related

- [Nodes & Edges](nodes-edges.md)
- [Map Configuration](../../user-guide/map-configuration.md)
