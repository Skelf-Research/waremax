# waremax-api

**REST and WebSocket API library for driving [WareMax](../../README.md) simulations remotely.**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

An Axum-based HTTP/WebSocket layer exposing the simulator as a service: submit a scenario, run it, stream metrics, poll for the report. Intended for a future dashboard front-end and for remote experiment orchestration. The binary entry point is [`waremax-api-server`](../waremax-api-server/).

## Built on

- [axum](https://docs.rs/axum) for routing and WebSockets.
- [tokio](https://docs.rs/tokio) for async runtime.
- The full WareMax stack (`waremax-sim`, `waremax-testing`, `waremax-metrics`, `waremax-analysis`).

## Status

In active development; see [`docs/roadmap.md`](../../docs/roadmap.md).

## See also

- [`waremax-api-server`](../waremax-api-server/) — runnable binary.
