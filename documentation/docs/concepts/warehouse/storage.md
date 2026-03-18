# Storage & Inventory

Racks, bins, and inventory management.

---

## Storage Hierarchy

```
Warehouse
└── Racks
    └── Levels
        └── Bins
            └── SKUs (with quantities)
```

---

## Racks

### Definition

Physical storage structures containing multiple levels of bins.

### Properties

| Property | Description |
|----------|-------------|
| `id` | Unique rack identifier |
| `node` | Map node location |
| `levels` | Number of vertical levels |
| `bins_per_level` | Bins per level |

### Example

```yaml
racks:
  - id: "R1"
    node: "N10"
    levels: 5
    bins_per_level: 20
    # Total: 100 bin locations
```

---

## Bins

### Definition

Individual storage locations within a rack.

### Addressing

Bins are addressed by:

```
(rack_id, level, bin_number)
```

Example: `(R1, 3, 15)` = Rack R1, Level 3, Bin 15

### Capacity

Each bin can hold one SKU type (simplified model).

---

## SKUs

### Definition

Stock Keeping Units - unique product identifiers.

### Properties

- SKU ID (e.g., "SKU001")
- Quantity at each location

### Placement

```yaml
placements:
  - rack: "R1"
    level: 2
    bin: 5
    sku: "SKU001"
    quantity: 100
```

---

## Inventory Flow

### Pick Operations

1. Order specifies SKU
2. System locates SKU in storage
3. Robot travels to rack
4. Item picked, quantity decremented

### Putaway Operations

1. Inbound items arrive
2. System assigns bin location
3. Robot travels to rack
4. Item stored, quantity incremented

### Replenishment

When bin quantity drops below threshold:

1. Replenishment task created
2. Robot retrieves from reserve
3. Bin replenished

---

## SKU Popularity

### Zipf Distribution

Models realistic popularity:

- Few SKUs are very popular
- Many SKUs are rarely picked

```yaml
orders:
  sku_popularity:
    type: zipf
    alpha: 1.0
```

### Alpha Effect

| Alpha | Distribution |
|-------|--------------|
| 0.5 | More even |
| 1.0 | Standard Zipf (80/20 rule) |
| 1.5 | More concentrated |

---

## Storage Strategy

### Slotting

Place fast-movers near stations:

- Reduces travel time
- Improves throughput

### Zoning

Group related SKUs:

- Zone A: Fast movers
- Zone B: Medium movers
- Zone C: Slow movers

### Level Assignment

Ergonomic considerations:

- Levels 2-3: Fast movers (easy access)
- Levels 1, 4+: Slow movers

---

## Capacity Planning

### Total Capacity

```
Capacity = Σ(levels × bins_per_level) for all racks
```

### SKU Coverage

Ensure sufficient locations for:

- Active SKUs
- Safety stock
- Seasonal inventory

---

## Example Configuration

```yaml
racks:
  # Zone A - Fast movers
  - id: "ZoneA_R1"
    node: "N10"
    levels: 4
    bins_per_level: 15

  - id: "ZoneA_R2"
    node: "N11"
    levels: 4
    bins_per_level: 15

  # Zone B - Medium movers
  - id: "ZoneB_R1"
    node: "N20"
    levels: 5
    bins_per_level: 20

placements:
  # Fast movers in Zone A, ergonomic levels
  - rack: "ZoneA_R1"
    level: 2
    bin: 1
    sku: "SKU001"
    quantity: 500

  - rack: "ZoneA_R1"
    level: 3
    bin: 1
    sku: "SKU002"
    quantity: 400
```

---

## Related

- [Storage Configuration](../../user-guide/storage-configuration.md)
- [Order Configuration](../../configuration/orders.md)
