# Custom Maps

Design warehouse layouts for your simulations.

---

## Goal

By the end of this tutorial, you will:

- Design warehouse layouts from scratch
- Use the map generator for common patterns
- Handle complex layouts with zones
- Validate map connectivity

**Time**: 45 minutes

---

## Prerequisites

- Completed [Creating Scenarios](../basic/creating-scenarios.md)
- Understanding of nodes and edges

---

## Step 1: Generate a Grid Map

Use the generator for a basic grid:

```bash
waremax generate map --type grid --rows 5 --cols 5 --spacing 3.0 > my_map.yaml
```

**Output:**

```yaml
map:
  nodes:
    - { id: 0, name: "N0", x: 0.0, y: 0.0, type: "aisle" }
    - { id: 1, name: "N1", x: 3.0, y: 0.0, type: "aisle" }
    - { id: 2, name: "N2", x: 6.0, y: 0.0, type: "aisle" }
    # ... 25 nodes total

  edges:
    - { from: 0, to: 1, bidirectional: true }
    - { from: 1, to: 2, bidirectional: true }
    # ... all edges
```

---

## Step 2: Understand Node Types

Assign meaningful types to nodes:

```yaml
map:
  nodes:
    # Aisles - regular traversable areas
    - { id: 0, name: "A1", x: 0, y: 0, type: "aisle" }

    # Racks - storage locations
    - { id: 1, name: "R1", x: 3, y: 0, type: "rack" }

    # Pick stations
    - { id: 2, name: "S1", x: 6, y: 0, type: "station_pick" }

    # Charging stations
    - { id: 3, name: "C1", x: 0, y: 6, type: "charging" }

    # Maintenance
    - { id: 4, name: "M1", x: 6, y: 6, type: "maintenance" }
```

---

## Step 3: Design a Simple Layout

Create a small warehouse manually:

```yaml
# Small warehouse: 4 racks, 1 station
map:
  nodes:
    # Main aisle (horizontal)
    - { id: 0, name: "A1", x: 0, y: 3, type: "aisle" }
    - { id: 1, name: "A2", x: 3, y: 3, type: "aisle" }
    - { id: 2, name: "A3", x: 6, y: 3, type: "aisle" }
    - { id: 3, name: "A4", x: 9, y: 3, type: "aisle" }

    # Top row racks
    - { id: 4, name: "R1", x: 1.5, y: 5, type: "rack" }
    - { id: 5, name: "R2", x: 4.5, y: 5, type: "rack" }

    # Bottom row racks
    - { id: 6, name: "R3", x: 1.5, y: 1, type: "rack" }
    - { id: 7, name: "R4", x: 4.5, y: 1, type: "rack" }

    # Pick station
    - { id: 8, name: "S1", x: 12, y: 3, type: "station_pick" }

  edges:
    # Main aisle
    - { from: 0, to: 1, bidirectional: true }
    - { from: 1, to: 2, bidirectional: true }
    - { from: 2, to: 3, bidirectional: true }
    - { from: 3, to: 8, bidirectional: true }

    # Top rack access
    - { from: 0, to: 4, bidirectional: true }
    - { from: 1, to: 4, bidirectional: true }
    - { from: 1, to: 5, bidirectional: true }
    - { from: 2, to: 5, bidirectional: true }

    # Bottom rack access
    - { from: 0, to: 6, bidirectional: true }
    - { from: 1, to: 6, bidirectional: true }
    - { from: 1, to: 7, bidirectional: true }
    - { from: 2, to: 7, bidirectional: true }
```

Visual representation:

```
    R1(4)──────R2(5)
      │╲      ╱│
      │ ╲    ╱ │
A1(0)─A2(1)─A3(2)─A4(3)───S1(8)
      │ ╱    ╲ │
      │╱      ╲│
    R3(6)──────R4(7)
```

---

## Step 4: Add One-Way Aisles

Create directional flow:

```yaml
map:
  nodes:
    # Two parallel aisles
    - { id: 0, name: "A1", x: 0, y: 0, type: "aisle" }
    - { id: 1, name: "A2", x: 3, y: 0, type: "aisle" }
    - { id: 2, name: "A3", x: 6, y: 0, type: "aisle" }

    - { id: 3, name: "B1", x: 0, y: 3, type: "aisle" }
    - { id: 4, name: "B2", x: 3, y: 3, type: "aisle" }
    - { id: 5, name: "B3", x: 6, y: 3, type: "aisle" }

  edges:
    # Top aisle: left to right only
    - { from: 0, to: 1, bidirectional: false }
    - { from: 1, to: 2, bidirectional: false }

    # Bottom aisle: right to left only
    - { from: 5, to: 4, bidirectional: false }
    - { from: 4, to: 3, bidirectional: false }

    # Connections (bidirectional)
    - { from: 0, to: 3, bidirectional: true }
    - { from: 2, to: 5, bidirectional: true }
```

```
A1 ──→ A2 ──→ A3
│              │
↕              ↕
│              │
B1 ←── B2 ←── B3
```

---

## Step 5: Create Zones

Organize into logical zones:

