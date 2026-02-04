# Maintenance & Failures Configuration

Configuration for scheduled maintenance and random failures.

---

## Robot Maintenance Schema

```yaml
robots:
  maintenance:
    enabled: <boolean>             # Default: false
    interval_hours: <float>       # Default: 8.0

  failure:
    enabled: <boolean>             # Default: false
    mtbf_hours: <float>           # Default: 100.0
```

## Maintenance Station Schema

```yaml
maintenance_stations:
  - id: <string>                   # Required
    node: <string>                 # Required
    bays: <integer>               # Default: 2
    maintenance_duration_s: <float> # Default: 300
    repair_time: <ServiceTimeConfig> # Optional
    queue_capacity: <integer>     # Optional
```

---

## Scheduled Maintenance

### maintenance.enabled

**Type**: boolean
**Default**: false

Enable scheduled maintenance.

```yaml
robots:
  maintenance:
    enabled: true
```

### maintenance.interval_hours

**Type**: float
**Default**: 8.0

Hours between scheduled maintenance visits.

```yaml
robots:
  maintenance:
    enabled: true
    interval_hours: 6.0  # Every 6 hours
```

**Typical values**:

| Scenario | Interval (hours) |
|----------|------------------|
| High reliability | 12-24 |
| Standard | 6-8 |
| Intensive use | 4-6 |

---

## Random Failures

### failure.enabled

**Type**: boolean
**Default**: false

Enable random failures.

```yaml
robots:
  failure:
    enabled: true
```

### failure.mtbf_hours

**Type**: float
**Default**: 100.0

Mean Time Between Failures in hours.

```yaml
robots:
  failure:
    enabled: true
    mtbf_hours: 80.0
```

**MTBF Guidelines**:

| Reliability | MTBF (hours) |
|-------------|--------------|
| High | 150-200+ |
| Normal | 80-120 |
| Low (stress testing) | 30-50 |

---

## Maintenance Stations

### id

**Type**: string
**Required**: Yes

Unique identifier.

### node

**Type**: string
**Required**: Yes

Map node location.

### bays

**Type**: integer
**Default**: 2

Concurrent repair capacity.

```yaml
maintenance_stations:
  - id: "maint_1"
    node: "N75"
    bays: 3
```

### maintenance_duration_s

**Type**: float
**Default**: 300

Duration for scheduled maintenance in seconds.

```yaml
maintenance_stations:
  - id: "maint_1"
    node: "N75"
    maintenance_duration_s: 600  # 10 minutes
```

### repair_time

**Type**: ServiceTimeConfig
**Optional**

Variable repair time for failures (uses same schema as station service times).

```yaml
maintenance_stations:
  - id: "maint_1"
    node: "N75"
    maintenance_duration_s: 300
    repair_time:
      distribution: lognormal
      base: 600       # Mean 10 minutes
      base_stddev: 180  # Significant variance
```

### queue_capacity

**Type**: integer
**Default**: unlimited

Maximum robots waiting for maintenance.

---

## Examples

### Basic Maintenance

```yaml
robots:
  count: 20
  max_speed_mps: 1.5
  maintenance:
    enabled: true
    interval_hours: 8.0

maintenance_stations:
  - id: "maint_1"
    node: "N75"
    bays: 2
    maintenance_duration_s: 300
```

### With Random Failures

```yaml
robots:
  count: 20
  max_speed_mps: 1.5
  maintenance:
    enabled: true
    interval_hours: 8.0
  failure:
    enabled: true
    mtbf_hours: 100.0

maintenance_stations:
  - id: "maint_zone_1"
    node: "N50"
    bays: 2
    maintenance_duration_s: 300
    repair_time:
      distribution: lognormal
      base: 900       # 15 min mean repair
      base_stddev: 300
    queue_capacity: 6
```

### Multiple Maintenance Stations

```yaml
robots:
  count: 50
  max_speed_mps: 1.8
  maintenance:
    enabled: true
    interval_hours: 6.0
  failure:
    enabled: true
    mtbf_hours: 80.0

maintenance_stations:
  - id: "maint_east"
    node: "N100"
    bays: 3
    maintenance_duration_s: 240
    repair_time:
      distribution: lognormal
      base: 720
      base_stddev: 240
    queue_capacity: 8

  - id: "maint_west"
    node: "N200"
    bays: 3
    maintenance_duration_s: 240
    repair_time:
      distribution: lognormal
      base: 720
      base_stddev: 240
    queue_capacity: 8
```

### Reliability Stress Test

```yaml
robots:
  count: 30
  max_speed_mps: 1.5
  maintenance:
    enabled: true
    interval_hours: 4.0    # Frequent maintenance
  failure:
    enabled: true
    mtbf_hours: 40.0       # Frequent failures

maintenance_stations:
  - id: "maint_main"
    node: "N75"
    bays: 5
    maintenance_duration_s: 180
    repair_time:
      distribution: exponential
      base: 600
    queue_capacity: 15
```

---

## Capacity Planning

### Maintenance Station Sizing

Estimate required maintenance capacity:

```
Scheduled visits/hour = robots / interval_hours
Failure repairs/hour = robots / mtbf_hours
Total/hour = scheduled + repairs
Required bays ≈ total/hour × avg_service_time_hours × 1.2
```

### Example

- 20 robots
- 8-hour maintenance interval
- 100-hour MTBF
- 5-minute scheduled maintenance
- 15-minute average repair

```
Scheduled: 20/8 = 2.5/hour × 5/60 = 0.21 bay-hours
Failures: 20/100 = 0.2/hour × 15/60 = 0.05 bay-hours
Total: 0.26 bay-hours → 1 bay minimum, 2 for buffer
```

---

## Behavior

### Scheduled Maintenance Flow

1. Robot reaches maintenance interval
2. Completes current task
3. Travels to maintenance station
4. Waits in queue if necessary
5. Receives maintenance (fixed duration)
6. Returns to service

### Failure Flow

1. Failure occurs (exponentially distributed)
2. Robot stops immediately
3. Travels to maintenance station (if mobile)
4. Waits in queue if necessary
5. Receives repair (variable duration)
6. Returns to service

---

## Related

- [Robot Configuration](robots.md)
- [Maintenance Concepts](../concepts/robots/maintenance.md)
