# Frequently Asked Questions

Common questions about Waremax.

---

## General

### What is Waremax?

Waremax is a discrete event simulation framework for warehouse robot operations. It models autonomous mobile robots performing pick, transport, and delivery tasks in a warehouse environment.

### What can I simulate with Waremax?

- Fleet sizing (how many robots needed)
- Throughput analysis
- Policy comparison (task allocation, routing)
- Layout optimization
- Capacity planning
- Performance benchmarking

### Is Waremax accurate?

Waremax uses proven discrete event simulation techniques. Accuracy depends on:

- Input data quality (layout, times)
- Configuration realism
- Appropriate modeling assumptions

Validate against real data when possible.

---

## Installation

### What are the system requirements?

- Rust 1.70 or later (for building from source)
- Any modern OS (Linux, macOS, Windows)
- 4GB RAM minimum (more for large simulations)

### How do I install Waremax?

**From source:**
```bash
git clone https://github.com/example/waremax.git
cd waremax
cargo install --path .
```

**Verify:**
```bash
waremax --version
```

### I get "command not found"

Ensure `~/.cargo/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add to your shell profile for persistence.

---

## Configuration

### Where do I start with configuration?

1. Use a preset: `waremax run --preset standard`
2. Generate a template: `waremax generate scenario > my_scenario.yaml`
3. Modify the template for your needs

### How do I validate my configuration?

```bash
waremax validate my_scenario.yaml
```

This checks for errors before running.

### What's the difference between presets?

| Preset | Duration | Robots | Use Case |
|--------|----------|--------|----------|
| minimal | 60s | 1 | Quick test |
| quick | 60s | 3 | Fast iteration |
| standard | 3600s | 10 | Typical analysis |
| high_load | 3600s | 25 | Stress testing |

### How do I add custom parameters?

Override at runtime:

```bash
waremax run scenario.yaml --param robots.count=20
```

---

## Simulation

### How long should my simulation run?

- **Quick tests**: 60-300 seconds
- **Analysis**: 3600 seconds (1 hour)
- **Statistical validity**: Multiple runs

Ensure steady state is reached (watch warmup period).

### Why are my results different each time?

Simulations include randomness. For reproducibility:

```yaml
simulation:
  seed: 12345  # Fixed seed
```

Or run multiple times and average.

### How do I run multiple simulations?

```bash
waremax run scenario.yaml --runs 10
```

Results include mean and standard deviation.

### Why is my simulation slow?

- Long duration: Use shorter for testing
- Many robots: Complexity scales with fleet size
- High event rate: More orders = more events

Try `--preset quick` for fast iteration.

---

## Analysis

### How do I interpret results?

Key metrics to check:

| Metric | Good | Warning |
|--------|------|---------|
| Robot utilization | 70-85% | >90% or <60% |
| Task completion | Meeting target | Below target |
| Wait time | <15% of task time | >25% |
| Deadlocks | 0 | >0 |

### How do I compare configurations?

```bash
waremax compare scenario.yaml \
  --param robots.count=10 \
  --param robots.count=20
```

### How do I find bottlenecks?

```bash
waremax analyze results/ --bottlenecks
```

Look for:
- High utilization resources
- Long queues
- Traffic congestion hotspots

---

## Performance

### How many robots can I simulate?

Tested with:
- 100+ robots
- 1000+ nodes
- Multi-hour simulations

Performance depends on hardware and event complexity.

### Can I run simulations in parallel?

Yes, for sweeps and comparisons:

```bash
waremax sweep scenario.yaml \
  --param "robots.count=[10,20,30,40]" \
  --parallel 4
```

### How do I speed up sweeps?

- Use shorter duration for exploration
- Reduce parameter grid size
- Use parallel execution
- Use `--preset quick` base

---

## Troubleshooting

### "Invalid configuration" error

Check:
1. YAML syntax is correct
2. Required fields are present
3. Values are valid types

Use `waremax validate` for details.

### Simulation hangs or deadlocks

Possible causes:
- Actual deadlock in robot movement
- Disconnected map regions
- Missing paths to stations

Enable deadlock detection:
```yaml
traffic:
  deadlock_detection: true
```

### Results seem wrong

Verify:
1. Configuration matches expectations
2. Simulation reached steady state
3. Enough runs for statistical significance
4. Compare against known baseline

### Memory issues

For large simulations:
- Reduce event logging detail
- Use shorter duration for testing
- Close other applications

---

## Best Practices

### What's a good workflow?

1. Start with preset or simple config
2. Validate configuration
3. Run short test simulation
4. Verify results make sense
5. Run full analysis
6. Document findings

### How should I organize my files?

```
project/
├── scenarios/
│   ├── baseline.yaml
│   ├── experiment_1.yaml
│   └── experiment_2.yaml
├── results/
│   ├── baseline/
│   └── experiment_1/
└── analysis/
    └── report.md
```

### How do I report results?

Include:
- Configuration used
- Key metrics
- Multiple runs (with std dev)
- Comparison to baseline
- Recommendations

---

## Getting Help

### Where can I get help?

- Documentation: This site
- Issues: GitHub Issues
- Discussions: GitHub Discussions

### How do I report a bug?

1. Check existing issues
2. Create minimal reproduction
3. Include:
   - Waremax version
   - Configuration (simplified)
   - Expected vs actual behavior
   - Error messages

### How can I contribute?

See [Contributing Guide](developer/contributing/code-style.md):
- Report bugs
- Suggest features
- Submit pull requests
- Improve documentation
