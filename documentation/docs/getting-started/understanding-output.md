# Understanding Output

Learn how to interpret Waremax simulation results and metrics.

---

## Console Output

When you run a simulation, Waremax displays a summary:

```
Simulation Complete
==================
Duration: 60.0 minutes (warmup: 5.0 minutes)

Orders:
  Completed: 245
  Throughput: 267.3 orders/hr

Cycle Times:
  Average: 42.3s
  P95: 78.5s
  P99: 95.2s

Utilization:
  Robot Fleet: 67.2%
  Stations: 72.5%
```

---

## Key Metrics Explained

### Orders Section

| Metric | Description |
|--------|-------------|
| **Completed** | Total orders finished during measurement period (excludes warmup) |
| **Throughput** | Orders completed per hour, extrapolated to hourly rate |

!!! note "Warmup Period"
    Metrics are collected only after the warmup period ends. This ensures the system reaches steady-state before measurement.

### Cycle Times

| Metric | Description |
|--------|-------------|
| **Average** | Mean time from order arrival to completion |
| **P95** | 95th percentile - 95% of orders complete within this time |
| **P99** | 99th percentile - 99% of orders complete within this time |

**Interpreting cycle times:**

- Large gap between Average and P95 indicates high variability
- High P99 may indicate occasional bottlenecks or edge cases
- Consistent values suggest stable system performance

### Utilization

| Metric | Description |
|--------|-------------|
| **Robot Fleet** | Percentage of time robots are busy (traveling or servicing) |
| **Stations** | Percentage of station capacity in use |

**Utilization guidelines:**

- **< 50%**: Under-utilized, may have excess capacity
- **50-80%**: Good balance of capacity and throughput
- **> 85%**: High utilization, may experience queuing delays
- **> 95%**: Likely capacity-constrained, expect long queues

---

## JSON Output

For detailed analysis, use JSON output:

```bash
waremax run --scenario my_scenario.yaml --output json
```

### JSON Structure

```json
{
  "duration_s": 3600.0,
  "warmup_s": 300.0,
  "orders_completed": 245,
  "throughput_per_hour": 267.3,
  "cycle_times": {
    "mean_s": 42.3,
    "median_s": 38.5,
    "p95_s": 78.5,
    "p99_s": 95.2,
    "min_s": 12.1,
    "max_s": 145.8
  },
  "robot_utilization": 0.672,
  "station_utilization": 0.725,
  "robot_reports": [...],
  "station_reports": [...],
  "heatmap": {...}
}
```

---

## Export Files

When using `--output-dir`, Waremax generates multiple files:

### report.json

Complete simulation report in JSON format.

### robots.csv

Per-robot metrics:

```csv
robot_id,tasks_completed,distance_traveled_m,utilization,idle_time_s,travel_time_s,service_time_s
0,45,1250.5,0.72,840.0,1560.0,1200.0
1,42,1180.2,0.68,960.0,1440.0,1200.0
...
```

| Column | Description |
|--------|-------------|
| `robot_id` | Robot identifier |
| `tasks_completed` | Number of tasks finished |
| `distance_traveled_m` | Total distance in meters |
| `utilization` | Fraction of time busy |
| `idle_time_s` | Time waiting for tasks |
| `travel_time_s` | Time moving |
| `service_time_s` | Time at stations |

### stations.csv

Per-station metrics:

```csv
station_id,name,tasks_serviced,utilization,avg_queue_length,max_queue_length,avg_wait_time_s
0,S1,120,0.78,2.3,8,15.4
1,S2,125,0.82,3.1,12,22.1
...
```

| Column | Description |
|--------|-------------|
| `station_id` | Station identifier |
| `name` | Station name |
| `tasks_serviced` | Number of robots serviced |
| `utilization` | Fraction of capacity used |
| `avg_queue_length` | Average robots waiting |
| `max_queue_length` | Peak queue size |
| `avg_wait_time_s` | Average time in queue |

### node_congestion.csv

Congestion data per map node:

```csv
node_id,congestion_score,wait_events,total_wait_time_s
0,0.85,45,234.5
5,0.72,32,156.2
...
```

### edge_congestion.csv

Congestion data per map edge:

```csv
edge_id,from_node,to_node,congestion_score,wait_events,total_wait_time_s
0,0,1,0.65,28,89.3
1,1,2,0.45,15,42.1
...
```

### timeseries.csv

Time series data (when `--timeseries` is used):

```csv
time_s,metric,entity_id,value
60.0,queue_length,station_0,3
60.0,queue_length,station_1,5
120.0,queue_length,station_0,2
...
```

### trace.csv

Event trace (when `--trace` is used):

```csv
time_s,event_type,entity_id,details
0.0,OrderArrival,order_0,"{\"lines\":3}"
0.5,TaskAssignment,robot_0,"{\"task_id\":0}"
...
```

---

## Interpreting Results

### Healthy System Indicators

- Throughput matches or exceeds order arrival rate
- Cycle times are consistent (low P95/P99 gap)
- Robot utilization 50-80%
- Station queues rarely exceed capacity

### Warning Signs

| Symptom | Possible Cause |
|---------|----------------|
| Low throughput | Not enough robots, station bottleneck |
| High cycle times | Long queues, congestion, routing delays |
| Very high utilization | System at capacity limit |
| Uneven robot utilization | Poor task allocation policy |
| High station queue lengths | Station under-capacity |

### Optimization Hints

1. **Low throughput + low utilization** → Check routing, may have congestion
2. **High cycle time + high utilization** → Add capacity (robots or stations)
3. **Uneven station utilization** → Adjust station assignment policy
4. **High queue variability** → Consider batching or different service times

---

## Using Results for Analysis

### Compare Configurations

```bash
# Run baseline
waremax run --scenario baseline.yaml --output json > baseline.json

# Run variant
waremax run --scenario variant.yaml --output json > variant.json

# Compare with jq
echo "Throughput improvement:"
echo "Baseline: $(jq '.throughput_per_hour' baseline.json)"
echo "Variant: $(jq '.throughput_per_hour' variant.json)"
```

### Visualize Time Series

Export time series and visualize with your preferred tool:

```bash
waremax run --scenario my_scenario.yaml --output-dir ./results --timeseries
```

Load `results/timeseries.csv` into Python, R, or Excel for visualization.

### Identify Bottlenecks

Use the analyze command for automatic bottleneck detection:

```bash
waremax analyze --scenario my_scenario.yaml --detailed
```

---

## Next Steps

- **[Running Simulations](../user-guide/running-simulations.md)** - Advanced run options
- **[Root Cause Analysis](../concepts/metrics/rca.md)** - Automatic bottleneck detection
- **[Parameter Sweeps](../tutorials/testing/parameter-sweeps.md)** - Systematic experimentation
