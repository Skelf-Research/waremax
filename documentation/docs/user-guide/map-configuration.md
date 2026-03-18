# Map Configuration

Map files define the warehouse topology as a graph of nodes and edges.

---

## Overview

A warehouse map consists of:

- **Nodes** - Physical locations (aisles, racks, stations)
- **Edges** - Connections between nodes (paths robots can traverse)

Maps are stored as JSON files.

---

## File Format

```json
{
  "nodes": [
    {
      "id": 0,
      "name": "N0",
      "x": 0.0,
      "y": 0.0,
      "type": "station_pick"
    },
    {
      "id": 1,
      "name": "N1",
      "x": 3.0,
      "y": 0.0,
      "type": "aisle"
    }
  ],
  "edges": [
    {
      "id": 0,
      "from": 0,
      "to": 1,
      "length": 3.0,
      "bidirectional": true
    }
  ]
}
```

---

## Nodes

### Node Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | integer | Unique node identifier |
| `name` | string | Human-readable name |
| `x` | float | X coordinate in meters |
| `y` | float | Y coordinate in meters |
| `type` | string | Node type |

### Node Types

| Type | Description |
|------|-------------|
| `aisle` | Regular traversable location |
| `rack` | Storage rack location |
| `station_pick` | Pick station location |
| `station_drop` | Drop station location |
| `station_inbound` | Inbound station location |
| `station_outbound` | Outbound station location |
| `charging` | Charging station location |
| `maintenance` | Maintenance station location |

### Example Nodes

```json
{
  "nodes": [
    {
      "id": 0,
      "name": "PickStation1",
      "x": 0.0,
      "y": 0.0,
      "type": "station_pick"
    },
    {
      "id": 1,
      "name": "Aisle_A1",
      "x": 3.0,
      "y": 0.0,
      "type": "aisle"
    },
    {
      "id": 2,
      "name": "Rack_R1",
      "x": 3.0,
      "y": 3.0,
      "type": "rack"
    },
    {
      "id": 10,
      "name": "Charger1",
      "x": 0.0,
      "y": 10.0,
      "type": "charging"
    }
  ]
}
```

---

## Edges

### Edge Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | integer | Unique edge identifier |
| `from` | integer | Source node ID |
| `to` | integer | Destination node ID |
| `length` | float | Edge length in meters |
| `bidirectional` | boolean | Optional: if false, one-way edge (default: true) |
| `capacity` | integer | Optional: max robots on edge simultaneously |

### Example Edges

```json
{
  "edges": [
    {
      "id": 0,
      "from": 0,
      "to": 1,
      "length": 3.0
    },
    {
      "id": 1,
      "from": 1,
      "to": 2,
      "length": 3.0
    },
    {
      "id": 2,
      "from": 0,
      "to": 3,
      "length": 3.0,
      "bidirectional": false
    },
    {
      "id": 3,
      "from": 1,
      "to": 4,
      "length": 3.0,
      "capacity": 2
    }
  ]
}
```

---

## Grid Layout Example

A simple 5x5 grid warehouse:

