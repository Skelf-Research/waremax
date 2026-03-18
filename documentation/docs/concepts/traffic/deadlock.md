# Deadlock

Circular waiting situations and how to handle them.

---

## What is Deadlock?

Deadlock occurs when robots are waiting for each other in a cycle, preventing any from proceeding.

### Classic Example

```
Robot A: Has N1, wants N2
Robot B: Has N2, wants N1

    N1          N2
   [A] ───────> wants
   wants <───── [B]

Neither can move → Deadlock
```

---

## Deadlock Conditions

All four must be true simultaneously:

### 1. Mutual Exclusion

Resources can only be held by one robot:

```
Node capacity = 1
Only R1 can occupy N1
```

### 2. Hold and Wait

Robot holds resources while waiting for others:

```
R1 holds N1, waits for N2
```

### 3. No Preemption

Cannot force robot to release resources:

```
R1 cannot be forced off N1
```

### 4. Circular Wait

Cycle in resource dependency graph:

```
R1 → N2 → R2 → N1 → R1
```

---

## Deadlock Types

### Two-Robot Deadlock

```
[R1] → N2 ← [R2]
  ↑           ↓
  N1 ← → → → →
```

### Multi-Robot Deadlock

```
[R1] → N2 ← [R2]
  ↑           ↓
  N4 ← [R4]  N3
  ↑           ↓
[R3] ← ← ← ← ←
```

### Intersection Deadlock

```
      [R2]
        ↓
[R1] → ╬ ← [R3]
        ↑
      [R4]

All waiting for center
```

---

## Detection

### Wait-For Graph

Build graph of who waits for whom:

```
R1 waits for R2
R2 waits for R3
R3 waits for R1

Graph: R1 → R2 → R3 → R1
                    ↑
            Cycle = Deadlock!
```

### Cycle Detection

Periodically check wait-for graph:

```yaml
traffic:
  deadlock_detection: true
  detection_interval_s: 1.0
```

---

## Prevention Strategies

### Resource Ordering

Always acquire resources in consistent order:

```
Rule: Lower ID first

R1 wants N3, N1 → Acquire N1, then N3
R2 wants N1, N3 → Acquire N1, then N3

No conflicting order → No deadlock
```

### Priority-Based

Higher priority robot always proceeds:

```yaml
traffic:
  deadlock_prevention: priority
```

```
R1 (priority 5): Has N1, wants N2
R2 (priority 3): Has N2, wants N1

R2 must yield → R1 proceeds
```

### Time-Based

Older requests have priority:

```
R1: Waiting since t=10
R2: Waiting since t=15

R1 has priority → R1 proceeds
```

---

## Resolution Strategies

### Backoff

One robot retreats:

```yaml
traffic:
  deadlock_resolution: backoff
```

```
Before:
  [R1] → N2 ← [R2]

After (R2 backs off):
  [R1] → N2    [R2] ← moved away
  [R1] in N2   [R2] waits
```

### Priority Resolution

Lower priority robot moves:

```yaml
traffic:
  deadlock_resolution: priority
```

### Timeout

After waiting too long, force resolution:

```yaml
traffic:
  deadlock_timeout_s: 30.0
```

---

## Prevention in Design

### One-Way Aisles

Eliminate head-on conflicts:

```
→→→→→→→→→→
          ↓
←←←←←←←←←←
```

### Passing Zones

Allow robots to pass:

```
═══╬═══╬═══╬═══
   │   │   │
 Pass zones (capacity > 1)
```

### Loop Layouts

Avoid dead ends:

```
╔═══════════╗
║           ║
║   ╔═══╗   ║
║   ║   ║   ║
║   ╚═══╝   ║
║           ║
╚═══════════╝

Always a path around
```

---

## Deadlock Metrics

### Detection Metrics

| Metric | Description |
|--------|-------------|
| Deadlock count | Number detected |
| Robots involved | Per deadlock |
| Time to detect | Detection latency |

### Resolution Metrics

| Metric | Description |
|--------|-------------|
| Resolution time | Time to clear deadlock |
| Backoff distance | How far robots retreat |
| Tasks affected | Delayed by deadlock |

---

## Configuration

```yaml
traffic:
  # Detection
  deadlock_detection: true
  detection_interval_s: 1.0

  # Prevention
  deadlock_prevention: priority

  # Resolution
  deadlock_resolution: backoff
  deadlock_timeout_s: 30.0
  max_backoff_distance: 5
```

---

## Example: Deadlock Resolution

### Scenario

```
t=0:   R1 at N1, heading to N3
       R2 at N3, heading to N1

t=1:   R1 at N2, wants N3
       R2 at N2... blocked!

       Wait... R2 is at N2?

       Actually:
       R1 at N2, wants N3
       R2 at N3, wants N2

       Deadlock detected!
```

### Resolution (Priority)

```
R1 priority: 5
R2 priority: 3

R2 must yield:
  - R2 backs off to N4
  - R1 moves to N3
  - R2 resumes path
```

### Timeline

```
t=1.00:  Deadlock detected
t=1.01:  Resolution: R2 yields
t=1.50:  R2 backs off to N4
t=2.00:  R1 moves to N3
t=2.50:  R2 resumes, moves to N3
t=3.00:  Normal operation resumes
```

---

## Best Practices

### Prevention Over Detection

- Good layout design prevents most deadlocks
- Detection is fallback, not primary strategy

### Monitor Deadlock Frequency

- High deadlock rate indicates design issues
- Investigate hotspots

### Test Edge Cases

- Simulate high-traffic scenarios
- Verify resolution works correctly

---

## Related

- [Capacity](capacity.md)
- [Congestion](congestion.md)
- [Traffic Configuration](../../configuration/traffic.md)
