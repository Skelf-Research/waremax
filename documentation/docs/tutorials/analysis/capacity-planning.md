# Capacity Planning

Size your warehouse system for target throughput.

---

## Goal

By the end of this tutorial, you will:

- Calculate required fleet size
- Determine station capacity needs
- Plan for peak load and growth
- Validate capacity with simulation

**Time**: 45 minutes

---

## Prerequisites

- Completed [Benchmarking](../testing/benchmarking.md)
- Understanding of throughput concepts

---

## Step 1: Define Requirements

Start with business requirements:

```
Requirements:
  Target throughput: 1,500 orders/hour
  Operating hours: 16 hours/day
  Peak factor: 1.3× average
  Growth buffer: 20%
  SLA: 95% orders in <60s
```

Calculate design capacity:

```
Design capacity = Target × Peak factor × Growth buffer
                = 1,500 × 1.3 × 1.2
                = 2,340 orders/hour
```

---

## Step 2: Baseline Measurement

Measure current capacity:

```bash
waremax benchmark scenario.yaml --find-max-throughput
```

```
Current Configuration:
  Robots: 10
  Stations: 2 (concurrency 2 each)

Maximum Throughput: 1,050/hr
Limiting Factor: Robot fleet

Capacity gap: 2,340 - 1,050 = 1,290/hr needed
```

---

## Step 3: Fleet Sizing

### Theoretical Calculation

```
Tasks per robot = Throughput / Robot count
               = 1,050 / 10
               = 105 tasks/robot/hour

For 2,340/hr target:
  Robots needed = 2,340 / 105 = 22.3
  Round up: 23 robots
  Add margin: 25 robots
```

### Validate with Simulation

```bash
waremax sweep scenario.yaml \
  --param "robots.count=[20,22,25,28,30]" \
  --target-throughput 2340
```

```
robots.count  Max Throughput  Meets Target?
20            1,850/hr        No
22            2,100/hr        No
25            2,380/hr        ✓ (margin: 2%)
28            2,650/hr        ✓ (margin: 13%)
30            2,580/hr        ✓ (congestion)

Optimal: 25-28 robots
```

---

## Step 4: Station Capacity

### Calculate Station Needs

```
Service rate per slot = 3600 / avg_service_time
                      = 3600 / 5s
                      = 720 tasks/hour per slot

Total slots needed = Target / Service rate
                   = 2,340 / 720
                   = 3.25 slots

Current: 2 stations × 2 slots = 4 slots ✓
```

### Validate with Simulation

```bash
waremax sweep scenario.yaml \
  --param robots.count=25 \
  --param "stations[*].concurrency=[1,2,3]" \
  --target-throughput 2340
```

```
Total Slots  Max Throughput  Station Util
2 (1+1)      1,420/hr        98%
4 (2+2)      2,380/hr        82%
6 (3+3)      2,420/hr        55%

4 slots (2+2) is sufficient
```

---

## Step 5: Traffic Capacity

### Edge Capacity Analysis

```bash
waremax analyze results/ --traffic-capacity
```

```
Edge Capacity Analysis:

Critical Edges:
  E15: 92% utilized (at risk)
  E22: 85% utilized
  E8:  78% utilized

For 2,340/hr (2.2× current):
  E15 would need: 92% × 2.2 = 202% ← Over capacity!

Action: Increase E15 capacity or add parallel path
```

### Solution

```yaml
# Increase edge capacity
edges:
  - from: 14
    to: 15
    capacity: 2  # Was 1
```

Validate:

```bash
waremax run scenario_wide_edge.yaml \
  --param robots.count=25 \
  -o results_wide/
```

---

## Step 6: Peak Load Planning

### Define Peak Scenarios

```yaml
# peak_scenario.yaml
orders:
  generation:
    type: variable
    schedule:
      - time: 0
        rate: 1500    # Normal
      - time: 3600
        rate: 2340    # Peak (1.5× for 1 hour)
      - time: 7200
        rate: 1500    # Return to normal
```

### Test Peak Handling

```bash
waremax run peak_scenario.yaml \
  --param robots.count=25 \
  -o peak_results/
```

