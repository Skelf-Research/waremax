# waremax-api-server

**Runnable HTTP/WebSocket server binary for [WareMax](../../README.md): wraps [`waremax-api`](../waremax-api/) as a service.**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Run

```bash
cargo run -p waremax-api-server --release
```

The server hosts the REST + WebSocket endpoints defined in [`waremax-api`](../waremax-api/): scenario submission, run control, live metrics streaming, report retrieval.

## See also

- [`waremax-api`](../waremax-api/) — the underlying library.
- [`docs/roadmap.md`](../../docs/roadmap.md) — planned dashboard front-end.
