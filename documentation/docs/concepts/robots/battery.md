# Battery Management

Power consumption, charging, and battery lifecycle.

---

## Battery Model

### State of Charge (SoC)

Battery level as percentage (0-100%):

```
SoC = current_charge / capacity × 100%
```

### Key Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `capacity_wh` | Total energy capacity | 500 Wh |
| `initial_soc` | Starting charge level | 100% |
| `consumption_rate_w` | Power draw while operating | 50 W |
| `charge_rate_w` | Charging power | 200 W |

---

## Power Consumption

### Consumption During Operation

Battery drains during:

- **Traveling**: Higher consumption
- **Idle**: Lower consumption
- **Working**: Variable

### Consumption Formula

```
energy_used = power × time
SoC_new = SoC_old - (energy_used / capacity)
```

### Example

Robot traveling for 60 seconds at 50W:

```
Energy = 50W × 60s = 3000 Ws = 0.833 Wh
If capacity = 500 Wh:
  SoC drop = 0.833 / 500 = 0.17%
```

---

## Charging

### Charge Time

Time to fully charge:

```
charge_time = capacity × (1 - current_soc) / charge_rate
```

### Example

Battery at 20%, capacity 500 Wh, charge rate 200W:

```
Energy needed = 500 × (1 - 0.20) = 400 Wh
Time = 400 Wh / 200 W = 2 hours
```

### Charging Process

```
Battery: [████████████░░░░░░░░] 60%
         ↓ Charging at 200W
         ...
Battery: [████████████████████] 100%
```

---

## Charging Triggers

### Threshold-Based

Charge when SoC drops below threshold:

```yaml
battery:
  charge_threshold_pct: 20
```

Robot automatically goes to charge at 20% SoC.

### Opportunistic

Charge during idle periods:

```yaml
battery:
  opportunistic_charging: true
  opportunistic_threshold_pct: 80
```

Charge when idle and below 80%.

### Scheduled

Charge at specific times or intervals.

---

## Charging Station Behavior

### Station Selection

How robots choose charging station:

| Policy | Behavior |
|--------|----------|
| `nearest` | Closest station |
| `shortest_queue` | Least congested |
| `highest_capacity` | Fastest charging |

### Queue Management

```
Charging Station CS1 (3 bays):
  Bay 1: [R5] charging (45%)
  Bay 2: [R8] charging (72%)
  Bay 3: [R2] charging (95%)
  Queue: [R11] → [R3]
```

---

## Battery Impact on Operations

### Task Interruption

Low battery may interrupt tasks:

```
Task T1: Traveling...
  → Battery at 18% (< 20% threshold)
  → Task paused
  → Robot goes to charge
  → Resume task after charging
```

### Task Feasibility

Check if robot can complete task:

```
Required: Travel (10m) + Work (60s) + Return (10m)
Energy needed: ~15 Wh
Current SoC: 25% = 125 Wh available
  → Task feasible ✓
```

### Range Anxiety

Conservative threshold prevents stranding:

```
Safe threshold = max_task_energy + reserve
```

---

## Configuration Example

```yaml
battery:
  # Battery capacity
  capacity_wh: 500

  # Initial state
  initial_soc_pct: 100

  # Consumption
  consumption_rate_w: 50
  idle_consumption_rate_w: 10

  # Charging triggers
  charge_threshold_pct: 20
  charge_target_pct: 95

  # Opportunistic charging
  opportunistic_charging: true
  opportunistic_threshold_pct: 80
```

---

## Battery Metrics

### Key Performance Indicators

| Metric | Description |
|--------|-------------|
| Average SoC | Mean battery level across fleet |
| Charge events | Number of charging sessions |
| Charge time ratio | Time charging / Total time |
| Deep discharge events | Times SoC < critical level |

### Fleet Battery Distribution

```
SoC Distribution:
  0-20%:  ██ (5 robots)
  20-40%: ████ (10 robots)
  40-60%: ████████ (20 robots)
  60-80%: ██████ (15 robots)
  80-100%: ████████████ (30 robots)
```

---

## Best Practices

### Threshold Selection

- **Too low** (10%): Risk of stranding
- **Too high** (50%): Excessive charging, reduced utilization
- **Recommended**: 15-25%

### Charging Infrastructure

- **Station count**: Enough for peak demand
- **Station placement**: Distributed across warehouse
- **Bay capacity**: Multiple bays reduce queuing

### Monitoring

Track these indicators:

- Robots frequently below threshold
- Long charging queues
- Tasks delayed due to charging

---

## Related

- [Battery Configuration](../../configuration/battery.md)
- [Charging Stations](../warehouse/stations.md#charging-stations)
- [Robot States](index.md)
