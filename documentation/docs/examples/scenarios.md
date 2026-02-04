# Scenario Examples

Complete scenario configurations for common use cases.

---

## Small Warehouse

A compact warehouse with basic configuration.

```yaml
# small_warehouse.yaml
# 10 racks, 5 robots, 2 pick stations

simulation:
  duration_s: 3600
  seed: 12345

map:
  nodes:
    # Main aisle
    - { id: 0, name: A0, x: 0, y: 5, type: aisle }
    - { id: 1, name: A1, x: 5, y: 5, type: aisle }
    - { id: 2, name: A2, x: 10, y: 5, type: aisle }
    - { id: 3, name: A3, x: 15, y: 5, type: aisle }

    # Top rack row
    - { id: 10, name: R1, x: 2, y: 8, type: rack }
    - { id: 11, name: R2, x: 5, y: 8, type: rack }
    - { id: 12, name: R3, x: 8, y: 8, type: rack }
    - { id: 13, name: R4, x: 11, y: 8, type: rack }
    - { id: 14, name: R5, x: 14, y: 8, type: rack }

    # Bottom rack row
    - { id: 20, name: R6, x: 2, y: 2, type: rack }
    - { id: 21, name: R7, x: 5, y: 2, type: rack }
    - { id: 22, name: R8, x: 8, y: 2, type: rack }
    - { id: 23, name: R9, x: 11, y: 2, type: rack }
    - { id: 24, name: R10, x: 14, y: 2, type: rack }

    # Stations
    - { id: 30, name: S1, x: 20, y: 7, type: station_pick }
    - { id: 31, name: S2, x: 20, y: 3, type: station_pick }

    # Charging
    - { id: 40, name: CH1, x: -3, y: 5, type: charging }

  edges:
    # Main aisle
    - { from: 0, to: 1, bidirectional: true }
    - { from: 1, to: 2, bidirectional: true }
    - { from: 2, to: 3, bidirectional: true }

    # Top rack access
    - { from: 0, to: 10, bidirectional: true }
    - { from: 1, to: 11, bidirectional: true }
    - { from: 1, to: 12, bidirectional: true }
    - { from: 2, to: 13, bidirectional: true }
    - { from: 3, to: 14, bidirectional: true }

    # Bottom rack access
    - { from: 0, to: 20, bidirectional: true }
    - { from: 1, to: 21, bidirectional: true }
    - { from: 1, to: 22, bidirectional: true }
    - { from: 2, to: 23, bidirectional: true }
    - { from: 3, to: 24, bidirectional: true }

    # Station access
    - { from: 3, to: 30, bidirectional: true }
    - { from: 3, to: 31, bidirectional: true }

    # Charging access
    - { from: 0, to: 40, bidirectional: true }

robots:
  count: 5
  speed_m_s: 1.5
  initial_positions: [0, 1, 2, 3, 0]

stations:
  - id: S1
    node: 30
    type: pick
    concurrency: 2
    service_time_s:
      distribution: lognormal
      base: 5.0
      base_stddev: 1.0

  - id: S2
    node: 31
    type: pick
    concurrency: 1
    service_time_s:
      distribution: constant
      base: 6.0

charging_stations:
  - id: CH1
    node: 40
    bays: 2
    charge_rate_w: 200

battery:
  capacity_wh: 500
  charge_threshold_pct: 20

orders:
  generation:
    type: poisson
    rate_per_hour: 150

policies:
  task_allocation: nearest_idle
  station_assignment: shortest_queue
```

---

## Medium Warehouse with Zones

Zone-based layout with fast and slow movers.

