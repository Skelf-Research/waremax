# Your First Simulation

This guide walks you through creating and running your first custom warehouse simulation.

---

## Overview

In this tutorial, you will:

1. Create a scenario configuration file
2. Run the simulation
3. Examine the results
4. Make modifications and re-run

---

## Step 1: Create a Scenario File

Create a new file called `my_warehouse.yaml`:

```yaml
# My first warehouse simulation
seed: 42

simulation:
  duration_minutes: 30
  warmup_minutes: 5

map:
  file: map.json

storage:
  file: storage.yaml

robots:
  count: 10
  max_speed_mps: 1.5
  max_payload_kg: 25

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

  - id: "S2"
    node: "5"
    type: pick
    concurrency: 2
    queue_capacity: 10
    service_time_s:
      distribution: constant
      base: 5.0
      per_item: 2.0

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
  due_times:
    type: fixed_offset
    minutes: 60

policies:
  task_allocation:
    type: nearest_robot
  station_assignment:
    type: least_queue
  batching:
    type: none
  priority:
    type: strict_priority

traffic:
  policy: wait_at_node
  edge_capacity_default: 1
  node_capacity_default: 1

routing:
  algorithm: dijkstra
  cache_routes: true
```

---

## Step 2: Understand the Configuration

Let's break down the key sections:

### Simulation Settings

```yaml
seed: 42                    # Random seed for reproducibility
simulation:
  duration_minutes: 30      # Total simulation time
  warmup_minutes: 5         # Warmup period (metrics excluded)
```

### Robot Fleet

```yaml
robots:
  count: 10                 # Number of robots
  max_speed_mps: 1.5        # Speed in meters per second
  max_payload_kg: 25        # Maximum payload capacity
```

### Stations

```yaml
stations:
  - id: "S1"                # Station identifier
    node: "0"               # Map node where station is located
    type: pick              # Station type (pick, drop, inbound, outbound)
    concurrency: 2          # Robots served simultaneously
    queue_capacity: 10      # Maximum queue length
    service_time_s:         # Time to service each robot
      distribution: constant
      base: 5.0             # Base time in seconds
      per_item: 2.0         # Additional time per item
```

### Order Generation

```yaml
orders:
  arrival_process:
    type: poisson           # Poisson arrival process
    rate_per_min: 1.0       # 1 order per minute = 60 orders/hour
  lines_per_order:
    type: negbinomial       # Negative binomial distribution
    mean: 3.0               # Average 3 items per order
```

### Policies

```yaml
policies:
  task_allocation:
    type: nearest_robot     # Assign tasks to nearest available robot
  station_assignment:
    type: least_queue       # Send robots to station with shortest queue
```

---

## Step 3: Run the Simulation

Run your scenario:

```bash
waremax run --scenario my_warehouse.yaml
```

Expected output:

```
Loading scenario from: my_warehouse.yaml
Running simulation with seed: 42
Duration: 30 minutes (warmup: 5 minutes)
Distributions:
  Arrivals: Poisson(rate=0.017/s)
  Lines/Order: NegBinomial(mean=3.0, dispersion=1.0)
  SKU Selection: Zipf(alpha=1.0)
Policies:
  Task Allocation: nearest_robot
  Station Assignment: least_queue
  Batching: none
  Priority: strict_priority

Simulation Complete
==================
Duration: 30.0 minutes (warmup: 5.0 minutes)

Orders:
  Completed: 25
  Throughput: 60.0 orders/hr

Cycle Times:
  Average: 45.2s
  P95: 82.3s
  P99: 98.1s

Utilization:
  Robot Fleet: 52.3%
  Stations: 48.7%
```

---

## Step 4: Export Detailed Results

Get more detailed output:

```bash
waremax run --scenario my_warehouse.yaml \
  --output-dir ./results \
  --per-robot \
  --per-station \
  --heatmap
```

Check the results directory:

```bash
ls ./results/
# report.json  robots.csv  stations.csv  node_congestion.csv  edge_congestion.csv
```

---

## Step 5: Experiment with Changes

### Increase Robot Count

Edit `my_warehouse.yaml`:

```yaml
robots:
  count: 15  # Changed from 10
```

Re-run and compare:

```bash
waremax run --scenario my_warehouse.yaml
```

### Try Different Policies

Change the task allocation policy:

```yaml
policies:
  task_allocation:
    type: workload_balanced  # Changed from nearest_robot
```

### Increase Order Rate

```yaml
orders:
  arrival_process:
    type: poisson
    rate_per_min: 2.0  # Doubled from 1.0
```

---

## Step 6: Use JSON Output

For programmatic analysis:

```bash
waremax run --scenario my_warehouse.yaml --output json > results.json
```

Parse with jq:

```bash
# Get throughput
jq '.throughput_per_hour' results.json

# Get cycle times
jq '.cycle_times' results.json

# Get robot utilization
jq '.robot_utilization' results.json
```

---

## Common Modifications

### Add Battery Simulation

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
      idle_power_w: 5.0
      service_power_w: 20.0

charging_stations:
  - id: "C1"
    node: "10"
    bays: 2
    charge_rate_w: 200
```

### Enable Maintenance

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
```

### Use Lognormal Service Times

```yaml
stations:
  - id: "S1"
    node: "0"
    type: pick
    concurrency: 2
    service_time_s:
      distribution: lognormal
      base: 8.0
      base_stddev: 2.0
      per_item: 2.0
      per_item_stddev: 0.5
```

---

## Troubleshooting

### "Map file not found"

The scenario references `map.json` which doesn't exist. For a quick test, Waremax generates a default grid map. For custom maps, see the [Map Configuration](../user-guide/map-configuration.md) guide.

### "No orders completed"

- Check that `rate_per_min` is greater than 0
- Ensure stations exist and are configured
- Verify robots can reach station nodes

### Simulation runs slowly

- Use release builds: `cargo build --release`
- Reduce simulation duration for testing
- Check for deadlock situations with many robots

---

## Next Steps

- **[Understanding Output](understanding-output.md)** - Learn to interpret all metrics
- **[Scenario Files](../user-guide/scenario-files.md)** - Deep dive into configuration
- **[Working with Presets](../user-guide/presets.md)** - Use predefined configurations
