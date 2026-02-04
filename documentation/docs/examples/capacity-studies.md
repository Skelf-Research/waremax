# Capacity Studies

Examples of fleet sizing and capacity planning.

---

## Fleet Sizing Study

Determine optimal number of robots for a given throughput target.

### Scenario

```yaml
# fleet_sizing.yaml
simulation:
  duration_s: 3600

stations:
  - { id: S1, node: 30, type: pick, concurrency: 2 }
  - { id: S2, node: 31, type: pick, concurrency: 2 }

orders:
  generation:
    type: constant
    rate_per_hour: 500  # Target throughput
```

### Run Sweep

```bash
waremax sweep fleet_sizing.yaml \
  --param "robots.count=[5,10,15,20,25,30,35]" \
  --runs 3
```

### Results

| Robots | Throughput | Utilization | Task Time | Status |
|--------|------------|-------------|-----------|--------|
| 5 | 280/hr | 95% | 64s | Under capacity |
| 10 | 420/hr | 89% | 52s | Under capacity |
| 15 | 495/hr | 82% | 45s | Near target |
| 20 | 502/hr | 72% | 43s | Target met |
| 25 | 505/hr | 62% | 42s | Excess capacity |
| 30 | 498/hr | 52% | 44s | Congestion |
| 35 | 485/hr | 45% | 48s | Congestion |

### Analysis

```
Throughput vs Fleet Size:

500│           ┌───┬───┐
   │         ┌─┘   │   └─┐
400│       ┌─┘     │     └──
   │     ┌─┘       │
300│   ┌─┘         │
   │ ┌─┘           │
200│─┘             │
   └───────────────┴─────────
   5  10  15  20  25  30  35
              Robots

Peak throughput at 20 robots
Diminishing returns after 15
Congestion after 30
```

### Recommendation

**Optimal: 20 robots**

- Meets 500/hr target
- 72% utilization (healthy)
- Buffer for variability

---

## Station Capacity Study

Determine required station slots for target throughput.

### Scenario

```yaml
# station_capacity.yaml
robots:
  count: 20  # Fixed fleet

orders:
  generation:
    type: constant
    rate_per_hour: 600  # Target
```

### Run Sweep

```bash
waremax sweep station_capacity.yaml \
  --param "stations[0].concurrency=[1,2,3]" \
  --param "stations[1].concurrency=[1,2,3]"
```

### Results

| S1 Slots | S2 Slots | Total | Throughput | Avg Queue |
|----------|----------|-------|------------|-----------|
| 1 | 1 | 2 | 320/hr | 12.5 |
| 2 | 1 | 3 | 450/hr | 6.2 |
| 1 | 2 | 3 | 445/hr | 6.5 |
| 2 | 2 | 4 | 580/hr | 2.1 |
| 3 | 2 | 5 | 605/hr | 1.2 |
| 2 | 3 | 5 | 600/hr | 1.4 |
| 3 | 3 | 6 | 610/hr | 0.8 |

### Recommendation

**Optimal: 5 slots (3+2 or 2+3)**

- Meets 600/hr target
- Reasonable queue lengths
- Cost-effective

---

## Growth Planning Study

Plan capacity for future growth.

### Current State

- Throughput: 400/hr
- Robots: 10
- Stations: 4 slots

### Growth Targets

| Year | Target | Increase |
|------|--------|----------|
| Year 1 | 600/hr | +50% |
| Year 2 | 900/hr | +50% |
| Year 3 | 1200/hr | +33% |

### Capacity Matrix

```bash
waremax sweep growth_scenario.yaml \
  --param "orders.rate=[400,600,900,1200]" \
  --param "robots.count=[10,15,20,25,30,35,40]" \
  --param "total_station_slots=[4,6,8,10,12]"
```

### Results Summary

| Target | Min Robots | Min Slots | Recommended |
|--------|------------|-----------|-------------|
| 400/hr | 10 | 4 | Current |
| 600/hr | 15 | 5 | +5 robots, +1 slot |
| 900/hr | 22 | 7 | +7 robots, +2 slots |
| 1200/hr | 30 | 10 | +8 robots, +3 slots |

### Growth Plan

```
Year 0 (Current)
├── Robots: 10
├── Stations: 4 slots
└── Capacity: 400/hr

Year 1
├── Add: 5 robots, 1 station slot
├── Total: 15 robots, 5 slots
└── Capacity: 600/hr

Year 2
├── Add: 7 robots, 2 station slots
├── Total: 22 robots, 7 slots
└── Capacity: 900/hr

Year 3
├── Add: 8 robots, 3 station slots
├── Total: 30 robots, 10 slots
└── Capacity: 1200/hr
```

---

## Peak Load Study

Size for peak demand periods.

### Scenario

```yaml
# peak_load.yaml
simulation:
  duration_s: 14400  # 4 hours

orders:
  generation:
    type: variable
    schedule:
      - { time: 0, rate: 300 }      # Normal
      - { time: 3600, rate: 600 }   # Peak (1 hour)
      - { time: 7200, rate: 300 }   # Normal
```

### Run Sweep

```bash
waremax sweep peak_load.yaml \
  --param "robots.count=[15,20,25,30]"
```

### Results

| Robots | Normal Rate | Peak Rate | Queue @ Peak | Recovery |
|--------|-------------|-----------|--------------|----------|
| 15 | 300/hr OK | 450/hr | 45 tasks | 30 min |
| 20 | 300/hr OK | 550/hr | 20 tasks | 12 min |
| 25 | 300/hr OK | 600/hr | 5 tasks | 3 min |
| 30 | 300/hr OK | 600/hr | 0 tasks | 0 min |

### Recommendation

**Choose based on priorities:**

- **Cost-focused**: 20 robots (12 min recovery acceptable)
- **Service-focused**: 25 robots (minimal queue buildup)
- **Zero-queue**: 30 robots (over-provisioned for normal times)

---

## Charging Infrastructure Study

Size charging stations for fleet.

### Parameters

```yaml
battery:
  capacity_wh: 500
  consumption_rate_w: 50
  charge_rate_w: 200
  charge_threshold_pct: 20

robots:
  count: 20
```

### Calculations

```
Operating time: 500 × 0.8 / 50 = 8 hours
Charge time: 500 × 0.75 / 200 = 1.88 hours

Daily charges per robot: 24 / (8 + 1.88) ≈ 2.4
Total daily charges: 20 × 2.4 = 48

Required bay-hours: 48 × 1.88 = 90.2
With 24 hours: 90.2 / 24 = 3.76 bays minimum
```

### Sweep

```bash
waremax sweep charging_study.yaml \
  --param "charging_stations[0].bays=[2,3,4,5,6]" \
  --duration 86400  # 24 hours
```

### Results

| Bays | Avg Queue | Max Queue | Throughput Impact |
|------|-----------|-----------|-------------------|
| 2 | 3.5 | 8 | -5% |
| 3 | 1.8 | 5 | -2% |
| 4 | 0.6 | 3 | -0.5% |
| 5 | 0.2 | 2 | 0% |
| 6 | 0.1 | 1 | 0% |

### Recommendation

**4-5 bays** for 20 robots

- 4 bays: Minimal impact, occasional queues
- 5 bays: No throughput impact

---

## Related

- [Capacity Planning Tutorial](../tutorials/analysis/capacity-planning.md)
- [Benchmarking](../tutorials/testing/benchmarking.md)
- [Battery Management](../tutorials/config/battery-management.md)