```yaml
# medium_warehouse_zones.yaml
# 2 zones, 20 racks, 10 robots, 3 stations

simulation:
  duration_s: 7200
  seed: 54321

map:
  nodes:
    # Zone A - Fast movers (near stations)
    - { id: 0, name: ZA_A0, x: 0, y: 10, type: aisle }
    - { id: 1, name: ZA_A1, x: 5, y: 10, type: aisle }
    - { id: 2, name: ZA_A2, x: 10, y: 10, type: aisle }

    # Zone A racks (10 racks)
    - { id: 10, name: ZA_R1, x: 2, y: 12, type: rack }
    - { id: 11, name: ZA_R2, x: 5, y: 12, type: rack }
    - { id: 12, name: ZA_R3, x: 8, y: 12, type: rack }
    - { id: 13, name: ZA_R4, x: 2, y: 8, type: rack }
    - { id: 14, name: ZA_R5, x: 5, y: 8, type: rack }
    - { id: 15, name: ZA_R6, x: 8, y: 8, type: rack }

    # Zone B - Slow movers (further from stations)
    - { id: 20, name: ZB_A0, x: 0, y: 0, type: aisle }
    - { id: 21, name: ZB_A1, x: 5, y: 0, type: aisle }
    - { id: 22, name: ZB_A2, x: 10, y: 0, type: aisle }

    # Zone B racks (10 racks)
    - { id: 30, name: ZB_R1, x: 2, y: 2, type: rack }
    - { id: 31, name: ZB_R2, x: 5, y: 2, type: rack }
    - { id: 32, name: ZB_R3, x: 8, y: 2, type: rack }
    - { id: 33, name: ZB_R4, x: 2, y: -2, type: rack }
    - { id: 34, name: ZB_R5, x: 5, y: -2, type: rack }
    - { id: 35, name: ZB_R6, x: 8, y: -2, type: rack }

    # Connecting corridor
    - { id: 50, name: C0, x: 0, y: 5, type: aisle, capacity: 2 }
    - { id: 51, name: C1, x: 5, y: 5, type: aisle, capacity: 2 }
    - { id: 52, name: C2, x: 10, y: 5, type: aisle, capacity: 2 }

    # Stations
    - { id: 60, name: S1, x: 15, y: 10, type: station_pick }
    - { id: 61, name: S2, x: 15, y: 5, type: station_pick }
    - { id: 62, name: S3, x: 15, y: 0, type: station_pick }

  edges:
    # Zone A internal
    - { from: 0, to: 1, bidirectional: true }
    - { from: 1, to: 2, bidirectional: true }
    - { from: 0, to: 10, bidirectional: true }
    - { from: 1, to: 11, bidirectional: true }
    - { from: 2, to: 12, bidirectional: true }
    - { from: 0, to: 13, bidirectional: true }
    - { from: 1, to: 14, bidirectional: true }
    - { from: 2, to: 15, bidirectional: true }

    # Zone B internal
    - { from: 20, to: 21, bidirectional: true }
    - { from: 21, to: 22, bidirectional: true }
    - { from: 20, to: 30, bidirectional: true }
    - { from: 21, to: 31, bidirectional: true }
    - { from: 22, to: 32, bidirectional: true }
    - { from: 20, to: 33, bidirectional: true }
    - { from: 21, to: 34, bidirectional: true }
    - { from: 22, to: 35, bidirectional: true }

    # Corridor
    - { from: 50, to: 51, bidirectional: true, capacity: 2 }
    - { from: 51, to: 52, bidirectional: true, capacity: 2 }

    # Zone connections
    - { from: 0, to: 50, bidirectional: true }
    - { from: 50, to: 20, bidirectional: true }
    - { from: 1, to: 51, bidirectional: true }
    - { from: 51, to: 21, bidirectional: true }
    - { from: 2, to: 52, bidirectional: true }
    - { from: 52, to: 22, bidirectional: true }

    # Station access
    - { from: 2, to: 60, bidirectional: true }
    - { from: 52, to: 61, bidirectional: true }
    - { from: 22, to: 62, bidirectional: true }

robots:
  count: 10
  speed_m_s: 1.5

stations:
  - { id: S1, node: 60, type: pick, concurrency: 2 }
  - { id: S2, node: 61, type: pick, concurrency: 2 }
  - { id: S3, node: 62, type: pick, concurrency: 1 }

orders:
  generation:
    type: poisson
    rate_per_hour: 300

  sku_popularity:
    type: zipf
    alpha: 1.2  # More concentrated in Zone A

policies:
  task_allocation: nearest_idle
  station_assignment: fastest_completion

routing:
  policy: congestion_aware
  congestion_weight: 1.5
```

---

## High-Volume Distribution Center

Large-scale operation with many robots.

```yaml
# distribution_center.yaml
# 50 racks, 30 robots, 5 stations

simulation:
  duration_s: 14400  # 4 hours
  seed: 99999

map:
  # Generated grid layout
  # Use: waremax generate map --type grid-with-racks --rows 10 --cols 5

robots:
  count: 30
  speed_m_s: 2.0

stations:
  - { id: S1, node: 100, type: pick, concurrency: 3 }
  - { id: S2, node: 101, type: pick, concurrency: 3 }
  - { id: S3, node: 102, type: pick, concurrency: 3 }
  - { id: S4, node: 103, type: pick, concurrency: 2 }
  - { id: S5, node: 104, type: pick, concurrency: 2 }

orders:
  generation:
    type: poisson
    rate_per_hour: 1000

  items_per_order:
    distribution: uniform
    min: 1
    max: 8

policies:
  task_allocation: nearest_idle
  station_assignment: fastest_completion
  batching:
    enabled: true
    max_batch_size: 5
    max_wait_time_s: 30

traffic:
  deadlock_detection: true
  reroute_on_congestion: true
```

---

## Next Steps

- [Policy Comparisons](policy-comparisons.md): Compare different policies
- [Capacity Studies](capacity-studies.md): Fleet sizing examples
