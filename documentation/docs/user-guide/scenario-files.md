# Scenario Files

Scenario files define the complete configuration for a warehouse simulation.

---

## Overview

A scenario file is a YAML document containing:

- Simulation parameters
- Map and storage references
- Robot fleet configuration
- Station definitions
- Order generation settings
- Policy configurations
- Traffic and routing settings
- Optional: battery, maintenance, metrics settings

---

## File Structure

```yaml
# Simulation seed for reproducibility
seed: 42

# Simulation timing
simulation:
  duration_minutes: 60
  warmup_minutes: 10

# External file references
map:
  file: map.json
storage:
  file: storage.yaml

# Robot fleet
robots:
  count: 10
  max_speed_mps: 1.5
  max_payload_kg: 25

# Stations
stations:
  - id: "S1"
    node: "0"
    type: pick
    concurrency: 2
    service_time_s:
      base: 5.0
      per_item: 2.0

# Order generation
orders:
  arrival_process:
    type: poisson
    rate_per_min: 1.0
  lines_per_order:
    type: negbinomial
    mean: 3.0
  sku_popularity:
    type: zipf
    alpha: 1.0

# Dispatching policies
policies:
  task_allocation:
    type: nearest_robot
  station_assignment:
    type: least_queue

# Traffic management
traffic:
  policy: wait_at_node
  edge_capacity_default: 1

# Routing
routing:
  algorithm: dijkstra
  cache_routes: true

# Metrics collection
metrics:
  sample_interval_s: 60
```

---

## Required Sections

### seed

Random seed for reproducible simulations:

```yaml
seed: 42
```

- Must be a positive integer
- Same seed + same config = same results

### simulation

Timing parameters:

```yaml
simulation:
  duration_minutes: 60    # Total simulation time
  warmup_minutes: 10      # Warmup period (metrics excluded)
  time_unit: seconds      # Optional: time unit (default: seconds)
```

### map

Reference to map topology file:

```yaml
map:
  file: warehouse_map.json
```

### storage

Reference to storage configuration:

```yaml
storage:
  file: inventory.yaml
```

### robots

Robot fleet configuration:

```yaml
robots:
  count: 10               # Number of robots
  max_speed_mps: 1.5      # Speed in m/s
  max_payload_kg: 25      # Payload capacity
```

### stations

List of station definitions:

```yaml
stations:
  - id: "S1"
    node: "0"
    type: pick
    concurrency: 2
    queue_capacity: 10
    service_time_s:
      distribution: constant
      base: 5.0
      per_item: 2.0
```

### orders

Order generation configuration:

```yaml
orders:
  arrival_process:
    type: poisson
    rate_per_min: 1.0
  lines_per_order:
    type: negbinomial
    mean: 3.0
    dispersion: 1.0
  sku_popularity:
    type: zipf
    alpha: 1.0
```

---

## Optional Sections

### policies

Dispatching policy configuration:

```yaml
policies:
  task_allocation:
    type: nearest_robot
  station_assignment:
    type: least_queue
  batching:
    type: none
  priority:
    type: strict_priority
```

### traffic

Traffic management settings:

```yaml
traffic:
  policy: wait_at_node
  edge_capacity_default: 1
  node_capacity_default: 1
  wait_threshold_s: 2.0
  deadlock_detection: false
```

### routing

Routing algorithm configuration:

```yaml
routing:
  algorithm: dijkstra
  congestion_aware: false
  cache_routes: true
  congestion_weight: 0.5
```

### metrics

Metrics collection settings:

```yaml
metrics:
  sample_interval_s: 60
  per_robot_breakdown: false
  per_station_breakdown: false
  generate_heatmap: false
  trace:
    enabled: false
    max_entries: 10000
```

---

## Battery Configuration

Enable battery simulation:

```yaml
robots:
  count: 10
  max_speed_mps: 1.5
  max_payload_kg: 25
  battery:
    enabled: true
    capacity_wh: 400
    min_soc: 0.15
    consumption:
      per_meter_wh: 0.1
      per_kg_per_meter_wh: 0.01
      idle_power_w: 5.0
      service_power_w: 20.0

charging_stations:
  - id: "C1"
    node: "10"
    bays: 2
    charge_rate_w: 200
    queue_capacity: 5
```

---

## Maintenance Configuration

Enable maintenance and failures:

```yaml
robots:
  count: 10
  max_speed_mps: 1.5
  max_payload_kg: 25
  maintenance:
    enabled: true
    interval_hours: 8.0
  failure:
    enabled: true
    mtbf_hours: 100.0

maintenance_stations:
  - id: "M1"
    node: "15"
    bays: 2
    maintenance_duration_s: 300
    repair_time:
      distribution: lognormal
      base: 600
      base_stddev: 120
```

---

## Inbound/Replenishment

