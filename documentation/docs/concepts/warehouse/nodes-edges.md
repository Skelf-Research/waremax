# Nodes & Edges

Physical locations and paths in the warehouse.

---

## Nodes

### Definition

A node represents a physical location where a robot can be.

### Node Types

| Type | Description |
|------|-------------|
| `aisle` | Regular traversable location |
| `rack` | Storage rack location |
| `station_pick` | Pick station |
| `station_drop` | Drop station |
| `station_inbound` | Inbound receiving |
| `station_outbound` | Outbound shipping |
| `charging` | Charging station |
| `maintenance` | Maintenance station |

### Node Properties

| Property | Description |
|----------|-------------|
| `id` | Unique identifier |
| `name` | Human-readable name |
| `x`, `y` | Coordinates (meters) |
| `type` | Node classification |

### Node Capacity

Default: 1 robot per node

Can be configured higher for:

- Intersections
- Station areas
- Waiting zones

---

## Edges

### Definition

An edge connects two nodes, representing a path robots can traverse.

### Edge Properties

| Property | Description |
|----------|-------------|
| `id` | Unique identifier |
| `from` | Source node ID |
| `to` | Destination node ID |
| `length` | Distance in meters |
| `bidirectional` | Both directions allowed? |
| `capacity` | Max robots simultaneously |

### Edge Length

Physical distance between nodes:

```
length = sqrt((x2-x1)² + (y2-y1)²)
```

Or specified explicitly for non-straight paths.

### Edge Capacity

Default: 1 robot per edge

Higher capacity for:

- Wide aisles
- Multi-lane paths
- High-traffic routes

---

## Bidirectional Edges

### Default (Bidirectional)

```json
{
  "from": 0,
  "to": 1,
  "bidirectional": true
}
```

Equivalent to two one-way edges.

### One-Way Edges

```json
{
  "from": 0,
  "to": 1,
  "bidirectional": false
}
```

Only allows travel from 0 to 1.

### Use Cases for One-Way

- Traffic flow control
- Aisle direction policies
- Reduced congestion

---

## Travel Time

Calculated from edge length and robot speed:

```
travel_time = edge_length / robot_speed
```

Example:

- Edge: 6 meters
- Speed: 1.5 m/s
- Time: 4 seconds

---

## Capacity and Congestion

### Node Capacity

When a node is at capacity:

- New robots must wait
- Or reroute (if configured)

### Edge Capacity

When an edge is at capacity:

- Robots wait at source node
- Release when edge clears

### Capacity Configuration

```yaml
traffic:
  node_capacity_default: 1
  edge_capacity_default: 1
```

---

## Example: Grid Layout

```
Nodes:
  N0(0,0) -- N1(3,0) -- N2(6,0)
     |          |          |
  N3(0,3) -- N4(3,3) -- N5(6,3)
     |          |          |
  N6(0,6) -- N7(3,6) -- N8(6,6)

Edges (bidirectional, length 3.0):
  N0-N1, N1-N2, N0-N3, N1-N4, N2-N5
  N3-N4, N4-N5, N3-N6, N4-N7, N5-N8
  N6-N7, N7-N8
```

---

## Best Practices

### Node Placement

- Place at decision points
- At racks and stations
- At intersections

### Edge Design

- Use actual path distances
- Consider robot turning radius
- Account for acceleration/deceleration

### Capacity Planning

- Higher capacity at bottlenecks
- Match to actual aisle widths
- Test with expected traffic

---

## Related

- [Graph-Based Maps](maps.md)
- [Traffic Configuration](../../configuration/traffic.md)
