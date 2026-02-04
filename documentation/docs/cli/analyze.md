# waremax analyze

Run root cause analysis on a simulation.

---

## Synopsis

```bash
waremax analyze --scenario <PATH> [OPTIONS]
```

---

## Description

The `analyze` command runs a simulation with attribution tracking enabled, then performs root cause analysis (RCA) to identify bottlenecks, anomalies, and provide recommendations for improvement.

---

## Options

### Required

| Option | Description |
|--------|-------------|
| `--scenario`, `-s` | Path to scenario file |

### Optional

| Option | Default | Description |
|--------|---------|-------------|
| `--output`, `-o` | None | Output file for RCA report |
| `--format` | text | Output format: `text`, `json`, `compact` |
| `--detailed` | false | Include detailed analysis |
| `--anomaly-threshold` | 2.0 | Anomaly detection threshold (z-score) |

---

## Examples

### Basic analysis

```bash
waremax analyze --scenario my_scenario.yaml
```

### Detailed analysis

```bash
waremax analyze --scenario my_scenario.yaml --detailed
```

### JSON output

```bash
waremax analyze --scenario my_scenario.yaml --format json
```

### Save report

```bash
waremax analyze --scenario my_scenario.yaml \
  --output rca_report.txt \
  --detailed
```

### Custom anomaly threshold

```bash
waremax analyze --scenario my_scenario.yaml \
  --anomaly-threshold 1.5 \
  --detailed
```

---

## Output

### Console Output (Text Format)

```
Running Root Cause Analysis...
Scenario: my_scenario.yaml
Running simulation with seed: 42

Simulation complete.
Orders completed: 245
Throughput: 267.3 orders/hr

============================================================
ROOT CAUSE ANALYSIS REPORT
============================================================

HEALTH SCORE: 72/100

SUMMARY
-------
Orders Analyzed: 245
Primary Delay Source: Station Queue Wait
Total Delay Time: 3,456s

BOTTLENECK ANALYSIS
-------------------
Total Bottlenecks Found: 5

Top Bottlenecks by Impact:

1. STATION BOTTLENECK - Station S2 (pick)
   Severity: HIGH
   Impact: 28.5% of total delay
   Avg Queue: 4.2 robots
   Max Queue: 12 robots
   Utilization: 92.3%
   Recommendation: Increase concurrency or add parallel station

2. CONGESTION HOTSPOT - Node 15
   Severity: MEDIUM
   Impact: 15.2% of total delay
   Wait Events: 89
   Total Wait: 523s
   Recommendation: Consider alternate routes or increase capacity

3. STATION BOTTLENECK - Station S1 (pick)
   Severity: MEDIUM
   Impact: 12.8% of total delay
   Avg Queue: 2.8 robots
   Max Queue: 8 robots
   Utilization: 85.1%

ANOMALIES DETECTED
------------------
Anomalies Found: 3

1. Station S2 queue spike at t=1,234s
   Z-score: 3.2
   Queue Length: 12 (expected: 4.2 ± 2.4)

2. Unusual robot idle time for Robot 7
   Z-score: 2.8
   Idle: 45.2% (expected: 28.3% ± 6.1%)

DELAY ATTRIBUTION
-----------------
Travel Time:     42.3% (avg 18.5s per order)
Queue Wait:      35.2% (avg 15.4s per order)
Service Time:    18.5% (avg 8.1s per order)
Traffic Wait:     4.0% (avg 1.7s per order)

RECOMMENDATIONS
---------------
1. [HIGH PRIORITY] Add capacity to Station S2
   - Current utilization at 92.3% is near saturation
   - Increase concurrency from 2 to 3, or add parallel station

2. [MEDIUM PRIORITY] Address congestion at Node 15
   - Consider alternate routing paths
   - Enable congestion-aware routing if not already

3. [LOW PRIORITY] Review Robot 7 assignment patterns
   - Unusual idle time suggests potential routing issue

============================================================

Analysis Summary:
  Health Score: 72/100
  Orders Analyzed: 245
  Primary Issue: Station Queue Wait
  Bottlenecks Found: 5
  Anomalies Detected: 3
```

---

## Analysis Components

### Health Score

Overall system health (0-100):

| Score | Interpretation |
|-------|----------------|
| 90-100 | Excellent - well optimized |
| 70-89 | Good - minor issues |
| 50-69 | Fair - significant bottlenecks |
| < 50 | Poor - major issues |

### Bottleneck Analysis

Identifies and ranks:

- Station bottlenecks (high queue, high utilization)
- Congestion hotspots (nodes and edges)
- Charging station bottlenecks
- Maintenance station bottlenecks

### Delay Attribution

Breaks down order completion time:

- **Travel time** - Time robots spend moving
- **Queue wait** - Time waiting in station queues
- **Service time** - Time being serviced at stations
- **Traffic wait** - Time waiting for traffic/congestion

### Anomaly Detection

Uses statistical methods to find:

- Unusual queue spikes
- Abnormal robot behavior
- Unexpected utilization patterns

---

## Output Formats

### Text (Default)

Human-readable report with sections and formatting.

### JSON

Machine-readable format for integration:

```json
{
  "summary": {
    "health_score": 72,
    "orders_analyzed": 245,
    "primary_delay_source": "Station Queue Wait",
    "anomaly_count": 3
  },
  "bottleneck_analysis": {
    "total_count": 5,
    "bottlenecks": [...]
  },
  "anomalies": [...],
  "recommendations": [...]
}
```

### Compact

Brief summary for quick review:

```
Health: 72/100 | Primary Issue: Station Queue Wait
Bottlenecks: 5 (2 HIGH, 2 MEDIUM, 1 LOW)
Anomalies: 3
Top Recommendation: Add capacity to Station S2
```

---

## Use Cases

### Post-Simulation Analysis

```bash
# Run simulation
waremax run --scenario scenario.yaml --output-dir ./results

# Analyze for bottlenecks
waremax analyze --scenario scenario.yaml --output rca_report.txt
```

### Configuration Optimization

```bash
# Analyze current config
waremax analyze --scenario current.yaml --detailed

# Make changes based on recommendations
# Edit configuration

# Re-analyze
waremax analyze --scenario improved.yaml --detailed
```

### Debugging Performance Issues

```bash
# Detailed analysis with low anomaly threshold
waremax analyze --scenario problematic.yaml \
  --detailed \
  --anomaly-threshold 1.5 \
  --format json \
  --output debug_analysis.json
```

---

## Best Practices

### Interpretation

- Focus on HIGH priority recommendations first
- Address bottlenecks in order of impact
- Re-run analysis after changes to verify improvement

### Thresholds

- Default anomaly threshold (2.0) is good for most cases
- Lower threshold (1.5) for sensitive detection
- Higher threshold (2.5+) to reduce false positives

### Regular Analysis

- Analyze after configuration changes
- Include in capacity planning workflow
- Track health score over time

---

## See Also

- [run](run.md) - Run simulations
- [Root Cause Analysis](../concepts/metrics/rca.md) - Concepts
- [Bottleneck Identification](../tutorials/analysis/bottleneck-identification.md) - Tutorial