Configure inbound and replenishment flows:

```yaml
inbound:
  arrival_process:
    type: poisson
    rate_per_min: 0.1
  items_per_shipment: 50

replenishment:
  enabled: true
  default_threshold: 10
  sku_thresholds:
    SKU001: 20
    SKU002: 15
```

---

## Due Times

Configure order due times:

```yaml
orders:
  # ... arrival and lines config ...
  due_times:
    type: fixed_offset
    minutes: 60
```

---

## Service Time Distributions

Stations support multiple service time distributions:

### Constant

```yaml
service_time_s:
  distribution: constant
  base: 5.0
  per_item: 2.0
```

### Lognormal

```yaml
service_time_s:
  distribution: lognormal
  base: 8.0
  base_stddev: 2.0
  per_item: 2.0
  per_item_stddev: 0.5
```

### Exponential

```yaml
service_time_s:
  distribution: exponential
  base: 10.0
```

### Uniform

```yaml
service_time_s:
  distribution: uniform
  min_s: 3.0
  max_s: 10.0
  per_item: 1.5
```

---

## File References

### Relative Paths

File paths are relative to the scenario file:

```yaml
# If scenario is at /project/scenarios/test.yaml
# This refers to /project/scenarios/map.json
map:
  file: map.json
```

### Absolute Paths

Absolute paths are also supported:

```yaml
map:
  file: /data/maps/warehouse.json
```

---

## Comments

YAML supports comments:

```yaml
# This is a comment
seed: 42  # Inline comment

# Simulation settings
simulation:
  duration_minutes: 60  # 1 hour simulation
```

---

## Validation

Validate a scenario before running:

```bash
waremax validate --scenario my_scenario.yaml
```

Common validation checks:

- Required fields present
- Valid field types
- Reasonable parameter ranges
- Station nodes exist in map
- Policies are valid types

---

## Best Practices

1. **Use descriptive station IDs**: `"pick_station_1"` not `"S1"`
2. **Include comments**: Document non-obvious settings
3. **Version control scenarios**: Track changes over time
4. **Use meaningful seeds**: Document what each seed represents
5. **Organize related files**: Keep map, storage, and scenario together

---

## Example: Complete Scenario

```yaml
# Production warehouse simulation
seed: 12345

simulation:
  duration_minutes: 480    # 8-hour shift
  warmup_minutes: 60       # 1 hour warmup

map:
  file: warehouse_layout.json
storage:
  file: current_inventory.yaml

robots:
  count: 25
  max_speed_mps: 1.8
  max_payload_kg: 30
  battery:
    enabled: true
    capacity_wh: 500
    min_soc: 0.20
    consumption:
      per_meter_wh: 0.08
      idle_power_w: 3.0
      service_power_w: 15.0

stations:
  - id: "pick_zone_a"
    node: "N100"
    type: pick
    concurrency: 3
    queue_capacity: 15
    service_time_s:
      distribution: lognormal
      base: 6.0
      base_stddev: 1.5
      per_item: 1.8
      per_item_stddev: 0.3

  - id: "pick_zone_b"
    node: "N200"
    type: pick
    concurrency: 3
    queue_capacity: 15
    service_time_s:
      distribution: lognormal
      base: 6.0
      base_stddev: 1.5
      per_item: 1.8
      per_item_stddev: 0.3

charging_stations:
  - id: "charger_1"
    node: "N50"
    bays: 4
    charge_rate_w: 250
    queue_capacity: 8

orders:
  arrival_process:
    type: poisson
    rate_per_min: 2.5      # 150 orders/hour
  lines_per_order:
    type: negbinomial
    mean: 4.0
    dispersion: 1.2
  sku_popularity:
    type: zipf
    alpha: 1.1
  due_times:
    type: fixed_offset
    minutes: 45

policies:
  task_allocation:
    type: auction
    travel_weight: 1.0
    queue_weight: 0.5
  station_assignment:
    type: due_time_priority
  batching:
    type: station_batch
    max_items: 20
  priority:
    type: sla_driven

traffic:
  policy: reroute_on_wait
  edge_capacity_default: 2
  node_capacity_default: 2
  wait_threshold_s: 3.0
  max_reroute_attempts: 3
  deadlock_detection: true
  deadlock_resolver: youngest_backs_up

routing:
  algorithm: astar
  congestion_aware: true
  congestion_weight: 0.6
  cache_routes: true

metrics:
  sample_interval_s: 60
  per_robot_breakdown: true
  per_station_breakdown: true
  generate_heatmap: true
```

---

## Next Steps

- **[Configuration Reference](../configuration/index.md)** - Complete parameter documentation
- **[Map Configuration](map-configuration.md)** - Creating map files
- **[Storage Configuration](storage-configuration.md)** - Setting up inventory
