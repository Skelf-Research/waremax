# Order Configuration

Configuration for order generation and characteristics.

---

## Schema

```yaml
orders:
  arrival_process:                 # Required
    type: <string>
    rate_per_min: <float>

  lines_per_order:                 # Required
    type: <string>
    mean: <float>
    dispersion: <float>

  sku_popularity:                  # Required
    type: <string>
    alpha: <float>

  due_times:                       # Optional
    type: <string>
    minutes: <float>
```

---

## Arrival Process

Defines how orders arrive over time.

### Poisson Process

Orders arrive randomly with specified rate.

```yaml
orders:
  arrival_process:
    type: poisson
    rate_per_min: 1.0  # 60 orders per hour
```

### rate_per_min

**Type**: float
**Required**: Yes

Order arrival rate per minute.

| rate_per_min | Orders/Hour |
|--------------|-------------|
| 0.5 | 30 |
| 1.0 | 60 |
| 2.0 | 120 |
| 5.0 | 300 |

---

## Lines Per Order

Defines how many items/lines each order contains.

### Negative Binomial

```yaml
orders:
  lines_per_order:
    type: negbinomial
    mean: 3.0
    dispersion: 1.0
```

### mean

**Type**: float
**Required**: Yes

Average number of lines per order.

```yaml
lines_per_order:
  type: negbinomial
  mean: 4.5  # Average 4.5 items per order
```

### dispersion

**Type**: float
**Default**: 1.0

Controls variance in order sizes.

| Dispersion | Effect |
|------------|--------|
| < 1.0 | Less variance, more consistent |
| 1.0 | Standard variance |
| > 1.0 | More variance, some very large orders |

```yaml
lines_per_order:
  type: negbinomial
  mean: 3.0
  dispersion: 1.5  # Higher variance
```

---

## SKU Popularity

Defines which SKUs are selected for orders.

### Zipf Distribution

Models realistic SKU popularity where few SKUs are very popular.

```yaml
orders:
  sku_popularity:
    type: zipf
    alpha: 1.0
```

### alpha

**Type**: float
**Default**: 1.0

Controls popularity skew.

| Alpha | Effect |
|-------|--------|
| 0.5 | Mild skew, more even distribution |
| 1.0 | Standard Zipf (80/20 rule approximately) |
| 1.5 | Strong skew, few SKUs dominate |
| 2.0 | Extreme skew |

```yaml
# Highly concentrated picking
sku_popularity:
  type: zipf
  alpha: 1.5  # Top SKUs get most picks
```

---

## Due Times

Optional configuration for order due times.

### Fixed Offset

Orders are due a fixed time after arrival.

```yaml
orders:
  due_times:
    type: fixed_offset
    minutes: 60  # Due 60 minutes after arrival
```

### minutes

**Type**: float

Minutes after order arrival when order is due.

```yaml
due_times:
  type: fixed_offset
  minutes: 45  # 45-minute SLA
```

---

## Examples

### Standard E-Commerce

```yaml
orders:
  arrival_process:
    type: poisson
    rate_per_min: 1.5  # 90 orders/hour

  lines_per_order:
    type: negbinomial
    mean: 2.5
    dispersion: 1.0

  sku_popularity:
    type: zipf
    alpha: 1.2

  due_times:
    type: fixed_offset
    minutes: 120
```

### High-Volume Retail

```yaml
orders:
  arrival_process:
    type: poisson
    rate_per_min: 5.0  # 300 orders/hour

  lines_per_order:
    type: negbinomial
    mean: 1.5          # Smaller orders
    dispersion: 0.8

  sku_popularity:
    type: zipf
    alpha: 1.0

  due_times:
    type: fixed_offset
    minutes: 30        # Fast turnaround
```

### Wholesale Distribution

```yaml
orders:
  arrival_process:
    type: poisson
    rate_per_min: 0.5  # 30 orders/hour

  lines_per_order:
    type: negbinomial
    mean: 15.0         # Large orders
    dispersion: 2.0    # High variance

  sku_popularity:
    type: zipf
    alpha: 0.8         # More even SKU distribution

  due_times:
    type: fixed_offset
    minutes: 240       # Same-day shipping
```

### Peak Hours Simulation

```yaml
orders:
  arrival_process:
    type: poisson
    rate_per_min: 3.5  # 210 orders/hour (peak)

  lines_per_order:
    type: negbinomial
    mean: 3.0
    dispersion: 1.2

  sku_popularity:
    type: zipf
    alpha: 1.3

  due_times:
    type: fixed_offset
    minutes: 60
```

---

## Order Rate Calculation

### From Orders per Hour

```
rate_per_min = orders_per_hour / 60
```

Examples:

| Orders/Hour | rate_per_min |
|-------------|--------------|
| 60 | 1.0 |
| 120 | 2.0 |
| 300 | 5.0 |

### Expected Throughput

```
Expected orders = rate_per_min × effective_duration_minutes
```

Example:

- Rate: 1.0/min
- Duration: 60 min
- Warmup: 10 min
- Expected: 1.0 × 50 = 50 orders

---

## Related

- [Priority Policies](policies.md#priority-arbitration)
- [Order Concepts](../concepts/simulation/events.md)
