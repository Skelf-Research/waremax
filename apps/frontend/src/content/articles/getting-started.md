---
title: "Getting Started with Waremax"
description: "Learn how to set up and run your first warehouse robot simulation"
pubDate: 2024-01-15
author: "Waremax Team"
tags: ["tutorial", "simulation", "getting-started"]
---

# Getting Started with Waremax

Waremax is a discrete-event simulation engine for warehouse robot fleets. This guide walks you through running your first simulation.

## What is Waremax?

Waremax models the behavior of autonomous mobile robots (AMRs) in a warehouse environment. It simulates:

- **Robot movement** along predefined paths
- **Task allocation** and scheduling
- **Station servicing** with queueing
- **Battery management** and charging cycles
- **Traffic policies** to prevent collisions

## Running Your First Simulation

### Prerequisites

You need Rust installed (1.75+). Clone the repository and build:

```bash
git clone https://github.com/skelfresearch/waremax.git
cd waremax
cargo build --release --bin waremax-api-server
```

### Start the API Server

```bash
cargo run --bin waremax-api-server
```

The server will start on `http://localhost:8080`.

### Launch the Frontend

In a separate terminal:

```bash
cd apps/frontend
npm install
npm run dev
```

Open `http://localhost:4321` and navigate to the Simulation page.

### Create a Session

1. Select a preset (e.g., "Standard Warehouse")
2. Adjust robot count, order rate, and duration
3. Click **Create Simulation**
4. Use the playback controls to start, pause, or step through the simulation

## Understanding the Dashboard

The simulation dashboard shows:

- **Warehouse Canvas**: Live visualization of robots, nodes, and stations
- **Metrics Panel**: Throughput, utilization, cycle times
- **Event Log**: Real-time simulation events
- **Config Panel**: Adjust parameters and view robot state counts

## Next Steps

- Read the [Configuration Guide](/articles/configuration) to customize scenarios
- Check the [API Reference](/articles/api-reference) for programmatic access
- Explore pluggable policies for traffic, routing, and task allocation
