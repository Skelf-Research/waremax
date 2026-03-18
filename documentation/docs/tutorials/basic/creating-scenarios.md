# Creating Scenarios

Build custom scenario files from scratch.

---

## Goal

By the end of this tutorial, you will:

- Understand scenario file structure
- Create a minimal scenario
- Add robots, stations, and orders
- Validate and run your scenario

**Time**: 30-45 minutes

---

## Prerequisites

- Completed [Your First Simulation](first-simulation.md)
- Text editor for YAML

---

## Step 1: Minimal Scenario

Create `my_scenario.yaml`:

```yaml
# Minimal Waremax scenario
simulation:
  duration_s: 300  # 5 minutes

map:
  nodes:
    - { id: 0, name: "N0", x: 0, y: 0, type: "aisle" }
    - { id: 1, name: "N1", x: 3, y: 0, type: "aisle" }
    - { id: 2, name: "S1", x: 6, y: 0, type: "station_pick" }

  edges:
    - { from: 0, to: 1, bidirectional: true }
    - { from: 1, to: 2, bidirectional: true }

robots:
  count: 1
  speed_m_s: 1.5
  initial_positions: [0]

stations:
  - id: "S1"
    node: 2
    type: pick
    concurrency: 1

orders:
  generation:
    type: constant
    rate_per_hour: 60
```

---

## Step 2: Validate the Scenario

Check for errors before running:

```bash
waremax validate my_scenario.yaml
```

**Success:**
```
✓ my_scenario.yaml is valid
  - 3 nodes
  - 2 edges
  - 1 robot
  - 1 station
```

**If errors:**
```
✗ my_scenario.yaml has errors:
  Line 15: Unknown field 'spead_m_s' (did you mean 'speed_m_s'?)
```

---

## Step 3: Run the Scenario

```bash
waremax run my_scenario.yaml
```

---

## Step 4: Expand the Map

Add more nodes for a grid layout:

```yaml
map:
  nodes:
    # Row 1
    - { id: 0, name: "N0", x: 0, y: 0, type: "aisle" }
    - { id: 1, name: "N1", x: 3, y: 0, type: "aisle" }
    - { id: 2, name: "N2", x: 6, y: 0, type: "aisle" }

    # Row 2
    - { id: 3, name: "N3", x: 0, y: 3, type: "aisle" }
    - { id: 4, name: "N4", x: 3, y: 3, type: "aisle" }
    - { id: 5, name: "N5", x: 6, y: 3, type: "aisle" }

    # Stations
    - { id: 6, name: "S1", x: 9, y: 0, type: "station_pick" }
    - { id: 7, name: "S2", x: 9, y: 3, type: "station_pick" }

  edges:
    # Horizontal
    - { from: 0, to: 1, bidirectional: true }
    - { from: 1, to: 2, bidirectional: true }
    - { from: 2, to: 6, bidirectional: true }
    - { from: 3, to: 4, bidirectional: true }
    - { from: 4, to: 5, bidirectional: true }
    - { from: 5, to: 7, bidirectional: true }

    # Vertical
    - { from: 0, to: 3, bidirectional: true }
    - { from: 1, to: 4, bidirectional: true }
    - { from: 2, to: 5, bidirectional: true }
    - { from: 6, to: 7, bidirectional: true }
```

---

## Step 5: Add Robots

Configure multiple robots:

```yaml
robots:
  count: 5
  speed_m_s: 1.5
  initial_positions: [0, 1, 3, 4, 5]  # Spread across map
```

Or let the system place them randomly:

```yaml
robots:
  count: 5
  speed_m_s: 1.5
  # initial_positions omitted = random placement
```

---

## Step 6: Configure Stations

Add station details:

```yaml
stations:
  - id: "S1"
    node: 6
    type: pick
    concurrency: 2
    queue_capacity: 10
    service_time_s:
      distribution: constant
      base: 5.0

  - id: "S2"
    node: 7
    type: pick
    concurrency: 1
    service_time_s:
      distribution: lognormal
      base: 6.0
      base_stddev: 1.5
```

---

## Step 7: Add Storage

Add racks and inventory:

```yaml
storage:
  racks:
    - id: "R1"
      node: 1
      levels: 3
      bins_per_level: 10

    - id: "R2"
      node: 4
      levels: 3
      bins_per_level: 10

  placements:
    - rack: "R1"
      level: 1
      bin: 1
      sku: "SKU001"
      quantity: 100

    - rack: "R1"
      level: 2
      bin: 1
      sku: "SKU002"
      quantity: 100

    - rack: "R2"
      level: 1
      bin: 1
      sku: "SKU003"
      quantity: 100
```

---

## Step 8: Configure Orders

Set up order generation:

```yaml
orders:
  generation:
    type: poisson
    rate_per_hour: 200

  items_per_order:
    distribution: uniform
    min: 1
    max: 5

  sku_popularity:
    type: zipf
    alpha: 1.0
```

---

## Complete Example

Here's a complete scenario:

```yaml
# Complete warehouse scenario

simulation:
  duration_s: 3600  # 1 hour
  seed: 12345       # Reproducible

# Warehouse layout
map:
  nodes:
    - { id: 0, name: "N0", x: 0, y: 0, type: "aisle" }
    - { id: 1, name: "N1", x: 3, y: 0, type: "rack" }
    - { id: 2, name: "N2", x: 6, y: 0, type: "aisle" }
    - { id: 3, name: "N3", x: 0, y: 3, type: "aisle" }
    - { id: 4, name: "N4", x: 3, y: 3, type: "rack" }
    - { id: 5, name: "N5", x: 6, y: 3, type: "aisle" }
    - { id: 6, name: "S1", x: 9, y: 1.5, type: "station_pick" }

  edges:
    - { from: 0, to: 1, bidirectional: true }
    - { from: 1, to: 2, bidirectional: true }
    - { from: 2, to: 6, bidirectional: true }
    - { from: 3, to: 4, bidirectional: true }
    - { from: 4, to: 5, bidirectional: true }
    - { from: 5, to: 6, bidirectional: true }
    - { from: 0, to: 3, bidirectional: true }
    - { from: 1, to: 4, bidirectional: true }
    - { from: 2, to: 5, bidirectional: true }

# Fleet
robots:
  count: 3
  speed_m_s: 1.5

# Stations
stations:
  - id: "S1"
    node: 6
    type: pick
    concurrency: 2
    service_time_s:
      distribution: constant
      base: 5.0

# Storage
storage:
  racks:
    - { id: "R1", node: 1, levels: 3, bins_per_level: 10 }
    - { id: "R2", node: 4, levels: 3, bins_per_level: 10 }

# Orders
orders:
  generation:
    type: constant
    rate_per_hour: 100

# Policies
policies:
  task_allocation: nearest_idle
  station_assignment: shortest_queue
```

---

## Validation Checklist

Before running, verify:

- [ ] All nodes have unique IDs
- [ ] Edges connect existing nodes
- [ ] Stations reference valid nodes
- [ ] Robots ≤ nodes (for initial positions)
- [ ] Storage racks at rack-type nodes

---

## Next Steps

- [Analyzing Results](analyzing-results.md): Understand output
- [Custom Maps](../config/custom-maps.md): Complex layouts
- [Tuning Policies](../config/tuning-policies.md): Optimize behavior
