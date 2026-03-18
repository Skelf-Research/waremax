# Root Cause Analysis

Systematically diagnose performance issues.

---

## Goal

By the end of this tutorial, you will:

- Follow a structured RCA methodology
- Trace symptoms to root causes
- Validate hypotheses with data
- Document findings for action

**Time**: 45 minutes

---

## Prerequisites

- Completed [Finding Bottlenecks](finding-bottlenecks.md)
- Results from a problematic simulation

---

## Step 1: Define the Problem

Start with a clear problem statement:

```
Problem: Throughput is 780/hr, target is 1,000/hr
Gap: 22% below target
Impact: Cannot meet order volume requirements
```

---

## Step 2: Gather Data

Collect comprehensive metrics:

```bash
# Run with full metrics
waremax run scenario.yaml -o rca_data/ --metrics full

# Generate analysis report
waremax analyze rca_data/ --full-report > analysis.txt
```

Key data to collect:

- [ ] Overall throughput
- [ ] Task time breakdown
- [ ] Resource utilization
- [ ] Queue lengths
- [ ] Wait times
- [ ] Traffic patterns

---

## Step 3: Start with Symptoms

List observable symptoms:

```
Observed Symptoms:
1. Throughput: 780/hr (target 1,000)
2. Average task time: 58s (expected 42s)
3. Robot utilization: 68% (seems low)
4. Station S1 queue: Often 8+ robots
5. Frequent waits near Node N15
```

---

## Step 4: Ask "Why?" Repeatedly

Use the 5 Whys technique:

```
Why is throughput low?
→ Because task times are high (58s vs 42s)

Why are task times high?
→ Because robots spend 35% of time waiting

Why are robots waiting so long?
→ Because of congestion near Station S1

Why is there congestion near S1?
→ Because all paths to S1 converge at Node N15

Why do all paths converge at N15?
→ Because there's only one approach to S1

ROOT CAUSE: Single access point to Station S1 creates bottleneck
```

---

## Step 5: Build a Causal Chain

Visualize the cause-effect relationships:

```
                    Low Throughput
                          ↑
                   High Task Times
                          ↑
                   Long Wait Times
                     ↑         ↑
          Station Queue    Traffic Congestion
                 ↑              ↑
         High S1 Demand    Single Path to S1
                 ↑              ↑
         Imbalanced Assignment    Layout Design
                      ↑
              ╔═══════════════════╗
              ║   ROOT CAUSES     ║
              ║ 1. Layout limits  ║
              ║ 2. Station policy ║
              ╚═══════════════════╝
```

---

## Step 6: Validate with Data

Test each hypothesis:

### Hypothesis 1: Station queue is excessive

```bash
waremax analyze rca_data/ --focus stations
```

```
Station S1:
  Utilization: 92%
  Avg queue: 6.8 (high)
  Max queue: 14

Station S2:
  Utilization: 58%
  Avg queue: 1.2

CONFIRMED: S1 overloaded while S2 underutilized
```

### Hypothesis 2: Traffic congestion at N15

```bash
waremax analyze rca_data/ --focus "node:N15"
```

```
Node N15:
  Occupancy: 89%
  Avg wait: 8.5s
  Robots passing: 1,250/hr
  Capacity: 1 robot

CONFIRMED: N15 is severely congested
```

### Hypothesis 3: Single path to S1

```bash
waremax analyze rca_data/ --paths-to "S1"
```

```
Paths to Station S1:
  Path 1: ... → N15 → S1  (85% of traffic)
  Path 2: ... → N15 → S1  (15% of traffic)

All paths go through N15

CONFIRMED: N15 is single point of access
```

---

## Step 7: Quantify Impact

Measure contribution of each factor:

```
Factor Analysis:

Factor                 Contribution to Delay
─────────────────────────────────────────────
Station S1 queue       35% (6.8s avg wait)
Traffic at N15         28% (8.5s avg wait)
Travel distance        22% (base travel time)
Service time           15% (normal)
─────────────────────────────────────────────

Primary factors: S1 queue + N15 traffic = 63%
```

