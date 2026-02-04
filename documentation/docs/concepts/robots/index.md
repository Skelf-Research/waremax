# Robot Concepts

Understanding autonomous mobile robots in Waremax.

---

## Overview

Robots are autonomous agents that execute tasks in the warehouse. They navigate between locations, transport items, and coordinate to avoid conflicts.

| Topic | Description |
|-------|-------------|
| [Movement](movement.md) | Navigation and pathfinding |
| [Task Lifecycle](task-lifecycle.md) | From assignment to completion |
| [Battery](battery.md) | Power management and charging |
| [Maintenance](maintenance.md) | Reliability and repair |

---

## Robot Model

### Physical Properties

| Property | Description | Typical Value |
|----------|-------------|---------------|
| Speed | Travel velocity | 1.0-2.0 m/s |
| Acceleration | Speed change rate | 0.5-1.0 m/s² |
| Payload | Carrying capacity | 10-50 kg |

### State Machine

```
                ┌──────────────────────────────────────┐
                │                                      │
                v                                      │
┌─────────┐  assign  ┌──────────┐  arrive  ┌─────────┐│
│  IDLE   │ ──────> │ TRAVELING │ ──────> │ WORKING ││
└─────────┘         └──────────┘          └─────────┘│
     ^                    │                    │      │
     │                    │ blocked            │ done │
     │                    v                    │      │
     │              ┌──────────┐               │      │
     │              │ WAITING  │               │      │
     │              └──────────┘               │      │
     │                    │                    │      │
     │                    └────────────────────┘      │
     │                                                │
     └────────────────────────────────────────────────┘
```

---

## Robot States

| State | Description |
|-------|-------------|
| `Idle` | No current task, waiting for assignment |
| `Traveling` | Moving toward destination |
| `Waiting` | Blocked by traffic or resource |
| `Working` | Performing task at station |
| `Charging` | At charging station |
| `Maintenance` | Under repair or scheduled maintenance |

---

## Fleet Management

### Homogeneous Fleets

All robots have identical capabilities:

- Same speed and capacity
- Simplified assignment
- Interchangeable robots

### Fleet Sizing

Key considerations:

- **Throughput requirements**: More robots = higher throughput (to a point)
- **Congestion**: Too many robots cause traffic issues
- **Cost**: Each robot has operational costs

### Optimal Fleet Size

Find the "sweet spot":

```
                Throughput
                    │
                    │        ┌────────────
                    │       /
                    │      /
                    │     /
                    │    /
                    │   /
                    │  /
                    │ /
                    │/
                    └────────────────────── Robots
                              ↑
                        Sweet Spot
```

---

## Task Execution

### Basic Flow

1. **Assignment**: Robot receives task
2. **Path Planning**: Calculate route
3. **Travel**: Navigate to destination
4. **Execute**: Perform task action
5. **Complete**: Return to idle or next task

### Efficiency Metrics

| Metric | Formula |
|--------|---------|
| Utilization | Working time / Total time |
| Travel ratio | Travel time / Total task time |
| Idle ratio | Idle time / Total time |

---

## Related

- [Robot Configuration](../../configuration/robots.md)
- [Task Allocation Policy](../policies/task-allocation.md)
