# Station Configuration

Configuration for pick, drop, and other stations.

---

## Schema

```yaml
stations:
  - id: <string>                   # Required
    node: <string>                 # Required
    type: <string>                 # Required
    concurrency: <integer>         # Default: 1
    queue_capacity: <integer>      # Optional (unlimited)
    service_time_s: <ServiceTime>  # Required
```

---

## Fields

### id

**Type**: string
**Required**: Yes

Unique station identifier.

```yaml
stations:
  - id: "pick_station_1"
```

**Best practices**:

- Use descriptive names
- Include zone or area if applicable
- Avoid spaces and special characters

### node

**Type**: string
**Required**: Yes

Map node where station is located.

```yaml
stations:
  - id: "S1"
    node: "N100"  # Must match a node ID in the map
```

### type

**Type**: string
**Required**: Yes
**Options**: `pick`, `drop`, `inbound`, `outbound`

Station type.

```yaml
stations:
  - id: "S1"
    node: "0"
    type: pick
```

| Type | Description |
|------|-------------|
| `pick` | Order picking station |
| `drop` | Order drop-off station |
| `inbound` | Receiving station |
| `outbound` | Shipping station |

### concurrency

**Type**: integer
**Default**: 1

Number of robots that can be serviced simultaneously.

```yaml
stations:
  - id: "S1"
    node: "0"
    type: pick
    concurrency: 3  # Service up to 3 robots at once
```

### queue_capacity

**Type**: integer
**Default**: unlimited

Maximum robots waiting in queue.

```yaml
stations:
  - id: "S1"
    node: "0"
    type: pick
    queue_capacity: 10
```

If not specified, queue is unlimited.

---

## Service Time Configuration

### Schema

```yaml
service_time_s:
  distribution: <string>           # "constant" | "lognormal" | "exponential" | "uniform"
  base: <float>                   # Base service time
  per_item: <float>               # Time per item
  # Lognormal specific:
  base_stddev: <float>
  per_item_stddev: <float>
  # Uniform specific:
  min_s: <float>
  max_s: <float>
```

### Constant Distribution

Fixed service time.

```yaml
service_time_s:
  distribution: constant
  base: 5.0       # 5 seconds base
  per_item: 2.0   # + 2 seconds per item
```

**Total time**: `base + (per_item × items)`

### Lognormal Distribution

Variable service time with right-skewed distribution.

```yaml
service_time_s:
  distribution: lognormal
  base: 8.0
  base_stddev: 2.0
  per_item: 2.0
  per_item_stddev: 0.5
```

**Realistic** - models human operator variability.

### Exponential Distribution

Memoryless service time distribution.

```yaml
service_time_s:
  distribution: exponential
  base: 10.0      # Mean service time
```

**Use for**: Queuing theory validation, simple models.

### Uniform Distribution

Service time uniformly distributed between min and max.

```yaml
service_time_s:
  distribution: uniform
  min_s: 3.0
  max_s: 10.0
  per_item: 1.5
```

---

## Examples

### Single Pick Station

```yaml
stations:
  - id: "pick_1"
    node: "0"
    type: pick
    concurrency: 2
    queue_capacity: 15
    service_time_s:
      distribution: constant
      base: 5.0
      per_item: 2.0
```

### Multiple Stations

```yaml
stations:
  - id: "pick_zone_a"
    node: "N100"
    type: pick
    concurrency: 3
    queue_capacity: 20
    service_time_s:
      distribution: lognormal
      base: 6.0
      base_stddev: 1.5
      per_item: 1.8
      per_item_stddev: 0.3

  - id: "pick_zone_b"
    node: "N200"
    type: pick
    concurrency: 3
    queue_capacity: 20
    service_time_s:
      distribution: lognormal
      base: 6.0
      base_stddev: 1.5
      per_item: 1.8
      per_item_stddev: 0.3

  - id: "drop_1"
    node: "N50"
    type: drop
    concurrency: 2
    service_time_s:
      distribution: constant
      base: 3.0
      per_item: 1.0
```

### High-Throughput Station

```yaml
stations:
  - id: "high_volume_pick"
    node: "N150"
    type: pick
    concurrency: 5
    queue_capacity: 30
    service_time_s:
      distribution: exponential
      base: 4.0
```

---

## Capacity Planning

### Estimating Station Capacity

Single station throughput:

```
Throughput = (concurrency × 3600) / avg_service_time_s
```

Example:

- Concurrency: 2
- Avg service time: 12s (5 base + 2/item × 3.5 items)
- Throughput: (2 × 3600) / 12 = 600 tasks/hour

### Queue Sizing

```
Recommended queue = 2 × concurrency × (order_rate / stations)
```

Example:

- 4 robots typically waiting + buffer
- queue_capacity: 10-15

---

## Related

- [Station Assignment Policy](policies.md#station-assignment)
- [Stations Concept](../concepts/warehouse/stations.md)
