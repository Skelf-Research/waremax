# Metrics Configuration

Configuration for metrics collection and output.

---

## Schema

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

## Basic Settings

### sample_interval_s

**Type**: float
**Default**: 60

Interval in seconds for time-series sampling.

```yaml
metrics:
  sample_interval_s: 30  # Sample every 30 seconds
```

**Guidelines**:

| Use Case | Interval |
|----------|----------|
| High resolution | 10-30s |
| Standard | 60s |
| Long simulations | 120-300s |

### congestion_top_n

**Type**: integer
**Default**: 10

Number of top congested nodes/edges to track.

```yaml
metrics:
  congestion_top_n: 20
```

### track_sla

**Type**: boolean
**Default**: false

Track SLA compliance metrics.

```yaml
metrics:
  track_sla: true
```

---

## Breakdown Options

### per_robot_breakdown

**Type**: boolean
**Default**: false

Generate per-robot metrics breakdown.

```yaml
metrics:
  per_robot_breakdown: true
```

**Exports**: `robots.csv` with per-robot statistics.

### per_station_breakdown

**Type**: boolean
**Default**: false

Generate per-station metrics breakdown.

```yaml
metrics:
  per_station_breakdown: true
```

**Exports**: `stations.csv` with per-station statistics.

### generate_heatmap

**Type**: boolean
**Default**: false

Generate congestion heatmap data.

```yaml
metrics:
  generate_heatmap: true
```

**Exports**:

- `node_congestion.csv`
- `edge_congestion.csv`

---

## Event Tracing

### trace.enabled

**Type**: boolean
**Default**: false

Enable event tracing.

```yaml
metrics:
  trace:
    enabled: true
```

### trace.max_entries

**Type**: integer
**Default**: 10000

Maximum trace entries to store.

```yaml
metrics:
  trace:
    enabled: true
    max_entries: 50000
```

**Memory consideration**: Each entry uses ~100-200 bytes.

### trace.sample_rate

**Type**: float
**Default**: 1.0

Fraction of events to trace (0.0-1.0).

```yaml
metrics:
  trace:
    enabled: true
    sample_rate: 0.1  # Trace 10% of events
```

---

## Examples

### Minimal Metrics

```yaml
metrics:
  sample_interval_s: 60
```

### Standard Analysis

```yaml
metrics:
  sample_interval_s: 60
  per_robot_breakdown: true
  per_station_breakdown: true
```

### Detailed Analysis

```yaml
metrics:
  sample_interval_s: 30
  congestion_top_n: 20
  per_robot_breakdown: true
  per_station_breakdown: true
  generate_heatmap: true
```

### Full Tracing

```yaml
metrics:
  sample_interval_s: 30
  congestion_top_n: 25
  track_sla: true
  per_robot_breakdown: true
  per_station_breakdown: true
  generate_heatmap: true
  trace:
    enabled: true
    max_entries: 100000
    sample_rate: 1.0
```

### Production Monitoring

```yaml
metrics:
  sample_interval_s: 60
  congestion_top_n: 15
  track_sla: true
  per_robot_breakdown: true
  per_station_breakdown: true
  generate_heatmap: true
  trace:
    enabled: true
    max_entries: 50000
    sample_rate: 0.5  # 50% sampling for performance
```

---

## CLI Integration

Metrics settings can be overridden via CLI:

```bash
# Enable exports via CLI
waremax run --scenario scenario.yaml \
  --output-dir ./results \
  --per-robot \
  --per-station \
  --heatmap \
  --timeseries \
  --trace
```

CLI flags take precedence over config file settings.

---

## Output Files

When using `--output-dir`, generated files depend on settings:

| Setting | File Generated |
|---------|----------------|
| Always | `report.json` |
| `per_robot_breakdown` | `robots.csv` |
| `per_station_breakdown` | `stations.csv` |
| `generate_heatmap` | `node_congestion.csv`, `edge_congestion.csv` |
| `--timeseries` | `timeseries.csv` |
| `trace.enabled` | `trace.csv` |

---

## Performance Considerations

### Memory Usage

| Setting | Impact |
|---------|--------|
| `sample_interval_s` | Lower = more memory |
| `trace.max_entries` | Direct memory impact |
| `trace.sample_rate` | Lower = less memory |
| `generate_heatmap` | Moderate memory |

### Simulation Speed

| Setting | Impact on Speed |
|---------|-----------------|
| Basic metrics | Minimal |
| Per-robot/station | Low |
| Heatmap | Low-moderate |
| Full tracing (1.0) | Moderate |

---

## Related

- [Understanding Output](../getting-started/understanding-output.md)
- [Export Formats](../user-guide/export-formats.md)
- [KPIs](../concepts/metrics/kpis.md)