```yaml
# Zone-based warehouse
map:
  nodes:
    # Zone A - Fast movers (near station)
    - { id: 0, name: "ZA1", x: 0, y: 0, type: "aisle" }
    - { id: 1, name: "ZA_R1", x: 0, y: 2, type: "rack" }
    - { id: 2, name: "ZA_R2", x: 0, y: 4, type: "rack" }

    # Zone B - Slow movers (far from station)
    - { id: 10, name: "ZB1", x: 10, y: 0, type: "aisle" }
    - { id: 11, name: "ZB_R1", x: 10, y: 2, type: "rack" }
    - { id: 12, name: "ZB_R2", x: 10, y: 4, type: "rack" }

    # Main corridor connecting zones
    - { id: 20, name: "C1", x: 3, y: 0, type: "aisle" }
    - { id: 21, name: "C2", x: 6, y: 0, type: "aisle" }

    # Station
    - { id: 30, name: "S1", x: -3, y: 0, type: "station_pick" }

  edges:
    # Zone A internal
    - { from: 0, to: 1, bidirectional: true }
    - { from: 1, to: 2, bidirectional: true }

    # Zone B internal
    - { from: 10, to: 11, bidirectional: true }
    - { from: 11, to: 12, bidirectional: true }

    # Corridor
    - { from: 0, to: 20, bidirectional: true }
    - { from: 20, to: 21, bidirectional: true }
    - { from: 21, to: 10, bidirectional: true }

    # Station access
    - { from: 0, to: 30, bidirectional: true }
```

---

## Step 6: Validate Connectivity

Ensure all nodes are reachable:

```bash
waremax validate my_scenario.yaml --check connectivity
```

```
✓ Map connectivity: OK
  - All 15 nodes reachable
  - No isolated components
```

If issues:

```
✗ Map connectivity: FAILED
  - Isolated nodes: [12, 13]
  - These nodes cannot be reached from the main network
```

---

## Step 7: Use Capacity

Set node and edge capacity for wide areas:

```yaml
map:
  nodes:
    # Regular node (default capacity 1)
    - { id: 0, name: "A1", x: 0, y: 0, type: "aisle" }

    # Wide intersection (multiple robots)
    - { id: 1, name: "I1", x: 3, y: 0, type: "aisle", capacity: 4 }

    # Station approach area
    - { id: 2, name: "SA", x: 6, y: 0, type: "aisle", capacity: 3 }

  edges:
    # Regular edge
    - { from: 0, to: 1, bidirectional: true }

    # Wide aisle (2 lanes)
    - { from: 1, to: 2, bidirectional: true, capacity: 2 }
```

---

## Step 8: Generate Complex Layouts

Use generator options:

```bash
# Grid with racks
waremax generate map \
  --type grid-with-racks \
  --rows 10 \
  --cols 5 \
  --spacing 3.0 \
  --rack-rows 8 \
  > warehouse.yaml
```

```bash
# Add stations
waremax generate map \
  --type grid \
  --rows 5 --cols 5 \
  --stations 2 \
  --station-positions "right" \
  > with_stations.yaml
```

---

## Complete Example

```yaml
# Complete warehouse map
map:
  nodes:
    # Main corridor (wide)
    - { id: 0, name: "C0", x: 0, y: 5, type: "aisle", capacity: 2 }
    - { id: 1, name: "C1", x: 5, y: 5, type: "aisle", capacity: 2 }
    - { id: 2, name: "C2", x: 10, y: 5, type: "aisle", capacity: 2 }
    - { id: 3, name: "C3", x: 15, y: 5, type: "aisle", capacity: 2 }

    # Rack aisles (top)
    - { id: 10, name: "R1", x: 2, y: 8, type: "rack" }
    - { id: 11, name: "R2", x: 5, y: 8, type: "rack" }
    - { id: 12, name: "R3", x: 8, y: 8, type: "rack" }

    # Rack aisles (bottom)
    - { id: 20, name: "R4", x: 2, y: 2, type: "rack" }
    - { id: 21, name: "R5", x: 5, y: 2, type: "rack" }
    - { id: 22, name: "R6", x: 8, y: 2, type: "rack" }

    # Stations
    - { id: 30, name: "S1", x: 18, y: 7, type: "station_pick" }
    - { id: 31, name: "S2", x: 18, y: 3, type: "station_pick" }

    # Charging
    - { id: 40, name: "CH1", x: -3, y: 5, type: "charging" }

  edges:
    # Main corridor
    - { from: 0, to: 1, bidirectional: true, capacity: 2 }
    - { from: 1, to: 2, bidirectional: true, capacity: 2 }
    - { from: 2, to: 3, bidirectional: true, capacity: 2 }

    # Top rack access
    - { from: 0, to: 10, bidirectional: true }
    - { from: 1, to: 11, bidirectional: true }
    - { from: 2, to: 12, bidirectional: true }

    # Bottom rack access
    - { from: 0, to: 20, bidirectional: true }
    - { from: 1, to: 21, bidirectional: true }
    - { from: 2, to: 22, bidirectional: true }

    # Station access
    - { from: 3, to: 30, bidirectional: true }
    - { from: 3, to: 31, bidirectional: true }

    # Charging access
    - { from: 0, to: 40, bidirectional: true }
```

---

## Best Practices

### Node Placement

- Place nodes at decision points
- Include nodes at rack faces
- Add intersection nodes

### Edge Design

- Use actual distances
- Consider turning requirements
- Plan for traffic flow

### Capacity Planning

- Higher capacity at intersections
- Match to physical aisle widths
- Test with expected traffic

---

## Next Steps

- [Tuning Policies](tuning-policies.md): Optimize behavior
- [Map Configuration](../../user-guide/map-configuration.md): Full reference
