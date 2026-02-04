# Finding Bottlenecks

Identify what's limiting your system's performance.

---

## Goal

By the end of this tutorial, you will:

- Identify system bottlenecks systematically
- Use metrics to pinpoint constraints
- Understand different bottleneck types
- Plan interventions effectively

**Time**: 30-45 minutes

---

## Prerequisites

- Completed [Analyzing Results](../basic/analyzing-results.md)
- Results from a simulation run

---

## Step 1: Run and Collect Data

Run a simulation with detailed metrics:

```bash
waremax run scenario.yaml \
  -o results/ \
  --metrics detailed
```

---

## Step 2: Quick Bottleneck Scan

Use the analyze command:

```bash
waremax analyze results/ --bottlenecks
```

**Output:**

```
=== Bottleneck Analysis ===

Primary Bottleneck: Station S1
  - Utilization: 95%
  - Avg queue: 6.2 robots
  - Throughput limited by service capacity

Secondary Bottleneck: Edge E15
  - Occupancy: 88%
  - Traffic congestion causing delays

Impact:
  - 23% of task time spent waiting for S1
  - 15% of task time in traffic delays
```

---

## Step 3: Identify Bottleneck Type

### Station Bottleneck

**Symptoms:**
```
Station utilization > 90%
Long station queues
Low overall throughput despite available robots
```

**Verification:**
```bash
waremax analyze results/ --focus stations
```

```
Station S1:
  Utilization: 95% ← Very high
  Avg queue: 6.2 ← Growing queue
  Max queue: 12
  Tasks processed: 850

Station S2:
  Utilization: 62% ← Underutilized
  Avg queue: 1.5
  Tasks processed: 520

Imbalance: S1 handles 62% more tasks than S2
```

---

### Traffic Bottleneck

**Symptoms:**
```
High wait times
Edge/node occupancy > 80%
Low robot utilization despite tasks available
```

**Verification:**
```bash
waremax analyze results/ --focus traffic
```

```
Traffic Hotspots:
  Node N15: 92% occupancy
  Node N22: 85% occupancy
  Edge E15: 88% occupancy

Wait Time Analysis:
  Avg traffic wait: 12.3s
  Peak traffic wait: 45s
  Time spent waiting: 28% of task time
```

---

### Robot Bottleneck

**Symptoms:**
```
Robot utilization > 90%
Tasks queued with no available robots
Idle time = 0
```

**Verification:**
```bash
waremax analyze results/ --focus robots
```

```
Fleet Analysis:
  All robots: 92-95% utilization
  Idle time: <1%
  Task queue: 15 tasks waiting

This indicates: Not enough robots for demand
```

---

### Demand Bottleneck

**Symptoms:**
```
Low robot utilization
Low station utilization
Robots often idle
```

**Verification:**
```
Fleet utilization: 45%
Station utilization: 38%
Throughput: Equal to order arrival rate

This indicates: System capacity exceeds demand
```

---

## Step 4: Deep Dive Analysis

### Time Breakdown

```bash
waremax analyze results/ --breakdown task-time
```

```
Task Time Breakdown:
  Component       Time    %
  ─────────────────────────────
  Queue wait      3.2s    7%
  Travel time     15.8s   35%
  Traffic wait    10.5s   23%  ← High
  Station queue   8.2s    18%  ← High
  Service time    7.5s    17%
  ─────────────────────────────
  Total           45.2s   100%

Largest delays:
  1. Traffic wait (23%)
  2. Station queue (18%)
```

### Location Analysis

```bash
waremax analyze results/ --breakdown location
```

```
Congestion by Location:

Top 5 Congested Nodes:
  N15 (intersection):  92% occupancy, 8.5s avg wait
  N22 (station area):  85% occupancy, 6.2s avg wait
  N8 (crossing):       78% occupancy, 4.1s avg wait
  N30 (aisle):         72% occupancy, 3.5s avg wait
  N12 (junction):      68% occupancy, 2.8s avg wait
```

---

## Step 5: Visualize the Bottleneck

### Heat Map View

```
Warehouse Traffic Heat Map:

   ░░░░░░░░░░░░░░░░
   ░░░░░▒▒▒▒░░░░░░░
   ░░░▒▒▓▓▓▓▒▒░░░░░
   ░░▒▓▓████▓▓▒░░░░  S1 area
   ░░░▒▒▓▓▓▓▒▒░░░░░
   ░░░░░▒▒▒▒░░░░░░░
   ░░░░░░░░░░░░░░░░

   ░ = Low    ▓ = High
   ▒ = Med    █ = Bottleneck
```

### Time Series

```
Queue Length at S1 Over Time:

12│            ████
  │          ██████████
 8│        ████████████████
  │    ████████████████████████
 4│ ████████████████████████████
  │████████████████████████████████
 0└─────────────────────────────────
   0   15   30   45   60  75  90 min

Queue builds during peak, never clears
```

---

## Step 6: Confirm with Experiment

Test your hypothesis:

### Station Bottleneck Fix

```bash
# Increase S1 capacity
waremax run scenario.yaml \
  --param stations.S1.concurrency=3 \
  -o results_fixed/

waremax analyze results/ results_fixed/
```

```
Comparison:
                Original    Fixed      Change
Throughput      850/hr      1,020/hr   +20%
S1 utilization  95%         68%        -27%
Avg task time   45.2s       38.1s      -16%

Confirmed: S1 was the primary bottleneck
```

---

## Step 7: Systematic Search

When bottleneck isn't obvious:

```bash
waremax analyze results/ --systematic-bottleneck
```

```
Systematic Bottleneck Search:

1. Theoretical max throughput analysis:
   - Station capacity: 1,200/hr
   - Robot capacity: 900/hr ← Limiting
   - Traffic capacity: 1,100/hr

2. Constraint identification:
   Primary: Robot count (fleet too small)
   Secondary: Traffic at N15

3. Utilization analysis:
   - Robots: 95% (saturated)
   - Stations: 72% (headroom)
   - Edges: 65% (headroom)

Conclusion: Add more robots to increase throughput
```

---

## Step 8: Bottleneck Resolution

### Quick Reference

| Bottleneck | Fix Options |
|------------|-------------|
| Station | Add concurrency, add station, reduce service time |
| Traffic | Widen paths, add routes, improve routing policy |
| Robots | Add robots, improve utilization |
| Layout | Restructure map, move stations |

### Priority Matrix

```
Impact    Easy to Fix    Hard to Fix
───────────────────────────────────
High      Fix now        Plan fix
Low       Maybe fix      Don't fix
```

---

## Common Bottleneck Patterns

### The Hidden Bottleneck

```
Symptom: Low throughput, nothing obviously saturated

Cause: Multiple small bottlenecks compound

Solution: Look at time breakdown, fix biggest first
```

### The Shifting Bottleneck

```
Symptom: Fix one issue, another appears

Cause: Sequential bottlenecks

Solution: Analyze system holistically, balance capacity
```

### The Interaction Bottleneck

```
Symptom: Components fine individually, slow together

Cause: Traffic conflicts, resource contention

Solution: Analyze interactions, improve coordination
```

---

## Next Steps

- [Root Cause Analysis](root-cause-analysis.md): Deeper investigation
- [Capacity Planning](capacity-planning.md): Size system properly
- [Tuning Policies](../config/tuning-policies.md): Optimize behavior
