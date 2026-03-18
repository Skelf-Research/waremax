# Root Cause Analysis

Finding the underlying causes of performance issues.

---

## What is RCA?

Root Cause Analysis (RCA) goes beyond symptoms to find fundamental issues:

```
Symptom:     Low throughput
Immediate:   Long task times
Root cause:  Station S1 understaffed during peak hours
```

---

## RCA Process

### 1. Identify Symptoms

What's the observable problem?

```
Symptoms:
- Throughput 20% below target
- Tasks taking longer than expected
- Robots frequently idle
```

### 2. Gather Data

Collect relevant metrics:

```bash
waremax run scenario.yaml -o results/
waremax analyze results/
```

### 3. Trace Back

Follow the chain of causation:

```
Low throughput
    ↓
Long task times
    ↓
High station wait times
    ↓
Station S1 queue length = 8 (max = 3 others)
    ↓
S1 concurrency = 1 (others = 2)
    ↓
ROOT CAUSE: S1 under-provisioned
```

### 4. Validate

Confirm hypothesis:

```bash
# Test fix
waremax run scenario.yaml \
  --param stations.S1.concurrency=2 \
  -o results_fixed/

# Compare
waremax analyze results/ results_fixed/
```

---

## Common Root Causes

### Throughput Issues

| Symptom | Check | Possible Root Cause |
|---------|-------|---------------------|
| Low throughput | Station queues | Under-provisioned stations |
| Low throughput | Robot utilization | Too few robots |
| Low throughput | Travel times | Poor layout / routing |
| Low throughput | Congestion | Too many robots |

### Time Issues

| Symptom | Check | Possible Root Cause |
|---------|-------|---------------------|
| Long task times | Wait time breakdown | Traffic congestion |
| Long task times | Travel distances | Far storage locations |
| Long task times | Service times | Slow stations |
| Variable times | Queue lengths | Unbalanced station load |

### Utilization Issues

| Symptom | Check | Possible Root Cause |
|---------|-------|---------------------|
| Low robot utilization | Task arrival rate | Insufficient demand |
| Low robot utilization | Charging time | Battery issues |
| High robot utilization + low throughput | Wait times | Congestion |

---

## Analysis Techniques

### Time Breakdown

Decompose task time:

```
Total Task Time: 45.0s (100%)
├── Queue wait:    5.2s (12%)
├── Travel time:  18.3s (41%)
├── Traffic wait:  8.7s (19%)
├── Station queue: 7.5s (17%)
└── Service time:  5.3s (12%)

Biggest contributor: Travel time (41%)
→ Investigate layout/slotting
```

### Bottleneck Identification

Find constraining resources:

```
Resource Utilization:
├── Robot R1-R10:  75-85%
├── Station S1:    95% ← Bottleneck
├── Station S2:    62%
├── Station S3:    58%
└── Edge E5:       88% ← Secondary

Primary bottleneck: Station S1
```

### Correlation Analysis

Find relationships:

```
When orders spike:
  → Station queues grow
  → Wait times increase
  → Throughput drops

Correlation: Order rate ↔ Queue length: 0.89
```

---

## Diagnostic Questions

### Throughput Diagnosis

```
Q: Is throughput meeting target?
├── No → Q: Where is time being spent?
│        ├── Travel → Check layout, routing
│        ├── Station queues → Check station capacity
│        ├── Traffic waits → Check congestion
│        └── Other → Investigate specifics
└── Yes → System performing well
```

### Congestion Diagnosis

```
Q: Is there congestion?
├── Yes → Q: Where?
│         ├── Everywhere → Too many robots or poor layout
│         ├── Specific spots → Add capacity or bypass
│         └── Time-based → Schedule optimization
└── No → Congestion not the issue
```

---

## Example RCA Walkthrough

### Scenario

Target throughput: 1,000 tasks/hour
Actual throughput: 780 tasks/hour (22% below target)

### Step 1: Symptoms

```
- Throughput: 780/hr vs. 1,000/hr target
- Avg task time: 52s vs. 38s expected
- Some robots frequently idle
```

### Step 2: Data Gathering

```
Time Breakdown:
- Travel: 18s (35%)
- Traffic wait: 15s (29%)  ← Unusually high
- Station queue: 12s (23%)
- Service: 7s (13%)

Congestion:
- Avg occupancy: 45%
- Hotspot N15: 92% occupancy
- Hotspot N22: 85% occupancy
```

### Step 3: Trace Back

```
High wait time
    ↓
Congestion at N15 and N22
    ↓
N15 is main intersection near Station S1
N22 is approach to S1
    ↓
S1 has longest queue (avg 6 robots)
    ↓
All robots funnel through N15/N22 to reach S1
    ↓
ROOT CAUSE: S1 creates bottleneck and congestion
```

### Step 4: Solutions

```
Option A: Add station capacity (S1 concurrency 1→2)
Option B: Add second station (S4)
Option C: Reroute traffic (bypass N15)
```

### Step 5: Validate

```bash
# Test Option A
waremax run scenario.yaml \
  --param stations.S1.concurrency=2 \
  -o results_option_a/

# Results:
Throughput: 920/hr (+18%)
Wait time: 8s (-47%)
```

---

## RCA Tools in Waremax

### Analyze Command

```bash
waremax analyze results/
```

Provides:
- Summary statistics
- Bottleneck identification
- Hotspot detection

### Compare Command

```bash
waremax compare results_baseline/ results_modified/
```

Shows:
- Metric differences
- Improvement/regression
- Statistical significance

### Detailed Metrics

```yaml
metrics:
  detail_level: high
  include:
    - time_breakdown
    - congestion_by_location
    - utilization_by_resource
```

---

## Best Practices

### Start with Symptoms

Don't guess at root causes. Follow the data.

### One Change at a Time

When testing fixes, change one thing:

```
Good:  Test adding 1 station
Bad:   Test adding station + changing policy + more robots
```

### Validate Fixes

Always simulate proposed solution before implementing.

### Document Findings

Record RCA for future reference:

```
Issue: Low throughput (780/hr)
Root cause: Station S1 bottleneck causing congestion
Solution: Added S1 concurrency 1→2
Result: Throughput 920/hr (+18%)
```

---

## Related

- [KPIs](kpis.md)
- [Congestion Metrics](congestion-metrics.md)
- [Analysis Command](../../cli/analyze.md)
