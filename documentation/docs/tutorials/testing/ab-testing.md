# A/B Testing

Statistically compare configurations with confidence.

---

## Goal

By the end of this tutorial, you will:

- Design proper A/B experiments
- Run statistically valid comparisons
- Interpret statistical significance
- Make data-driven decisions

**Time**: 30-45 minutes

---

## Prerequisites

- Completed [Parameter Sweeps](parameter-sweeps.md)
- Basic understanding of statistics (helpful)

---

## Step 1: Why A/B Testing?

Simple comparisons can be misleading:

```
Run 1: Config A = 950, Config B = 980
→ B is better? Maybe just random variation.

Run 10 each:
Config A: 950, 962, 948, 971, 955... (mean: 958)
Config B: 980, 945, 968, 952, 975... (mean: 964)
→ Is 6-point difference real or noise?
```

A/B testing answers: **Is the difference statistically significant?**

---

## Step 2: Basic A/B Test

Compare two configurations:

```bash
waremax ab-test scenario.yaml \
  --control "policies.task_allocation=nearest_idle" \
  --treatment "policies.task_allocation=least_busy" \
  --runs 20
```

**Output:**

```
=== A/B Test Results ===

Control (nearest_idle):
  Throughput: 952 ± 28 tasks/hr
  Runs: 20

Treatment (least_busy):
  Throughput: 938 ± 32 tasks/hr
  Runs: 20

Comparison:
  Difference: -14 tasks/hr (-1.5%)
  95% CI: [-32, +4]
  p-value: 0.12

Conclusion: No significant difference (p > 0.05)
  The observed difference is likely due to random variation.
```

---

## Step 3: Interpret Results

### Key Statistics

| Statistic | Meaning |
|-----------|---------|
| Mean ± std | Average and spread |
| Difference | Treatment - Control |
| 95% CI | Range of likely true difference |
| p-value | Probability difference is by chance |

### Decision Rules

```
p < 0.05:  Significant difference (reject null hypothesis)
p ≥ 0.05:  No significant difference (cannot reject null)

95% CI:
  - Excludes 0: Significant
  - Includes 0: Not significant
```

---

## Step 4: Design Good Experiments

### Sample Size

More runs = more confidence:

```bash
# Minimum viable
waremax ab-test ... --runs 10

# Standard
waremax ab-test ... --runs 20

# High confidence
waremax ab-test ... --runs 50
```

### Power Analysis

Calculate needed sample size:

```bash
waremax ab-test ... \
  --power-analysis \
  --expected-effect 0.05 \  # 5% difference
  --power 0.8               # 80% chance to detect
```

**Output:**

```
Power Analysis:
  To detect 5% effect with 80% power:
  Minimum runs per group: 32
```

---

## Step 5: Multiple Metrics

Test multiple outcomes:

```bash
waremax ab-test scenario.yaml \
  --control "routing.policy=shortest_path" \
  --treatment "routing.policy=congestion_aware" \
  --metrics throughput,avg_task_time,wait_time \
  --runs 30
```

**Output:**

```
=== A/B Test Results (Multiple Metrics) ===

Metric          Control    Treatment    Diff      p-value  Sig?
───────────────────────────────────────────────────────────────
Throughput      952 ± 28   985 ± 25     +3.5%     0.002    ✓
Avg task time   48.2 ± 3.1 45.8 ± 2.8   -5.0%     0.008    ✓
Wait time       8.5 ± 1.2  6.2 ± 1.0    -27.0%    <0.001   ✓

Summary: Treatment (congestion_aware) significantly better
         on all measured metrics.
```

---

## Step 6: Handle Multiple Comparisons

When testing multiple metrics, adjust for false positives:

```bash
waremax ab-test scenario.yaml \
  --control "config_a" \
  --treatment "config_b" \
  --metrics throughput,task_time,utilization,wait_time \
  --correction bonferroni \  # Adjust p-values
  --runs 30
```

**Bonferroni correction:**

```
Adjusted α = 0.05 / 4 = 0.0125

Metric       p-value   Adjusted sig?
Throughput   0.02      No (0.02 > 0.0125)
Task time    0.008     Yes
Utilization  0.15      No
Wait time    0.001     Yes
```

---

## Step 7: Sequential Testing

Stop early when result is clear:

```bash
waremax ab-test scenario.yaml \
  --control "old_config" \
  --treatment "new_config" \
  --sequential \
  --max-runs 100 \
  --early-stopping
```

**Output:**

```
Sequential A/B Test:

Run 10: Inconclusive (continue)
Run 20: Inconclusive (continue)
Run 30: Treatment winning (p=0.08, continue)
Run 40: Treatment significantly better (p=0.02)

Stopped early at run 40 (max was 100)
Saved 60% of runs while maintaining statistical validity.
```

---

## Step 8: Real-World Example

Compare routing policies:

```bash
# Define configurations
cat > control.yaml << EOF
routing:
  policy: shortest_path
EOF

cat > treatment.yaml << EOF
routing:
  policy: congestion_aware
  congestion_weight: 1.5
EOF

# Run A/B test
waremax ab-test base_scenario.yaml \
  --control control.yaml \
  --treatment treatment.yaml \
  --metrics throughput,p95_task_time \
  --runs 30 \
  -o ab_results/
```

**Detailed analysis:**

```bash
waremax analyze ab_results/ --ab-test
```

```
=== Detailed A/B Analysis ===

Throughput:
  Control:   952 ± 28 (min: 901, max: 1008)
  Treatment: 998 ± 24 (min: 955, max: 1042)

  Effect size (Cohen's d): 0.51 (medium)
  95% CI for difference: [28, 64]
  p-value: 0.0003

  Distribution overlap: 32%
  Probability treatment > control: 89%

Recommendation: Strong evidence that congestion_aware
routing improves throughput by 4-5%.
```

---

## Step 9: Document Results

Create an A/B test report:

```markdown
# A/B Test Report: Routing Policy

## Hypothesis
Congestion-aware routing improves throughput compared
to shortest-path routing.

## Setup
- Base scenario: standard.yaml
- Control: shortest_path routing
- Treatment: congestion_aware routing (weight=1.5)
- Runs: 30 per group
- Duration: 1 hour per run

## Results
| Metric | Control | Treatment | Change | p-value |
|--------|---------|-----------|--------|---------|
| Throughput | 952±28 | 998±24 | +4.8% | 0.0003 |
| P95 time | 85±8 | 72±6 | -15.3% | <0.001 |
| Wait time | 8.5±1.2 | 5.8±0.9 | -31.8% | <0.001 |

## Conclusion
Congestion-aware routing significantly improves all
measured metrics. Recommend adoption.

## Next Steps
- Test with higher traffic loads
- Tune congestion_weight parameter
```

---

## Common Pitfalls

### Not Enough Runs

```
Bad:  3 runs each (too few for statistics)
Good: 20+ runs each
```

### Peeking at Results

```
Bad:  Stop when result looks good
Good: Pre-define sample size, stick to it
      OR use sequential testing properly
```

### Ignoring Variance

```
Bad:  Only report means
Good: Report means, std, confidence intervals
```

### Multiple Testing

```
Bad:  Test 10 metrics, report only significant ones
Good: Pre-specify metrics, apply corrections
```

---

## Next Steps

- [Benchmarking](benchmarking.md): Performance limits
- [A/B Test Command Reference](../../cli/ab-test.md)
- [Parameter Sweeps](parameter-sweeps.md): Explore more configurations
