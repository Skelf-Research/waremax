# Battery Management

Configure power consumption and charging behavior.

---

## Goal

By the end of this tutorial, you will:

- Configure battery parameters
- Set up charging stations
- Tune charging thresholds
- Monitor battery health

**Time**: 30 minutes

---

## Prerequisites

- Completed [Creating Scenarios](../basic/creating-scenarios.md)
- Understanding of robot configuration

---

## Step 1: Basic Battery Configuration

Add battery settings to robots:

```yaml
battery:
  # Battery capacity
  capacity_wh: 500

  # Initial charge
  initial_soc_pct: 100

  # Power consumption
  consumption_rate_w: 50       # While operating
  idle_consumption_rate_w: 10  # While idle

  # Charging settings
  charge_rate_w: 200
  charge_threshold_pct: 20     # Go charge below 20%
  charge_target_pct: 95        # Charge until 95%
```

---

## Step 2: Add Charging Stations

Create charging station nodes and configure them:

```yaml
map:
  nodes:
    # ... other nodes ...
    - { id: 50, name: "CH1", x: 0, y: 0, type: "charging" }
    - { id: 51, name: "CH2", x: 0, y: 10, type: "charging" }

charging_stations:
  - id: "CH1"
    node: 50
    bays: 2          # 2 robots can charge simultaneously
    charge_rate_w: 200
    queue_capacity: 5

  - id: "CH2"
    node: 51
    bays: 1
    charge_rate_w: 300  # Faster charger
    queue_capacity: 3
```

---

## Step 3: Calculate Battery Life

Estimate how long robots can operate:

```
Operating time = Capacity × SoC / Consumption rate

Example:
  Capacity: 500 Wh
  Starting SoC: 100%
  Consumption: 50 W

  Time to 20%: 500 × 0.8 / 50 = 8 hours
```

Charging time:

```
Charge time = Capacity × (Target - Current) / Charge rate

Example:
  Capacity: 500 Wh
  Current: 20%
  Target: 95%
  Charge rate: 200 W

  Time: 500 × 0.75 / 200 = 1.875 hours (112.5 min)
```

---

## Step 4: Tune Charge Threshold

The charge threshold affects utilization:

**Low threshold (10%):**
```
+ More operating time before charging
- Risk of stranding
- Deep discharge stress on battery
```

**High threshold (40%):**
```
+ Safe margin
+ Better for battery health
- More time charging
- Lower effective capacity
```

Test different thresholds:

```bash
waremax sweep scenario.yaml \
  --param "battery.charge_threshold_pct=[10,15,20,25,30]"
```

**Example results:**

```
Threshold  Utilization  Charge Events  Throughput
10%        82%          45             950
15%        80%          52             940
20%        78%          61             925
25%        74%          73             890
30%        70%          88             850
```

---

## Step 5: Enable Opportunistic Charging

Charge during idle periods:

```yaml
battery:
  charge_threshold_pct: 20
  charge_target_pct: 95

  # Opportunistic charging
  opportunistic_charging: true
  opportunistic_threshold_pct: 80  # Charge if idle and below 80%
```

This keeps batteries topped up without dedicated charging trips.

---

## Step 6: Configure Charging Station Selection

How robots choose which station:

```yaml
policies:
  charging:
    station_selection: nearest      # Or: shortest_queue, fastest
```

Compare policies:

```bash
waremax compare scenario.yaml \
  --param policies.charging.station_selection=nearest \
  --param policies.charging.station_selection=shortest_queue
```

---

## Step 7: Monitor Battery Metrics

Enable battery metrics:

```yaml
metrics:
  battery:
    enabled: true
    track_distribution: true
    low_soc_threshold: 15
```

Run and analyze:

```bash
waremax run scenario.yaml -o results/
waremax analyze results/ --focus battery
```

**Output:**

```
=== Battery Analysis ===

Fleet Battery Summary:
  Average SoC: 65%
  Min SoC observed: 12%
  Max SoC observed: 98%

Charging Statistics:
  Total charge events: 145
  Avg charge duration: 42 min
  Avg SoC at charge start: 18%

Distribution (end of simulation):
  0-20%:   ██ (2 robots)
  20-40%:  ████ (4 robots)
  40-60%:  ████████ (8 robots)
  60-80%:  ██████ (6 robots)
  80-100%: ██████████ (10 robots)

Warnings:
  ⚠️ 3 instances of SoC below 15%
```

---

## Step 8: Size Charging Infrastructure

Calculate required capacity:

```
Fleet: 20 robots
Operating time before charge: 8 hours
Charge time: 2 hours
Shifts per day: 3 (24 hours)

Charges per robot per day: 24 / (8 + 2) = 2.4
Total charges per day: 20 × 2.4 = 48

If charging takes 2 hours:
Required bay-hours: 48 × 2 = 96
With 24 hours available: 96 / 24 = 4 bays minimum
Add margin: 5-6 bays recommended
```

Test with simulation:

```bash
waremax sweep scenario.yaml \
  --param "charging_stations[0].bays=[2,3,4,5,6]" \
  --duration 86400  # 24-hour simulation
```

---

## Complete Configuration

```yaml
# Complete battery configuration
simulation:
  duration_s: 28800  # 8 hours

robots:
  count: 20
  speed_m_s: 1.5

battery:
  # Capacity
  capacity_wh: 500
  initial_soc_pct: 100

  # Consumption
  consumption_rate_w: 50
  idle_consumption_rate_w: 10

  # Charging behavior
  charge_threshold_pct: 20
  charge_target_pct: 95

  # Opportunistic
  opportunistic_charging: true
  opportunistic_threshold_pct: 80

charging_stations:
  - id: "CH1"
    node: 50
    bays: 3
    charge_rate_w: 200
    queue_capacity: 5

  - id: "CH2"
    node: 51
    bays: 3
    charge_rate_w: 200
    queue_capacity: 5

policies:
  charging:
    station_selection: shortest_queue

metrics:
  battery:
    enabled: true
    track_distribution: true
```

---

## Best Practices

### Threshold Selection

| Scenario | Recommended Threshold |
|----------|----------------------|
| Short shifts (<4 hours) | 15% |
| Full shifts (8 hours) | 20% |
| Critical operations | 25% |
| Battery health priority | 30% |

### Station Placement

- Distribute across warehouse
- Place near high-traffic areas
- Avoid creating bottlenecks

### Monitoring

Watch for:
- Robots frequently below threshold
- Long charging queues
- Uneven station utilization

---

## Troubleshooting

### Robots Running Out of Charge

```
Causes:
- Threshold too low
- Not enough charging capacity
- Consumption rate too high

Solutions:
- Raise charge_threshold_pct
- Add charging bays
- Check for unexpected consumption
```

### Long Charging Queues

```
Causes:
- Not enough bays
- Uneven station selection
- All robots charging at once

Solutions:
- Add bays
- Use shortest_queue policy
- Stagger initial SoC
```

---

## Next Steps

- [Battery Configuration](../../configuration/battery.md): Full reference
- [Maintenance](../../concepts/robots/maintenance.md): Combined battery + maintenance
