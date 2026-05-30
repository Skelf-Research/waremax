# waremax-storage

**Inventory model for [WareMax](../../README.md): racks, bins, SKUs, replicas, and per-bin stock with deterministic placement.**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

Represents what's stored where in the warehouse: a catalog of SKUs (stock-keeping units), racks anchored at access nodes, multi-level bins, and inventory placements with quantities. Supports replica lookup (a SKU may be in multiple bins) — the spatial lever the *smart pickup-bin* policy and the RL joint-control experiments rely on.

## Key types

| Item | Purpose |
|---|---|
| `Sku` / `SkuCatalog` | Item master data. |
| `Rack` | Storage rack at an access `NodeId`, with levels × bins. |
| `BinAddress` | `{ rack_id, level, bin }`. |
| `Inventory` | Bins → `InventorySlot { sku, quantity }`; `add_placement`, `find_sku`, `find_sku_with_stock`, `decrement`. |

## Determinism

`Inventory` is built up by deterministic `add_placement` calls; `sku_locations` is an insertion-ordered `Vec`, so `find_sku_with_stock` is reproducible. WareMax also sorts the storage-node iteration in `World::init_demo_inventory` so SKU-to-bin assignment is seed-deterministic (a previously latent bug, now tested).

## See also

- [`waremax-entities::Task`](../waremax-entities/) — tasks carry a `source: BinLocation`.
- [WareMax README — Reproducibility](../../README.md#reproducibility).
