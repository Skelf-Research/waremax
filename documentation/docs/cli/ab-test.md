# waremax ab-test

Run A/B test with statistical significance testing.

---

## Synopsis

```bash
waremax ab-test --baseline <PATH> --variant <PATH> [OPTIONS]
```

---

## Description

The `ab-test` command performs a rigorous statistical comparison between two configurations. It uses Welch's t-test to determine if differences are statistically significant.

---

## Options

### Required

| Option | Description |
|--------|-------------|
| `--baseline` | Path to baseline scenario file |
| `--variant` | Path to variant scenario file |

### Optional

| Option | Default | Description |
|--------|---------|-------------|
| `--replications` | 10 | Replications per variant |
| `--alpha` | 0.05 | Significance level |
| `--output` | None | Output file for results (JSON) |

---

## Examples

### Basic A/B test

```bash
waremax ab-test \
  --baseline baseline.yaml \
  --variant variant.yaml
```

### Higher confidence

```bash
waremax ab-test \
  --baseline baseline.yaml \
  --variant variant.yaml \
  --alpha 0.01
```

### More replications

```bash
waremax ab-test \
  --baseline baseline.yaml \
  --variant variant.yaml \
  --replications 20
```

### Save results

```bash
waremax ab-test \
  --baseline baseline.yaml \
  --variant variant.yaml \
  --replications 15 \
  --output ab_results.json
```

---

## Output

### Console Output

```
Running A/B test...
  Baseline: baseline.yaml
  Variant: variant.yaml
  Replications: 10
  Alpha: 0.05

Running A/B test (10 replications per variant)...

A/B Test Results
================

Metric              Baseline        Variant         Diff      p-value   Significant
-------------------------------------------------------------------------------------
Throughput (ord/hr) 198.5 ± 12.3    267.8 ± 8.7     +34.9%    0.0001    YES ***
P95 Cycle Time (s)  78.5 ± 5.2      65.3 ± 4.1      -16.8%    0.0023    YES **
Robot Utilization   67.2% ± 3.1%    58.4% ± 2.8%    -13.1%    0.0089    YES **
Station Utilization 72.5% ± 4.5%    68.2% ± 3.2%    -5.9%     0.0521    NO

Conclusion: Variant shows SIGNIFICANT improvement in throughput
  - 34.9% higher throughput (p < 0.001)
  - 16.8% lower P95 cycle time (p < 0.01)
```

### Significance Indicators

| Indicator | Meaning |
|-----------|---------|
| `***` | p < 0.001 (highly significant) |
| `**` | p < 0.01 (very significant) |
| `*` | p < 0.05 (significant) |
| (none) | Not significant at alpha level |

---

## Statistical Method

### Welch's t-test

The A/B test uses Welch's t-test, which:

- Does not assume equal variances
- Is robust to unequal sample sizes
- Provides p-values for hypothesis testing

### Hypothesis

- **Null hypothesis (H0)**: No difference between baseline and variant
- **Alternative hypothesis (H1)**: There is a difference

### Interpretation

- **p < alpha**: Reject null hypothesis, difference is significant
- **p >= alpha**: Cannot reject null hypothesis, difference may be due to chance

---

## Choosing Parameters

### Replications

| Use Case | Recommended |
|----------|-------------|
| Quick check | 5-10 |
| Standard testing | 10-15 |
| Important decisions | 20-30 |
| Publication | 30+ |

### Alpha Level

| Alpha | Confidence | Use Case |
|-------|------------|----------|
| 0.10 | 90% | Exploratory testing |
| 0.05 | 95% | Standard (default) |
| 0.01 | 99% | Important decisions |
| 0.001 | 99.9% | Critical decisions |

---

## Use Cases

### Policy Evaluation

```bash
# Test new task allocation policy
waremax ab-test \
  --baseline current_policy.yaml \
  --variant new_policy.yaml \
  --replications 20 \
  --alpha 0.01 \
  --output policy_test.json
```

### Capacity Change

```bash
# Test adding more robots
waremax ab-test \
  --baseline 10_robots.yaml \
  --variant 15_robots.yaml \
  --replications 15
```

### Configuration Change

```bash
# Test routing algorithm change
waremax ab-test \
  --baseline dijkstra_routing.yaml \
  --variant astar_routing.yaml \
  --replications 10
```

---

## Best Practices

### Sample Size

- More replications = more statistical power
- Aim for at least 10 replications per variant
- Use more when differences are expected to be small

### Effect Size

Consider practical significance, not just statistical:

- A statistically significant 0.1% improvement may not matter
- Focus on meaningful effect sizes for your use case

### Multiple Comparisons

If testing many variants:

- Use Bonferroni correction (alpha / number of comparisons)
- Or use sweep + ab-test on promising candidates

---

## JSON Output

```json
{
  "baseline": {
    "scenario": "baseline.yaml",
    "replications": 10,
    "throughput": {
      "values": [195.2, 201.3, 198.7, ...],
      "mean": 198.5,
      "std_dev": 12.3
    }
  },
  "variant": {
    "scenario": "variant.yaml",
    "replications": 10,
    "throughput": {
      "values": [262.4, 271.8, 268.2, ...],
      "mean": 267.8,
      "std_dev": 8.7
    }
  },
  "tests": {
    "throughput": {
      "t_statistic": 5.82,
      "p_value": 0.0001,
      "significant": true,
      "effect_size_pct": 34.9
    },
    "p95_cycle_time": {
      "t_statistic": -3.45,
      "p_value": 0.0023,
      "significant": true,
      "effect_size_pct": -16.8
    }
  },
  "summary": {
    "winner": "variant",
    "confidence": "high",
    "recommendation": "Variant shows significant improvement"
  }
}
```

---

## See Also

- [compare](compare.md) - Quick comparison without significance testing
- [sweep](sweep.md) - Test multiple parameter values
- [benchmark](benchmark.md) - Performance benchmarking