---

## Step 8: Identify Root Causes

Distinguish symptoms from causes:

```
Symptom: Long wait at S1
Intermediate: High demand + limited capacity
Root Cause: Station assignment policy sends too much to S1

Symptom: Traffic congestion at N15
Intermediate: Too many robots in one area
Root Cause: Map layout has single access point

Symptom: Low overall throughput
Result: Compound effect of above root causes
```

---

## Step 9: Test Solutions

Experiment with fixes:

### Fix 1: Better station assignment

```bash
waremax run scenario.yaml \
  --param policies.station_assignment=shortest_queue \
  -o fix1_results/

waremax analyze rca_data/ fix1_results/
```

```
Result:
  S1 utilization: 92% → 75%
  S2 utilization: 58% → 72%
  Throughput: 780 → 850 (+9%)
```

### Fix 2: Add second path to S1

```bash
# Modify map to add alternative route
waremax run scenario_alt_path.yaml -o fix2_results/

waremax analyze rca_data/ fix2_results/
```

```
Result:
  N15 occupancy: 89% → 52%
  Avg wait: 8.5s → 3.2s
  Throughput: 780 → 920 (+18%)
```

### Fix 3: Combine both fixes

```bash
waremax run scenario_combined.yaml -o fix3_results/

waremax analyze rca_data/ fix3_results/
```

```
Result:
  Throughput: 780 → 1,050 (+35%)
  Task time: 58s → 41s (-29%)
  Target achieved! ✓
```

---

## Step 10: Document Findings

Create an RCA report:

```markdown
# Root Cause Analysis Report

## Problem Statement
Throughput: 780/hr (target: 1,000/hr)
Gap: 22% below target

## Investigation Summary

### Symptoms Observed
1. High task times (58s vs 42s expected)
2. Station S1 queue length (avg 6.8)
3. Traffic congestion at Node N15 (89% occupancy)
4. Low robot utilization (68%)

### Root Causes Identified
1. **Station assignment policy** sends 62% of traffic to S1
   - S1 over capacity, S2 under-utilized
   - Policy: "nearest" favors S1 due to layout

2. **Map layout** has single access point to S1
   - All traffic funnels through N15
   - Creates traffic bottleneck

### Causal Chain
Layout → Single path → N15 congestion
Policy → S1 overload → Queue buildup
Both → High wait time → Low throughput

## Solution
1. Change station assignment to "shortest_queue"
2. Add alternative path to S1 bypassing N15

## Results After Fix
- Throughput: 780 → 1,050 (+35%)
- Task time: 58s → 41s (-29%)
- Target exceeded

## Recommendations
1. Implement combined fix immediately
2. Monitor queue lengths as early warning
3. Review layout for similar bottlenecks
```

---

## RCA Checklist

- [ ] Problem clearly defined
- [ ] Data collected
- [ ] Symptoms listed
- [ ] 5 Whys completed
- [ ] Causal chain mapped
- [ ] Hypotheses validated
- [ ] Impact quantified
- [ ] Root causes identified
- [ ] Solutions tested
- [ ] Report documented

---

## Common RCA Mistakes

### Stopping at Symptoms

```
Bad: "Fix the queue at S1"
Good: "Why is there a queue at S1?"
```

### Single-Cause Thinking

```
Bad: "The problem is X"
Good: "The problem has multiple causes: X, Y, Z"
```

### Untested Assumptions

```
Bad: "S1 is the bottleneck"
Good: "S1 utilization is 92%, confirming bottleneck"
```

---

## Next Steps

- [Capacity Planning](capacity-planning.md): Size for future growth
- [Tuning Policies](../config/tuning-policies.md): Optimize behavior
- [Custom Maps](../config/custom-maps.md): Improve layout
