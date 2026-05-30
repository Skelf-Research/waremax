# waremax-statemachine

**Generic state-machine primitives used by [WareMax](../../README.md) policies and entity lifecycles.**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

A tiny supporting crate of state-machine helpers shared by other crates (notably the robot lifecycle in [`waremax-entities`](../waremax-entities/) and policy implementations in [`waremax-policies`](../waremax-policies/)). Kept in its own crate to avoid pulling heavy deps into common downstream consumers.

## See also

- [`waremax-entities::RobotState`](../waremax-entities/) — the largest consumer.
- [`waremax-policies`](../waremax-policies/) — policy implementations.
