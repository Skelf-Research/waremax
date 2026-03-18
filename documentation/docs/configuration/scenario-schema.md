# Scenario Schema

Complete schema reference for scenario configuration files.

---

## Schema Overview

```yaml
# Required fields
seed: <integer>                    # Random seed

simulation:                        # Simulation timing
  duration_minutes: <float>        # Required
  warmup_minutes: <float>          # Default: 0
  time_unit: <string>              # Default: "seconds"

map:
  file: <string>                   # Path to map file

storage:
  file: <string>                   # Path to storage file

robots:                            # Robot configuration
  count: <integer>                 # Required
  max_speed_mps: <float>          # Required
  max_payload_kg: <float>         # Default: 25
  battery: <BatteryConfig>        # Optional
  maintenance: <MaintenanceConfig> # Optional
  failure: <FailureConfig>        # Optional

stations:                          # List of stations
  - <StationConfig>

orders:                            # Order generation
  arrival_process: <ArrivalProcess>
  lines_per_order: <LinesConfig>
  sku_popularity: <SkuPopularity>
  due_times: <DueTimeConfig>       # Optional

# Optional sections
policies: <PolicyConfig>
traffic: <TrafficConfig>
routing: <RoutingConfig>
charging_stations: [<ChargingStationConfig>]
maintenance_stations: [<MaintenanceStationConfig>]
inbound: <InboundConfig>
replenishment: <ReplenishmentConfig>
metrics: <MetricsConfig>
```

---

## Type Definitions

### SimulationParams

```yaml
simulation:
  duration_minutes: <float>        # Total simulation duration
  warmup_minutes: <float>          # Warmup period (default: 0)
  time_unit: <string>              # "seconds" | "minutes" (default: "seconds")
```

### RobotConfig

```yaml
robots:
  count: <integer>                 # Number of robots (>0)
  max_speed_mps: <float>          # Speed in m/s (>0)
  max_payload_kg: <float>         # Payload capacity (default: 25)
  battery:                         # Optional battery config
    enabled: <boolean>
    capacity_wh: <float>
    min_soc: <float>              # 0-1
    consumption:
      per_meter_wh: <float>
      per_kg_per_meter_wh: <float>
      idle_power_w: <float>
      service_power_w: <float>
  maintenance:                     # Optional maintenance
    enabled: <boolean>
    interval_hours: <float>
  failure:                         # Optional failures
    enabled: <boolean>
    mtbf_hours: <float>
```

### StationConfig

```yaml
stations:
  - id: <string>                   # Unique identifier
    node: <string>                 # Map node ID
    type: <string>                 # "pick" | "drop" | "inbound" | "outbound"
    concurrency: <integer>         # Default: 1
    queue_capacity: <integer>      # Optional (unlimited if not set)
    service_time_s:
      distribution: <string>       # "constant" | "lognormal" | "exponential" | "uniform"
      base: <float>               # Base time
      per_item: <float>           # Per-item time
      # Lognormal specific:
      base_stddev: <float>
      per_item_stddev: <float>
      # Uniform specific:
      min_s: <float>
      max_s: <float>
```

### OrderConfig

```yaml
orders:
  arrival_process:
    type: <string>                 # "poisson"
    rate_per_min: <float>         # Arrival rate

  lines_per_order:
    type: <string>                 # "negbinomial"
    mean: <float>                 # Average lines per order
    dispersion: <float>           # Default: 1.0

  sku_popularity:
    type: <string>                 # "zipf"
    alpha: <float>                # Default: 1.0

  due_times:                       # Optional
    type: <string>                 # "fixed_offset"
    minutes: <float>              # Offset from order arrival
```

### PolicyConfig

```yaml
policies:
  task_allocation:
    type: <string>                 # "nearest_robot" | "auction" | "workload_balanced"
    travel_weight: <float>        # For auction (default: 1.0)
    queue_weight: <float>         # For auction (default: 0.5)

  station_assignment:
    type: <string>                 # "least_queue" | "fastest_service" | "due_time_priority"

  batching:
    type: <string>                 # "none" | "station_batch" | "zone_batch"
    max_items: <integer>          # Max items per batch
    max_weight_kg: <float>        # Max weight per batch

  priority:
    type: <string>                 # "strict_priority" | "weighted_fair" | "sla_driven"
    pick_weight: <integer>        # For weighted_fair
    putaway_weight: <integer>
    replen_weight: <integer>
```

### TrafficConfig

```yaml
traffic:
  policy: <string>                 # "wait_at_node" | "reroute_on_wait" | "adaptive_traffic"
  edge_capacity_default: <integer> # Default: 1
  node_capacity_default: <integer> # Default: 1
  wait_threshold_s: <float>       # Default: 2.0
  max_reroute_attempts: <integer> # Default: 3
  deadlock_detection: <boolean>   # Default: false
  deadlock_resolver: <string>     # "youngest_backs_up" | "lowest_priority_aborts" | "tiered"
  deadlock_check_interval_s: <float>
  reservation_enabled: <boolean>  # Default: false
  reservation_lookahead_s: <float> # Default: 30.0
```

### RoutingConfig

```yaml
routing:
  algorithm: <string>              # "dijkstra" | "astar"
  congestion_aware: <boolean>     # Default: false
  cache_routes: <boolean>         # Default: true
  congestion_weight: <float>      # Default: 0.5
```

### ChargingStationConfig

```yaml
charging_stations:
  - id: <string>
    node: <string>
    bays: <integer>               # Default: 1
    charge_rate_w: <float>        # Default: 200
    queue_capacity: <integer>     # Optional
```

### MaintenanceStationConfig

```yaml
maintenance_stations:
  - id: <string>
    node: <string>
    bays: <integer>               # Default: 2
    maintenance_duration_s: <float> # Default: 300
    repair_time:                  # Optional ServiceTimeConfig
      distribution: <string>
      base: <float>
      ...
    queue_capacity: <integer>     # Optional
```

### MetricsConfig

```yaml
metrics:
  sample_interval_s: <float>      # Default: 60
  congestion_top_n: <integer>     # Default: 10
  track_sla: <boolean>            # Default: false
  per_robot_breakdown: <boolean>  # Default: false
  per_station_breakdown: <boolean> # Default: false
  generate_heatmap: <boolean>     # Default: false
  trace:
    enabled: <boolean>            # Default: false
    max_entries: <integer>        # Default: 10000
    sample_rate: <float>          # Default: 1.0
```

---

## Constraints

### Required Constraints

| Field | Constraint |
|-------|------------|
| `seed` | > 0 |
| `duration_minutes` | > 0 |
| `robots.count` | > 0 |
| `robots.max_speed_mps` | > 0 |
| `stations` | At least 1 |
| `orders.arrival_process.rate_per_min` | > 0 |

### Optional Constraints

| Field | Constraint |
|-------|------------|
| `warmup_minutes` | >= 0 |
| `max_payload_kg` | > 0 |
| `battery.capacity_wh` | > 0 |
| `battery.min_soc` | 0-1 |
| `concurrency` | > 0 |
| `service_time_s.base` | >= 0 |

---

## File References

File paths are relative to the scenario file location:

```yaml
# Scenario at /project/scenarios/test.yaml
map:
  file: ../maps/warehouse.json    # -> /project/maps/warehouse.json

storage:
  file: inventory.yaml            # -> /project/scenarios/inventory.yaml
```

Absolute paths are also supported:

```yaml
map:
  file: /data/maps/warehouse.json
```