```json
{
  "nodes": [
    {"id": 0, "name": "N0", "x": 0.0, "y": 0.0, "type": "station_pick"},
    {"id": 1, "name": "N1", "x": 3.0, "y": 0.0, "type": "aisle"},
    {"id": 2, "name": "N2", "x": 6.0, "y": 0.0, "type": "aisle"},
    {"id": 3, "name": "N3", "x": 9.0, "y": 0.0, "type": "aisle"},
    {"id": 4, "name": "N4", "x": 12.0, "y": 0.0, "type": "station_pick"},

    {"id": 5, "name": "N5", "x": 0.0, "y": 3.0, "type": "aisle"},
    {"id": 6, "name": "N6", "x": 3.0, "y": 3.0, "type": "rack"},
    {"id": 7, "name": "N7", "x": 6.0, "y": 3.0, "type": "rack"},
    {"id": 8, "name": "N8", "x": 9.0, "y": 3.0, "type": "rack"},
    {"id": 9, "name": "N9", "x": 12.0, "y": 3.0, "type": "aisle"},

    {"id": 10, "name": "N10", "x": 0.0, "y": 6.0, "type": "aisle"},
    {"id": 11, "name": "N11", "x": 3.0, "y": 6.0, "type": "rack"},
    {"id": 12, "name": "N12", "x": 6.0, "y": 6.0, "type": "rack"},
    {"id": 13, "name": "N13", "x": 9.0, "y": 6.0, "type": "rack"},
    {"id": 14, "name": "N14", "x": 12.0, "y": 6.0, "type": "aisle"},

    {"id": 15, "name": "N15", "x": 0.0, "y": 9.0, "type": "aisle"},
    {"id": 16, "name": "N16", "x": 3.0, "y": 9.0, "type": "rack"},
    {"id": 17, "name": "N17", "x": 6.0, "y": 9.0, "type": "rack"},
    {"id": 18, "name": "N18", "x": 9.0, "y": 9.0, "type": "rack"},
    {"id": 19, "name": "N19", "x": 12.0, "y": 9.0, "type": "aisle"},

    {"id": 20, "name": "N20", "x": 0.0, "y": 12.0, "type": "charging"},
    {"id": 21, "name": "N21", "x": 3.0, "y": 12.0, "type": "aisle"},
    {"id": 22, "name": "N22", "x": 6.0, "y": 12.0, "type": "aisle"},
    {"id": 23, "name": "N23", "x": 9.0, "y": 12.0, "type": "aisle"},
    {"id": 24, "name": "N24", "x": 12.0, "y": 12.0, "type": "charging"}
  ],
  "edges": [
    {"id": 0, "from": 0, "to": 1, "length": 3.0},
    {"id": 1, "from": 1, "to": 2, "length": 3.0},
    {"id": 2, "from": 2, "to": 3, "length": 3.0},
    {"id": 3, "from": 3, "to": 4, "length": 3.0},
    {"id": 4, "from": 0, "to": 5, "length": 3.0},
    {"id": 5, "from": 1, "to": 6, "length": 3.0},
    {"id": 6, "from": 2, "to": 7, "length": 3.0},
    {"id": 7, "from": 3, "to": 8, "length": 3.0},
    {"id": 8, "from": 4, "to": 9, "length": 3.0}
  ]
}
```

---

## Coordinate System

- Origin (0, 0) is typically bottom-left
- X increases to the right
- Y increases upward
- Units are meters

```
Y
^
|  [N20]---[N21]---[N22]---[N23]---[N24]
|    |       |       |       |       |
|  [N15]---[N16]---[N17]---[N18]---[N19]
|    |       |       |       |       |
|  [N10]---[N11]---[N12]---[N13]---[N14]
|    |       |       |       |       |
|  [N5 ]---[N6 ]---[N7 ]---[N8 ]---[N9 ]
|    |       |       |       |       |
|  [N0 ]---[N1 ]---[N2 ]---[N3 ]---[N4 ]
+-----------------------------------------> X
```

---

## Linking to Scenario

Reference the map file in your scenario:

```yaml
map:
  file: warehouse_map.json
```

Match station nodes to map:

```yaml
stations:
  - id: "S1"
    node: "0"       # Must match node ID in map
    type: pick
    # ...
```

---

## Default Map

If no map file is specified, Waremax generates a default grid map based on scenario parameters.

---

## Best Practices

### Node Naming

Use consistent, descriptive names:

```json
{"id": 0, "name": "PickStation_A", ...},
{"id": 1, "name": "Aisle_A1", ...},
{"id": 2, "name": "Rack_A1_L1", ...}
```

### Edge Lengths

Calculate accurate edge lengths based on physical distances:

```json
{
  "from": 0,
  "to": 1,
  "length": 3.0  // Actual distance in meters
}
```

### Connectivity

Ensure all nodes are reachable:

- Every node should have at least one edge
- Check that robots can reach all stations
- Verify charging/maintenance stations are accessible

### Capacity Planning

For high-traffic areas, consider:

- Higher edge capacity
- Multiple parallel aisles
- Sufficient node capacity at intersections

---

## Validation

Waremax validates maps during scenario validation:

```bash
waremax validate --scenario my_scenario.yaml
```

Checks include:

- All station nodes exist
- Graph is connected
- Edge lengths are positive
- Node IDs are unique

---

## Next Steps

- **[Storage Configuration](storage-configuration.md)** - Setting up racks and inventory
- **[Configuration Reference](../configuration/index.md)** - Complete parameter reference
