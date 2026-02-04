# Export Formats

Waremax exports simulation data in multiple formats for analysis.

---

## Overview

When using `--output-dir`, Waremax generates:

| File | Contents | Flag |
|------|----------|------|
| `report.json` | Full simulation report | Always generated |
| `robots.csv` | Per-robot metrics | `--per-robot` |
| `stations.csv` | Per-station metrics | `--per-station` |
| `node_congestion.csv` | Node congestion data | `--heatmap` |
| `edge_congestion.csv` | Edge congestion data | `--heatmap` |
| `timeseries.csv` | Time series data | `--timeseries` |
| `trace.csv` | Event trace log | `--trace` |

---

## report.json

Complete simulation report in JSON format.

### Structure

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

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `duration_s` | float | Total simulation duration in seconds |
| `warmup_s` | float | Warmup period in seconds |
| `orders_completed` | integer | Number of completed orders |
| `throughput_per_hour` | float | Orders completed per hour |
| `cycle_times` | object | Cycle time statistics |
| `robot_utilization` | float | Average robot utilization (0-1) |
| `station_utilization` | float | Average station utilization (0-1) |
| `robot_reports` | array | Per-robot details (if enabled) |
| `station_reports` | array | Per-station details (if enabled) |
| `heatmap` | object | Congestion data (if enabled) |

---

## robots.csv

Per-robot performance metrics.

### Columns

| Column | Type | Description |
|--------|------|-------------|
| `robot_id` | integer | Robot identifier |
| `tasks_completed` | integer | Number of tasks finished |
| `distance_traveled_m` | float | Total distance in meters |
| `utilization` | float | Fraction of time busy (0-1) |
| `idle_time_s` | float | Time waiting for tasks |
| `travel_time_s` | float | Time spent moving |
| `service_time_s` | float | Time at stations |
| `wait_time_s` | float | Time waiting (traffic, queues) |

### Example

```csv
robot_id,tasks_completed,distance_traveled_m,utilization,idle_time_s,travel_time_s,service_time_s,wait_time_s
0,45,1250.5,0.72,840.0,1560.0,1200.0,200.0
1,42,1180.2,0.68,960.0,1440.0,1200.0,180.0
2,48,1320.8,0.75,750.0,1680.0,1320.0,250.0
```

### Analysis Tips

- High `wait_time_s` indicates congestion issues
- Uneven `tasks_completed` may suggest allocation inefficiency
- Low `utilization` with tasks waiting suggests routing/traffic problems

---

## stations.csv

Per-station performance metrics.

### Columns

| Column | Type | Description |
|--------|------|-------------|
| `station_id` | integer | Station identifier |
| `name` | string | Station name |
| `type` | string | Station type |
| `tasks_serviced` | integer | Robots serviced |
| `utilization` | float | Fraction of capacity used (0-1) |
| `avg_queue_length` | float | Average queue size |
| `max_queue_length` | integer | Peak queue size |
| `avg_wait_time_s` | float | Average time in queue |
| `avg_service_time_s` | float | Average service duration |

### Example

```csv
station_id,name,type,tasks_serviced,utilization,avg_queue_length,max_queue_length,avg_wait_time_s,avg_service_time_s
0,S1,pick,120,0.78,2.3,8,15.4,11.2
1,S2,pick,125,0.82,3.1,12,22.1,11.5
2,S3,pick,118,0.75,1.8,6,12.3,10.8
```

### Analysis Tips

- High `avg_queue_length` indicates station bottleneck
- Uneven `utilization` suggests assignment policy issues
- High `max_queue_length` may exceed `queue_capacity`

---

## node_congestion.csv

Congestion metrics per map node.

### Columns

| Column | Type | Description |
|--------|------|-------------|
| `node_id` | integer | Node identifier |
| `congestion_score` | float | Normalized congestion score (0-1) |
| `wait_events` | integer | Number of wait events |
| `total_wait_time_s` | float | Total time robots waited |

### Example

```csv
node_id,congestion_score,wait_events,total_wait_time_s
0,0.85,45,234.5
5,0.72,32,156.2
10,0.45,18,67.3
```

### Analysis Tips

- High `congestion_score` nodes are hotspots
- Station nodes often have high scores (expected)
- High scores on aisle nodes indicate traffic bottlenecks

---

## edge_congestion.csv

Congestion metrics per map edge.

### Columns