```
Peak Period Analysis (hour 2):
  Order rate: 2,340/hr
  Throughput: 2,280/hr
  Queue buildup: 60 orders
  Recovery time: 12 minutes

Result: System handles peak with minor queue buildup
```

---

## Step 7: Growth Planning

### Capacity Curves

```bash
waremax sweep scenario.yaml \
  --param "orders.rate=[1000,1500,2000,2500,3000]" \
  --param "robots.count=[15,20,25,30,35]" \
  -o capacity_matrix/
```

```
Capacity Matrix (Throughput):

Order Rate    15 robots  20 robots  25 robots  30 robots
1,000         980        1,000      1,000      1,000
1,500         1,450      1,500      1,500      1,500
2,000         1,850      2,000      2,000      2,000
2,500         2,100      2,420      2,500      2,500
3,000         2,250      2,680      2,920      3,000

Scaling recommendations:
  - 1,500/hr: 20 robots adequate
  - 2,500/hr: 25 robots needed
  - 3,000/hr: 30 robots + station upgrade
```

### Growth Timeline

```
Current:        1,000/hr (10 robots)
Year 1 target:  1,500/hr → Add 10 robots
Year 2 target:  2,500/hr → Add 5 robots + station
Year 3 target:  3,500/hr → Major expansion
```

---

## Step 8: Create Capacity Plan

```markdown
# Capacity Plan

## Current State
- Throughput capacity: 1,050/hr
- Robots: 10
- Stations: 2 (concurrency 2)

## Target State (Design)
- Throughput capacity: 2,340/hr
- Design factor: 1.3 peak × 1.2 growth

## Changes Required

### Fleet
| Current | Target | Change |
|---------|--------|--------|
| 10      | 25     | +15    |

### Stations
| Current | Target | Change |
|---------|--------|--------|
| 4 slots | 4 slots| None   |

### Infrastructure
| Item | Current | Target | Change |
|------|---------|--------|--------|
| Edge E15 | Cap 1 | Cap 2 | Widen |

### Charging
| Current | Target | Change |
|---------|--------|--------|
| 4 bays  | 8 bays | +4     |

## Validation Results
- Simulated throughput: 2,380/hr ✓
- Peak handling: OK ✓
- SLA compliance: 96% ✓

## Timeline
1. Phase 1: Add 10 robots, widen E15
2. Phase 2: Add 5 robots, 4 charging bays
3. Monitor and adjust
```

---

## Step 9: Validate Complete Plan

Run full validation:

```bash
waremax run capacity_plan_scenario.yaml \
  --duration 86400 \   # 24-hour test
  --runs 5 \
  -o capacity_validation/
```

```
Capacity Validation (24-hour simulation):

Throughput:
  Average: 2,320/hr
  Peak hour: 2,580/hr
  Minimum: 2,180/hr

SLA Performance:
  Orders < 60s: 96.2%
  Orders < 90s: 99.1%

Resource Utilization:
  Robots: 72%
  Stations: 78%
  Charging: 65%

Verdict: Capacity plan validated ✓
```

---

## Capacity Planning Formulas

### Quick Estimates

**Fleet size:**
```
Robots ≈ (Target throughput × Avg task time) / 3600
       ≈ (2,340 × 50) / 3600
       ≈ 32.5 → 35 robots (with margin)
```

**Station slots:**
```
Slots ≈ Target throughput / (3600 / Avg service time)
     ≈ 2,340 / (3600 / 5)
     ≈ 3.25 → 4 slots
```

**Charging bays:**
```
Bays ≈ Fleet × Charging time / (Operating time + Charging time)
    ≈ 25 × 2 / (8 + 2)
    ≈ 5 → 6 bays (with margin)
```

---

## Best Practices

### Include Margins

```
Production capacity = Design capacity × Safety margin
                    = Theoretical × 1.1 to 1.2
```

### Plan for Variability

```
Peak handling: 1.3× to 1.5× average
Daily variation: Consider high and low periods
```

### Validate Before Implementing

```
Always simulate full configuration
Test peak scenarios
Verify SLA compliance
```

---

## Next Steps

- [Benchmarking](../testing/benchmarking.md): Measure limits
- [Parameter Sweeps](../testing/parameter-sweeps.md): Optimize configuration
- [Configuration Reference](../../configuration/index.md): Full options
