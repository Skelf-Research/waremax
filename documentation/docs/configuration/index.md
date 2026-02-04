# Configuration Reference

Complete reference for all Waremax configuration options.

---

## Overview

Waremax configurations are YAML files with the following top-level sections:

| Section | Required | Description |
|---------|----------|-------------|
| `seed` | Yes | Random seed for reproducibility |
| `simulation` | Yes | Timing parameters |
| `map` | Yes | Map file reference |
| `storage` | Yes | Storage file reference |
| `robots` | Yes | Robot fleet configuration |
| `stations` | Yes | Station definitions |
| `orders` | Yes | Order generation settings |
| `policies` | No | Dispatching policies |
| `traffic` | No | Traffic management |
| `routing` | No | Routing algorithm |
| `charging_stations` | No | Charging stations |
| `maintenance_stations` | No | Maintenance stations |
| `inbound` | No | Inbound flow configuration |
| `replenishment` | No | Replenishment settings |
| `metrics` | No | Metrics collection |

---

## Configuration Sections

| Reference | Description |
|-----------|-------------|
| [Scenario Schema](scenario-schema.md) | Complete schema overview |
| [Simulation](simulation.md) | Timing and duration settings |
| [Robots](robots.md) | Robot fleet configuration |
| [Stations](stations.md) | Station definitions |
| [Orders](orders.md) | Order generation |
| [Policies](policies.md) | Dispatching policies |
| [Traffic](traffic.md) | Traffic management |
| [Routing](routing.md) | Routing algorithms |
| [Battery](battery.md) | Battery and charging |
| [Maintenance](maintenance.md) | Maintenance and failures |
| [Metrics](metrics.md) | Metrics collection |

---

## Quick Reference

### Minimal Configuration

```yaml
seed: 42
simulation:
  duration_minutes: 30
map:
  file: map.json
storage:
  file: storage.yaml
robots:
  count: 10
  max_speed_mps: 1.5
stations:
  - id: "S1"
    node: "0"
    type: pick
    service_time_s:
      base: 5.0
orders:
  arrival_process:
    type: poisson
    rate_per_min: 1.0
  lines_per_order:
    type: negbinomial
    mean: 3.0
  sku_popularity:
    type: zipf
```

### Full Configuration

```yaml
seed: 42

simulation:
  duration_minutes: 60
  warmup_minutes: 10
  time_unit: seconds

map:
  file: warehouse.json

storage:
  file: inventory.yaml

robots:
  count: 20
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
  maintenance:
    enabled: true
    interval_hours: 8.0
  failure:
    enabled: true
    mtbf_hours: 100.0

stations:
  - id: "pick_1"
    node: "N100"
    type: pick
    concurrency: 2
    queue_capacity: 15
    service_time_s:
      distribution: lognormal
      base: 6.0
      base_stddev: 1.5
      per_item: 2.0
      per_item_stddev: 0.5

charging_stations:
  - id: "charger_1"
    node: "N50"
    bays: 4
    charge_rate_w: 250
    queue_capacity: 8

maintenance_stations:
  - id: "maint_1"
    node: "N75"
    bays: 2
    maintenance_duration_s: 300
    repair_time:
      distribution: lognormal
      base: 600
      base_stddev: 120

orders:
  arrival_process:
    type: poisson
    rate_per_min: 2.0
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
  reservation_enabled: false

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
  trace:
    enabled: false
    max_entries: 10000
    sample_rate: 1.0
```

---

## Default Values

Most optional fields have sensible defaults:

| Field | Default |
|-------|---------|
| `warmup_minutes` | 0 |
| `max_payload_kg` | 25 |
| `concurrency` | 1 |
| `queue_capacity` | unlimited |
| `traffic.policy` | `wait_at_node` |
| `routing.algorithm` | `dijkstra` |
| `metrics.sample_interval_s` | 60 |

See individual section pages for complete default values.

---

## Validation

Validate configurations before running:

```bash
waremax validate --scenario my_scenario.yaml
```

Common validation errors:

- Missing required fields
- Invalid parameter types
- Out-of-range values
- Unknown policy types
- Missing file references
