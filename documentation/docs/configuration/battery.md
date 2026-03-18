# Battery & Charging Configuration

Configuration for battery simulation and charging stations.

---

## Robot Battery Schema

```yaml
robots:
  battery:
    enabled: <boolean>             # Default: false
    capacity_wh: <float>          # Default: 400
    min_soc: <float>              # Default: 0.15
    consumption:
      per_meter_wh: <float>       # Default: 0.1
      per_kg_per_meter_wh: <float> # Default: 0.01
      idle_power_w: <float>       # Default: 5.0
      service_power_w: <float>    # Default: 20.0
```

## Charging Station Schema

```yaml
charging_stations:
  - id: <string>                   # Required
    node: <string>                 # Required
    bays: <integer>               # Default: 1
    charge_rate_w: <float>        # Default: 200
    queue_capacity: <integer>     # Optional
```

---

## Battery Configuration

### enabled

**Type**: boolean
**Default**: false

Enable battery simulation.

```yaml
robots:
  battery:
    enabled: true
```

### capacity_wh

**Type**: float
**Default**: 400

Battery capacity in watt-hours.

```yaml
robots:
  battery:
    enabled: true
    capacity_wh: 500
```

**Typical values**:

| Robot Size | Capacity (Wh) |
|------------|---------------|
| Small | 200-300 |
| Medium | 400-600 |
| Large | 600-1000 |

### min_soc

**Type**: float
**Default**: 0.15

Minimum state of charge before robot must charge.

```yaml
robots:
  battery:
    min_soc: 0.20  # Charge when below 20%
```

**Range**: 0.0 to 1.0 (0% to 100%)

---

## Consumption Model

### per_meter_wh

**Type**: float
**Default**: 0.1

Energy consumed per meter of travel (empty).

```yaml
robots:
  battery:
    consumption:
      per_meter_wh: 0.08
```

### per_kg_per_meter_wh

**Type**: float
**Default**: 0.01

Additional energy per kg of payload per meter.

```yaml
robots:
  battery:
    consumption:
      per_kg_per_meter_wh: 0.012
```

**Example**: Robot carrying 20kg traveling 100m:

```
Energy = (0.08 + 0.012 × 20) × 100 = 32 Wh
```

### idle_power_w

**Type**: float
**Default**: 5.0

Power consumption when idle (watts).

```yaml
robots:
  battery:
    consumption:
      idle_power_w: 3.0
```

### service_power_w

**Type**: float
**Default**: 20.0

Power consumption during station service (watts).

```yaml
robots:
  battery:
    consumption:
      service_power_w: 15.0
```

---

## Charging Station Configuration

### id

**Type**: string
**Required**: Yes

Unique charging station identifier.

### node

**Type**: string
**Required**: Yes

Map node where charging station is located.

### bays

**Type**: integer
**Default**: 1

Number of charging bays (robots that can charge simultaneously).

```yaml
charging_stations:
  - id: "charger_1"
    node: "N50"
    bays: 4  # Can charge 4 robots at once
```

### charge_rate_w

**Type**: float
**Default**: 200

Charging power per bay in watts.

```yaml
charging_stations:
  - id: "charger_1"
    node: "N50"
    charge_rate_w: 300  # Fast charger
```

**Charge time calculation**:

```
Time (hours) = (capacity_wh × (1 - current_soc)) / charge_rate_w
```

### queue_capacity

**Type**: integer
**Default**: unlimited

Maximum robots waiting to charge.

```yaml
charging_stations:
  - id: "charger_1"
    node: "N50"
    bays: 2
    queue_capacity: 6
```

---

## Examples

### Basic Battery Setup

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
      idle_power_w: 5.0
      service_power_w: 20.0

charging_stations:
  - id: "charger_1"
    node: "N50"
    bays: 2
    charge_rate_w: 200
```

### Multiple Charging Stations

```yaml
robots:
  count: 25
  max_speed_mps: 1.8
  battery:
    enabled: true
    capacity_wh: 500
    min_soc: 0.18
    consumption:
      per_meter_wh: 0.08
      per_kg_per_meter_wh: 0.01
      idle_power_w: 4.0
      service_power_w: 18.0

charging_stations:
  - id: "charger_zone_a"
    node: "N100"
    bays: 3
    charge_rate_w: 250
    queue_capacity: 6

  - id: "charger_zone_b"
    node: "N200"
    bays: 3
    charge_rate_w: 250
    queue_capacity: 6

  - id: "fast_charger"
    node: "N150"
    bays: 2
    charge_rate_w: 400
    queue_capacity: 4
```

### Conservative Battery Setup

```yaml
robots:
  count: 15
  max_speed_mps: 1.5
  battery:
    enabled: true
    capacity_wh: 600
    min_soc: 0.25  # Higher threshold for safety
    consumption:
      per_meter_wh: 0.12
      per_kg_per_meter_wh: 0.015
      idle_power_w: 6.0
      service_power_w: 25.0

charging_stations:
  - id: "main_charger"
    node: "N75"
    bays: 5
    charge_rate_w: 200
    queue_capacity: 10
```

---

## Capacity Planning

### Charging Station Sizing

Estimate required charging capacity:

```
Required bays ≈ (robots × avg_charge_time) / operating_hours
```

Where:

- `avg_charge_time`: Time to charge from min_soc to full
- `operating_hours`: Hours before robots need recharge

### Example Calculation

- 20 robots
- 400 Wh battery, charge from 15% to 100%
- 200 W charger
- Avg charge time: (400 × 0.85) / 200 = 1.7 hours
- Operating time per charge: ~4 hours

```
Required bays ≈ (20 × 1.7) / 4 ≈ 8.5 → 9 bays
```

---

## Related

- [Robot Configuration](robots.md)
- [Battery Management Concepts](../concepts/robots/battery.md)