| Column | Type | Description |
|--------|------|-------------|
| `edge_id` | integer | Edge identifier |
| `from_node` | integer | Source node |
| `to_node` | integer | Destination node |
| `congestion_score` | float | Normalized congestion score (0-1) |
| `wait_events` | integer | Number of wait events |
| `total_wait_time_s` | float | Total time robots waited |

### Example

```csv
edge_id,from_node,to_node,congestion_score,wait_events,total_wait_time_s
0,0,1,0.65,28,89.3
1,1,2,0.45,15,42.1
5,0,5,0.82,38,145.6
```

### Analysis Tips

- High-score edges may need capacity increase
- Consider adding parallel paths for congested routes

---

## timeseries.csv

Time series data for trend analysis.

### Columns

| Column | Type | Description |
|--------|------|-------------|
| `time_s` | float | Simulation time in seconds |
| `metric` | string | Metric name |
| `entity_id` | string | Entity identifier |
| `value` | float | Metric value |

### Metrics

| Metric | Description |
|--------|-------------|
| `queue_length` | Station queue size |
| `robot_position` | Robot location |
| `orders_completed` | Cumulative orders |
| `utilization` | Current utilization |

### Example

```csv
time_s,metric,entity_id,value
60.0,queue_length,station_0,3
60.0,queue_length,station_1,5
60.0,orders_completed,system,12
120.0,queue_length,station_0,2
120.0,queue_length,station_1,7
120.0,orders_completed,system,25
```

### Analysis Tips

- Plot queue lengths over time to identify patterns
- Look for correlation between metrics
- Identify periodic patterns (shift changes, order surges)

---

## trace.csv

Detailed event log for debugging and analysis.

### Columns

| Column | Type | Description |
|--------|------|-------------|
| `time_s` | float | Event time |
| `event_type` | string | Event type |
| `entity_id` | string | Entity involved |
| `details` | string | JSON details |

### Event Types

| Event | Description |
|-------|-------------|
| `OrderArrival` | New order entered system |
| `TaskAssignment` | Task assigned to robot |
| `RobotDepart` | Robot started moving |
| `RobotArrive` | Robot reached destination |
| `QueueJoin` | Robot joined station queue |
| `ServiceStart` | Robot began service |
| `ServiceEnd` | Robot completed service |
| `ChargingStart` | Robot began charging |
| `ChargingEnd` | Robot finished charging |
| `MaintenanceStart` | Robot began maintenance |
| `MaintenanceEnd` | Robot completed maintenance |
| `FailureOccurred` | Robot experienced failure |

### Example

```csv
time_s,event_type,entity_id,details
0.0,OrderArrival,order_0,"{\"lines\":3,\"skus\":[\"SKU001\",\"SKU002\",\"SKU003\"]}"
0.5,TaskAssignment,robot_0,"{\"task_id\":0,\"order_id\":0}"
0.5,RobotDepart,robot_0,"{\"from\":5,\"to\":12,\"distance\":15.0}"
10.3,RobotArrive,robot_0,"{\"node\":12}"
```

### Analysis Tips

- Trace specific orders through the system
- Identify where delays occur
- Debug unexpected behavior

---

## Loading Data

### Python (pandas)

```python
import pandas as pd
import json

# Load CSV files
robots = pd.read_csv('results/robots.csv')
stations = pd.read_csv('results/stations.csv')
timeseries = pd.read_csv('results/timeseries.csv')

# Load JSON report
with open('results/report.json') as f:
    report = json.load(f)

# Basic analysis
print(f"Throughput: {report['throughput_per_hour']:.1f} orders/hr")
print(f"Avg robot utilization: {robots['utilization'].mean():.1%}")
print(f"Max station queue: {stations['max_queue_length'].max()}")
```

### R

```r
library(jsonlite)

# Load CSV files
robots <- read.csv('results/robots.csv')
stations <- read.csv('results/stations.csv')

# Load JSON report
report <- fromJSON('results/report.json')

# Basic analysis
cat(sprintf("Throughput: %.1f orders/hr\n", report$throughput_per_hour))
cat(sprintf("Avg utilization: %.1f%%\n", mean(robots$utilization) * 100))
```

---

## Next Steps

- **[Understanding Output](../getting-started/understanding-output.md)** - Interpreting results
- **[Root Cause Analysis](../concepts/metrics/rca.md)** - Automatic analysis
- **[Tutorials](../tutorials/index.md)** - Step-by-step guides
