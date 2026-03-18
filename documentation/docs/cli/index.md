# CLI Reference

Complete reference for the Waremax command-line interface.

---

## Overview

Waremax provides a comprehensive CLI for running and analyzing warehouse simulations.

```bash
waremax [COMMAND] [OPTIONS]
```

---

## Commands

### Simulation Commands

| Command | Description |
|---------|-------------|
| [run](run.md) | Run a simulation from a scenario file |
| [validate](validate.md) | Validate a scenario file without running |
| [demo](demo.md) | Run a quick demo simulation |

### Generation Commands

| Command | Description |
|---------|-------------|
| [generate](generate.md) | Generate a scenario file from a preset |
| [list-presets](list-presets.md) | List available scenario presets |

### Testing Commands

| Command | Description |
|---------|-------------|
| [sweep](sweep.md) | Run a parameter sweep |
| [compare](compare.md) | Compare two configurations |
| [ab-test](ab-test.md) | Run A/B test with statistical analysis |
| [benchmark](benchmark.md) | Run benchmark suite |
| [stress-test](stress-test.md) | Run stress test |

### Analysis Commands

| Command | Description |
|---------|-------------|
| [analyze](analyze.md) | Run root cause analysis |

---

## Global Options

| Option | Description |
|--------|-------------|
| `--help`, `-h` | Show help information |
| `--version`, `-V` | Show version number |

---

## Common Patterns

### Run a simulation

```bash
waremax run --scenario my_scenario.yaml
```

### Generate and run

```bash
waremax generate --preset standard --output scenario.yaml
waremax run --scenario scenario.yaml
```

### Export results

```bash
waremax run --scenario scenario.yaml \
  --output-dir ./results \
  --per-robot \
  --per-station \
  --heatmap
```

### Compare configurations

```bash
waremax compare \
  --baseline baseline.yaml \
  --variant variant.yaml \
  --replications 5
```

### Parameter sweep

```bash
waremax sweep \
  --base baseline.yaml \
  --sweep "robots:5,10,15,20" \
  --output-dir ./sweep_results
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (invalid config, runtime error) |

---

## Environment Variables

Currently, Waremax does not use environment variables for configuration.

---

## Getting Help

Get help for any command:

```bash
# General help
waremax --help

# Command-specific help
waremax run --help
waremax generate --help
waremax sweep --help
```
