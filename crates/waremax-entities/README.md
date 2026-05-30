# waremax-entities

**Domain entities for [WareMax](../../README.md): robots, orders, tasks, stations, charging stations, maintenance stations, and their state machines.**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

Defines the in-memory representation of every actor and work item the simulator manipulates. These types are owned by `World` (in [`waremax-sim`](../waremax-sim/)), passed read-only to policies via `PolicyContext`, and inspected by metrics and analysis.

## Key types

| Item | Purpose |
|---|---|
| `Robot` | Position, state (Idle/Moving/Waiting/Servicing/Charging/Failed), battery (`BatteryState`), task queue, cumulative stats. |
| `RobotState` | Lifecycle enum used by the state machine. |
| `BatteryState` / `BatteryConsumptionModel` | SoC dynamics: per-meter, per-kg-per-meter, idle, service. |
| `MaintenanceState` / `FailureModel` | v3 reliability: scheduled maintenance + MTBF-based random failures. |
| `Order` / `OrderLine` / `OrderStatus` | Customer demand: arrival/due/completion times, line items, `is_late()`. |
| `Task` / `TaskType` / `TaskStatus` / `BinLocation` | Atomic unit of work, with `sku_id`, `quantity`, `source`, `destination_station`, lifecycle timestamps. |
| `Station` / `StationType` / `ServiceTimeModel` | Pick/drop/inbound/outbound; concurrency, queue, service-time distribution. |
| `ChargingStation` / `MaintenanceStation` | v1/v3 station types. |

## See also

- [`waremax-sim::World`](../waremax-sim/) — owns the entity maps.
- [`waremax-policies::PolicyContext`](../waremax-policies/) — read-only view passed to policies.
