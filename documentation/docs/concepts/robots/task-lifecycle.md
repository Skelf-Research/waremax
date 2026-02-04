# Task Lifecycle

The complete journey of a robot task from creation to completion.

---

## Task Types

| Type | Description |
|------|-------------|
| **Pick** | Retrieve item from storage |
| **Drop** | Deliver item to destination |
| **Transport** | Move item between locations |
| **Charge** | Travel to charging station |
| **Maintenance** | Go for scheduled maintenance |

---

## Lifecycle Stages

```
┌─────────────────────────────────────────────────────────────┐
│                        TASK LIFECYCLE                        │
├──────────┬───────────┬──────────┬──────────┬────────────────┤
│ Created  │ Assigned  │ Traveling│ Executing│   Completed    │
│          │           │          │          │                │
│ Order    │ Robot     │ Moving   │ At       │ Task done,     │
│ arrives, │ selected, │ to       │ station, │ robot freed    │
│ task     │ task      │ station  │ being    │                │
│ queued   │ scheduled │          │ serviced │                │
└──────────┴───────────┴──────────┴──────────┴────────────────┘
     │           │           │          │            │
     t₀         t₁          t₂         t₃           t₄
```

---

## Stage 1: Task Creation

### Trigger

Tasks created when:

- Order arrives
- Replenishment needed
- Charging required
- Maintenance scheduled

### Initial State

```
Task {
  id: T001
  type: Pick
  origin: Rack R5
  destination: Station S1
  status: Pending
  created_at: 10:00:00
}
```

---

## Stage 2: Assignment

### Assignment Process

1. Task enters queue
2. Policy selects robot
3. Robot receives task
4. Path calculated

### Assignment Policies

| Policy | Selection Criteria |
|--------|-------------------|
| `nearest_idle` | Closest available robot |
| `least_busy` | Robot with fewest tasks |
| `round_robin` | Sequential assignment |

### After Assignment

```
Task {
  ...
  status: Assigned
  robot: R3
  assigned_at: 10:00:05
  path: [N1, N5, N10, N15]
}
```

---

## Stage 3: Traveling

### Robot Movement

Robot navigates through calculated path:

```
N1 ──> N5 ──> N10 ──> N15
      ↑
    Current position
```

### Possible Events

| Event | Response |
|-------|----------|
| Path clear | Continue travel |
| Node blocked | Wait or reroute |
| Battery low | Interrupt for charging |
| Higher priority task | May preempt (if configured) |

### Travel Metrics

- **Travel time**: Time from start to arrival
- **Wait time**: Time blocked by traffic
- **Distance**: Total path length

---

## Stage 4: Execution

### At Destination

Robot arrives at station and begins service:

1. Enter station queue (if needed)
2. Wait for service slot
3. Receive service
4. Complete task actions

### Service Time

Variable based on task:

```yaml
service_time_s:
  distribution: lognormal
  base: 6.0
  per_item: 2.0
```

### Queue Behavior

```
Station Queue:  [R3] ← [R7] ← [R12]
                 ↑
              Being serviced
```

---

## Stage 5: Completion

### Task Finished

```
Task {
  ...
  status: Completed
  completed_at: 10:02:30
}
```

### Robot Released

Robot transitions to:

- **Idle**: No pending tasks
- **Next task**: If tasks queued
- **Charging**: If battery low

---

## Time Breakdown

### Task Time Components

```
┌─────────────────────────────────────────────────────────┐
│                    Total Task Time                       │
├──────────┬──────────┬──────────┬───────────┬───────────┤
│  Queue   │  Travel  │   Wait   │   Queue   │  Service  │
│  Wait    │  Time    │  (traffic)│  (station)│   Time   │
│          │          │          │           │           │
│  t_q     │   t_t    │   t_w    │    t_sq   │    t_s    │
└──────────┴──────────┴──────────┴───────────┴───────────┘

Total = t_q + t_t + t_w + t_sq + t_s
```

### Key Metrics

| Metric | Formula |
|--------|---------|
| Task time | completion_time - creation_time |
| Travel ratio | travel_time / task_time |
| Wait ratio | (traffic_wait + queue_wait) / task_time |
| Service ratio | service_time / task_time |

---

## Task Interruption

### Preemption

Higher priority tasks may interrupt:

1. Current task paused
2. Urgent task executed
3. Original task resumed

### Battery Interrupt

Low battery forces charging:

```
Task T1: Traveling... → Battery < 20% → Charging... → Resume T1
```

### Failure Interrupt

Robot failure stops task:

```
Task T1: Executing... → Robot fails → Task reassigned to R2
```

---

## Example Timeline

```
10:00:00.000  Order O1 created
10:00:00.100  Task T1 created (pick SKU001 from R5)
10:00:00.250  T1 assigned to Robot R3
10:00:00.300  R3 starts traveling to R5
10:00:04.500  R3 arrives at R5
10:00:04.600  R3 picks item
10:00:04.700  R3 starts traveling to Station S1
10:00:08.200  R3 arrives at S1
10:00:08.300  R3 joins station queue (position 2)
10:00:15.100  R3 reaches service slot
10:00:21.400  Service complete
10:00:21.500  Task T1 completed
10:00:21.600  R3 becomes idle
```

---

## Related

- [Task Allocation Policy](../policies/task-allocation.md)
- [Stations](../warehouse/stations.md)
- [Robot States](index.md)
