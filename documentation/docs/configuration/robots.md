# Robot Configuration

Configuration for the robot fleet.

---

## Schema

```yaml
robots:
  count: <integer>                 # Required
  max_speed_mps: <float>          # Required
  max_payload_kg: <float>         # Default: 25
  battery: <BatteryConfig>        # Optional
  maintenance: <MaintenanceConfig> # Optional
  failure: <FailureConfig>        # Optional
```

---

## Basic Fields

### count

**Type**: integer
**Required**: Yes

Number of robots in the fleet.

```yaml
robots:
  count: 10
```

**Guidelines**:

| Warehouse Size | Typical Robots |
|----------------|----------------|
| Small (< 100 nodes) | 5-15 |
| Medium (100-500 nodes) | 15-50 |
| Large (500+ nodes) | 50-200 |

### max_speed_mps

**Type**: float
**Required**: Yes

Maximum robot speed in meters per second.

```yaml
robots:
  max_speed_mps: 1.5  # 1.5 m/s = 5.4 km/h
```

**Typical values**:

| Robot Type | Speed (m/s) |
|------------|-------------|
| Conservative | 0.8-1.2 |
| Standard | 1.2-1.8 |
| Fast | 1.8-2.5 |

### max_payload_kg

**Type**: float
**Default**: 25

Maximum payload capacity in kilograms.

```yaml
robots:
  count: 10
  max_speed_mps: 1.5
  max_payload_kg: 30
```

---

## Battery Configuration

Enable battery simulation for charging station analysis.

```yaml
robots:
  count: 10
  max_speed_mps: 1.5
  battery:
    enabled: true
    capacity_wh: 400
    min_soc: 0.15
    consumption:
      per_meter_wh: 0.1
      per_kg_per_meter_wh: 0.01
      idle_power_w: 5.0
      service_power_w: 20.0
```

### battery.enabled

**Type**: boolean
**Default**: false

Enable battery simulation.

### battery.capacity_wh

**Type**: float
**Default**: 400

Battery capacity in watt-hours.

### battery.min_soc

**Type**: float
**Default**: 0.15

Minimum state of charge (0-1) before robot must charge.

```yaml
battery:
  min_soc: 0.20  # Robot charges when below 20%
```

### battery.consumption

Power consumption parameters:

| Field | Default | Description |
|-------|---------|-------------|
| `per_meter_wh` | 0.1 | Energy per meter traveled |
| `per_kg_per_meter_wh` | 0.01 | Additional energy per kg payload per meter |
| `idle_power_w` | 5.0 | Power when idle |
| `service_power_w` | 20.0 | Power during service at stations |

---

## Maintenance Configuration

Enable scheduled maintenance.

```yaml
robots:
  count: 10
  max_speed_mps: 1.5
  maintenance:
    enabled: true
    interval_hours: 8.0
```

### maintenance.enabled

**Type**: boolean
**Default**: false

Enable scheduled maintenance.

### maintenance.interval_hours

**Type**: float
**Default**: 8.0

Hours between scheduled maintenance visits.

```yaml
maintenance:
  enabled: true
  interval_hours: 4.0  # Maintenance every 4 hours
```

---

## Failure Configuration

Enable random failures.

```yaml
robots:
  count: 10
  max_speed_mps: 1.5
  failure:
    enabled: true
    mtbf_hours: 100.0
```

### failure.enabled

**Type**: boolean
**Default**: false

Enable random failures.

### failure.mtbf_hours

**Type**: float
**Default**: 100.0

Mean Time Between Failures in hours.

```yaml
failure:
  enabled: true
  mtbf_hours: 50.0  # More frequent failures for testing
```

**MTBF Guidelines**:

| Reliability | MTBF (hours) |
|-------------|--------------|
| High | 200+ |
| Normal | 80-150 |
| Low (testing) | 20-50 |

---

## Examples

### Basic Fleet

```yaml
robots:
  count: 10
  max_speed_mps: 1.5
```

### Fleet with Battery

```yaml
robots:
  count: 15
  max_speed_mps: 1.8
  max_payload_kg: 30
  battery:
    enabled: true
    capacity_wh: 500
    min_soc: 0.20
    consumption:
      per_meter_wh: 0.08
      per_kg_per_meter_wh: 0.008
      idle_power_w: 3.0
      service_power_w: 15.0
```

### Fleet with Maintenance

```yaml
robots:
  count: 20
  max_speed_mps: 1.5
  max_payload_kg: 25
  maintenance:
    enabled: true
    interval_hours: 8.0
  failure:
    enabled: true
    mtbf_hours: 100.0
```

### Full Configuration

```yaml
robots:
  count: 25
  max_speed_mps: 1.8
  max_payload_kg: 35
  battery:
    enabled: true
    capacity_wh: 600
    min_soc: 0.18
    consumption:
      per_meter_wh: 0.09
      per_kg_per_meter_wh: 0.01
      idle_power_w: 4.0
      service_power_w: 18.0
  maintenance:
    enabled: true
    interval_hours: 6.0
  failure:
    enabled: true
    mtbf_hours: 80.0
```

---

## Related

- [Battery & Charging](battery.md)
- [Maintenance & Failures](maintenance.md)
- [Robot Operations](../concepts/robots/index.md)
