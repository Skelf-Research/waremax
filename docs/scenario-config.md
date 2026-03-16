# Scenario Configuration

Waremax scenarios are defined using a single top-level file (YAML or JSON) that references supporting files. The structure is intentionally simple to keep scenarios editable by hand.

## File Structure

- `scenario.yaml` (or `scenario.json`)
- `map.json` (graph representation)
- `storage.yaml` (racks, bins, and placements)
- Optional: `skus.yaml` if you want to separate item definitions

## scenario.yaml

```yaml
seed: 42
simulation:
  duration_minutes: 480
  warmup_minutes: 30
  time_unit: seconds

map:
  file: "map.json"

storage:
  file: "storage.yaml"

robots:
  count: 30
  max_speed_mps: 1.6
  max_payload_kg: 25
  battery:
    enabled: false
    capacity_wh: 400
    min_soc: 0.15
    charge_rate_w: 200

stations:
  - id: P1
    node: STN_P1
    type: pick
    concurrency: 2
    queue_capacity: 20
    service_time_s:
      base: 12
      per_item: 3
  - id: D1
    node: STN_D1
    type: drop
    concurrency: 1
    queue_capacity: 10
    service_time_s:
      base: 8
      per_item: 2
  - id: IN1
    node: STN_IN1
    type: inbound
    concurrency: 2
    queue_capacity: 30
    service_time_s:
      base: 15
      per_item: 4
  - id: OUT1
    node: STN_OUT1
    type: outbound
    concurrency: 2
    queue_capacity: 25
    service_time_s:
      base: 10
      per_item: 3

orders:
  arrival_process:
    type: poisson
    rate_per_min: 4.0
  lines_per_order:
    type: negbin
    mean: 2.2
    dispersion: 1.3
  sku_popularity:
    type: zipf
    alpha: 1.05
  due_times:
    type: fixed_offset
    minutes: 60

putaway:
  enabled: false
  arrival_process:
    type: poisson
    rate_per_min: 0.5
  source_nodes: [N1, N2]
  destination_policy:
    type: nearest_empty_bin
  priority_class:
    type: mixed
    p_high: 0.2

policies:
  task_allocation:
    type: nearest_robot
  station_assignment:
    type: least_queue
  batching:
    type: station_batch
    max_items: 10
  priority:
    type: weighted_fair
    weights:
      pick: 0.7
      replen: 0.2
      putaway: 0.1

traffic:
  policy: reroute_on_wait    # Options: wait_at_node, reroute_on_wait, reservation
  edge_capacity_default: 1
  node_capacity_default: 1
  reroute_after_s: 20        # Used when policy is reroute_on_wait
  reservation_horizon_s: 5   # Used when policy is reservation

routing:
  algorithm: dijkstra        # Options: dijkstra, astar
  congestion_aware: false    # Add congestion penalty to edge costs
  congestion_weight: 0.5     # Weight for congestion penalty (0.0-1.0)
  cache_routes: true         # Cache routes between frequently used nodes
```

Notes:
- `priority_class` assigns a priority label to putaway jobs; arbitration between job types is controlled by `policies.priority`.
- Config keys use abbreviated job type names: `replen` for replenishment, `putaway` for putaway operations.

### Alternative Service Time Formats

Service times can be specified in two ways:

**Linear model (base + per_item):**
```yaml
service_time_s:
  base: 12
  per_item: 3
```

**Distribution model:**
```yaml
service_time_s:
  type: distribution
  distribution: lognormal  # Options: constant, uniform, normal, lognormal, exponential
  mean: 15.0
  stddev: 3.0
```

### Queue Capacity

Queue capacity can be finite or unbounded:
```yaml
queue_capacity: 20      # Finite capacity of 20
queue_capacity: null    # Unbounded (no limit)
```

If `queue_capacity` is omitted, it defaults to unbounded.

## map.json

```json
{
  "nodes": [
    {"id": "N1", "x": 0, "y": 0, "type": "aisle"},
    {"id": "N2", "x": 2, "y": 0, "type": "aisle"},
    {"id": "N3", "x": 4, "y": 0, "type": "aisle"},
    {"id": "STN_P1", "x": 10, "y": 5, "type": "station_pick"},
    {"id": "STN_D1", "x": 10, "y": -5, "type": "station_drop"},
    {"id": "STN_IN1", "x": -5, "y": 0, "type": "station_inbound"},
    {"id": "STN_OUT1", "x": 15, "y": 0, "type": "station_outbound"}
  ],
  "edges": [
    {"from": "N1", "to": "N2", "length_m": 2.0, "bidirectional": true, "capacity": 1},
    {"from": "N2", "to": "N3", "length_m": 2.0, "bidirectional": true, "capacity": 1},
    {"from": "N2", "to": "STN_P1", "length_m": 8.0, "bidirectional": true, "capacity": 1},
    {"from": "N2", "to": "STN_D1", "length_m": 8.0, "bidirectional": true, "capacity": 1},
    {"from": "N1", "to": "STN_IN1", "length_m": 5.0, "bidirectional": true, "capacity": 1},
    {"from": "N3", "to": "STN_OUT1", "length_m": 11.0, "bidirectional": true, "capacity": 1}
  ],
  "constraints": {
    "blocked_nodes": [],
    "blocked_edges": []
  }
}
```

## storage.yaml

```yaml
racks:
  - id: R1
    access_node: N1
    levels: 4
    bins_per_level: 20
    zone: A
  - id: R2
    access_node: N2
    levels: 4
    bins_per_level: 20
    zone: B

placements:
  sku_001:
    - { rack: R1, level: 0, bin: 1, qty: 30 }
    - { rack: R2, level: 1, bin: 2, qty: 10 }

skus:
  - id: sku_001
    unit_pick_time_s: 4
```

## Validation Rules (Recommended)

- All referenced node IDs must exist in the map.
- Edge lengths must be positive and non-zero.
- All placement bins must be within rack bounds.
- If `max_payload_kg` is set, batching must not exceed it.
- Station service times must be non-negative.
- If battery is disabled, all battery fields are ignored.
