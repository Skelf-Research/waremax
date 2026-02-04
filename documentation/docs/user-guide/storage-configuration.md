# Storage Configuration

Storage files define racks, bins, and inventory placements.

---

## Overview

Storage configuration includes:

- **Racks** - Physical storage structures
- **Bins** - Individual storage locations within racks
- **Placements** - SKU quantities in bins

Storage files are YAML format.

---

## File Format

```yaml
racks:
  - id: "R1"
    node: "6"
    levels: 4
    bins_per_level: 10

  - id: "R2"
    node: "7"
    levels: 4
    bins_per_level: 10

placements:
  - rack: "R1"
    level: 1
    bin: 1
    sku: "SKU001"
    quantity: 100

  - rack: "R1"
    level: 1
    bin: 2
    sku: "SKU002"
    quantity: 50
```

---

## Racks

### Rack Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | string | Unique rack identifier |
| `node` | string | Map node where rack is located |
| `levels` | integer | Number of vertical levels |
| `bins_per_level` | integer | Bins per level |

### Example

```yaml
racks:
  - id: "Rack_A1"
    node: "6"
    levels: 5
    bins_per_level: 20

  - id: "Rack_A2"
    node: "7"
    levels: 5
    bins_per_level: 20

  - id: "Rack_B1"
    node: "11"
    levels: 5
    bins_per_level: 20
```

---

## Placements

### Placement Properties

| Property | Type | Description |
|----------|------|-------------|
| `rack` | string | Rack ID |
| `level` | integer | Level number (1-indexed) |
| `bin` | integer | Bin number (1-indexed) |
| `sku` | string | SKU identifier |
| `quantity` | integer | Initial quantity |

### Example

```yaml
placements:
  # Fast-moving SKUs at eye level
  - rack: "Rack_A1"
    level: 3
    bin: 1
    sku: "SKU001"
    quantity: 500

  - rack: "Rack_A1"
    level: 3
    bin: 2
    sku: "SKU002"
    quantity: 300

  # Slower SKUs at higher/lower levels
  - rack: "Rack_A1"
    level: 1
    bin: 1
    sku: "SKU050"
    quantity: 50
```

---

## Complete Example

```yaml
# Warehouse storage configuration

racks:
  # Zone A - Fast movers
  - id: "ZoneA_R1"
    node: "6"
    levels: 4
    bins_per_level: 15

  - id: "ZoneA_R2"
    node: "7"
    levels: 4
    bins_per_level: 15

  # Zone B - Medium movers
  - id: "ZoneB_R1"
    node: "11"
    levels: 5
    bins_per_level: 20

  - id: "ZoneB_R2"
    node: "12"
    levels: 5
    bins_per_level: 20

  # Zone C - Slow movers
  - id: "ZoneC_R1"
    node: "16"
    levels: 6
    bins_per_level: 25

placements:
  # Zone A placements (high-velocity SKUs)
  - rack: "ZoneA_R1"
    level: 2
    bin: 1
    sku: "SKU001"
    quantity: 1000

  - rack: "ZoneA_R1"
    level: 2
    bin: 2
    sku: "SKU002"
    quantity: 800

  - rack: "ZoneA_R1"
    level: 3
    bin: 1
    sku: "SKU003"
    quantity: 600

  - rack: "ZoneA_R2"
    level: 2
    bin: 1
    sku: "SKU004"
    quantity: 500

  # Zone B placements (medium-velocity SKUs)
  - rack: "ZoneB_R1"
    level: 3
    bin: 1
    sku: "SKU020"
    quantity: 200

  - rack: "ZoneB_R1"
    level: 3
    bin: 2
    sku: "SKU021"
    quantity: 180

  # Zone C placements (slow-moving SKUs)
  - rack: "ZoneC_R1"
    level: 4
    bin: 1
    sku: "SKU100"
    quantity: 50

  - rack: "ZoneC_R1"
    level: 4
    bin: 2
    sku: "SKU101"
    quantity: 40
```

---

## Linking to Scenario

Reference the storage file in your scenario:

```yaml
storage:
  file: warehouse_storage.yaml
```

---

## SKU Popularity

The `sku_popularity` setting in orders affects which SKUs are picked:

```yaml
orders:
  sku_popularity:
    type: zipf
    alpha: 1.0  # Higher alpha = more skewed toward popular SKUs
```

Place fast-moving SKUs:

- Near pick stations
- At ergonomic levels (2-3)
- In easily accessible bins

---

## Replenishment Integration

With replenishment enabled, inventory is monitored:

```yaml
replenishment:
  enabled: true
  default_threshold: 10
  sku_thresholds:
    SKU001: 50  # High-velocity SKU needs more buffer
    SKU002: 30
```

When bin quantity drops below threshold, replenishment tasks are generated.

---

## Default Behavior

If no storage file is specified, Waremax generates demo inventory:

- 20 SKUs distributed across rack nodes
- Random initial quantities
- Used for quick testing

---

## Best Practices

### SKU Placement Strategy

1. **Fast movers near stations** - Minimize travel distance for high-velocity SKUs
2. **Ergonomic levels** - Place frequently accessed items at mid-levels
3. **Zone separation** - Group similar SKUs together
4. **Buffer stock** - Higher quantities for popular SKUs

### Capacity Planning

Calculate total storage capacity:

```
Total bins = Sum(levels × bins_per_level) for all racks
```

Example:

```yaml
racks:
  - id: "R1"
    levels: 4
    bins_per_level: 15
    # Capacity: 4 × 15 = 60 bins

  - id: "R2"
    levels: 5
    bins_per_level: 20
    # Capacity: 5 × 20 = 100 bins

# Total: 160 bins
```

### Multi-Location SKUs

Same SKU can be in multiple locations:

```yaml
placements:
  - rack: "R1"
    level: 2
    bin: 1
    sku: "SKU001"
    quantity: 200

  - rack: "R2"
    level: 3
    bin: 5
    sku: "SKU001"
    quantity: 150

  # Total SKU001: 350 units
```

---

## Validation

Storage is validated during scenario validation:

```bash
waremax validate --scenario my_scenario.yaml
```

Checks include:

- Rack nodes exist in map
- Level and bin numbers are valid
- Quantities are non-negative

---

## Next Steps

- **[Working with Presets](presets.md)** - Using built-in configurations
- **[Configuration Reference](../configuration/index.md)** - Complete parameter reference
