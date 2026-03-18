# Robot Maintenance

Reliability, failures, and maintenance modeling.

---

## Maintenance Types

### Scheduled Maintenance

Regular preventive maintenance:

- Planned intervals
- Predictable downtime
- Prevents failures

### Unscheduled Maintenance

Reactive repairs after failures:

- Random occurrence
- Unpredictable downtime
- Emergency response

---

## Failure Model

### Mean Time Between Failures (MTBF)

Average operating time before failure:

```yaml
maintenance:
  mtbf_hours: 500  # Fails every ~500 hours on average
```

### Failure Distribution

Exponential distribution (memoryless):

```
P(failure in next Δt) = 1 - e^(-Δt/MTBF)
```

This models random failures where:

- Past operating time doesn't affect future failure probability
- Failures can happen at any time
- Rate is constant

---

## Repair Model

### Mean Time To Repair (MTTR)

Average repair duration:

```yaml
maintenance:
  mttr_hours: 2.0  # 2 hours average repair time
```

### Repair Time Distribution

Often lognormal (right-skewed):

```yaml
maintenance:
  mttr_hours: 2.0
  mttr_stddev_hours: 0.5
```

Most repairs are quick, some take longer.

---

## Scheduled Maintenance

### Interval-Based

Maintenance at regular intervals:

```yaml
maintenance:
  scheduled_interval_hours: 100
  scheduled_duration_hours: 1.0
```

### Condition-Based

Maintenance triggered by metrics:

- Operating hours since last maintenance
- Task count since last maintenance
- Degradation indicators

---

## Maintenance Process

### Failure Workflow

```
Robot Operating
       │
       ▼
   [Failure]
       │
       ▼
Travel to Maintenance Station
       │
       ▼
   Join Queue
       │
       ▼
  Receive Repair
       │
       ▼
Return to Service
```

### Scheduled Workflow

```
Maintenance Due
       │
       ▼
Complete Current Task
       │
       ▼
Travel to Maintenance Station
       │
       ▼
   Join Queue
       │
       ▼
Receive Maintenance
       │
       ▼
Return to Service
```

---

## Availability

### Calculation

```
Availability = MTBF / (MTBF + MTTR)
```

### Example

- MTBF = 500 hours
- MTTR = 2 hours

```
Availability = 500 / (500 + 2) = 99.6%
```

### Fleet Availability

With N robots:

```
Expected available = N × individual_availability
```

---

## Maintenance Station

### Configuration

```yaml
maintenance_stations:
  - id: "M1"
    node: "N50"
    bays: 2
    service_time_s: 3600  # 1 hour scheduled
```

### Queue Behavior

```
Maintenance Station M1 (2 bays):
  Bay 1: [R5] under repair (45 min remaining)
  Bay 2: [R8] scheduled maintenance (30 min remaining)
  Queue: [R11]
```

---

## Impact on Operations

### Reduced Fleet Capacity

During maintenance:

```
Available robots = Total - Charging - Maintenance
```

### Task Reassignment

When robot fails mid-task:

1. Task marked incomplete
2. Robot goes to maintenance
3. Task reassigned to another robot

### Throughput Impact

```
Effective throughput = Base throughput × Availability
```

---

## Configuration Example

```yaml
maintenance:
  # Failure model
  enabled: true
  mtbf_hours: 500

  # Repair model
  mttr_hours: 2.0
  mttr_stddev_hours: 0.5

  # Scheduled maintenance
  scheduled_maintenance: true
  scheduled_interval_hours: 100
  scheduled_duration_hours: 1.0

  # Station assignment
  station_selection: nearest
```

---

## Reliability Metrics

### Key Indicators

| Metric | Description |
|--------|-------------|
| MTBF | Mean time between failures |
| MTTR | Mean time to repair |
| Availability | Uptime percentage |
| Failure rate | Failures per hour |
| Maintenance time ratio | Time in maintenance / Total time |

### Fleet Statistics

```
Fleet Reliability Summary:
  Total robots: 50
  Failures this period: 12
  MTBF (observed): 480 hours
  MTTR (average): 1.8 hours
  Fleet availability: 99.4%
```

---

## Best Practices

### MTBF Estimation

Use historical data:

```
MTBF = Total operating hours / Number of failures
```

### Maintenance Station Capacity

Plan for peak demand:

```
Stations needed ≥ Fleet size × (1 - Availability)
```

### Scheduled Maintenance Timing

- Schedule during low-demand periods
- Stagger across fleet
- Don't overload maintenance stations

---

## Related

- [Maintenance Configuration](../../configuration/maintenance.md)
- [Maintenance Stations](../warehouse/stations.md#maintenance-stations)
- [Robot States](index.md)
