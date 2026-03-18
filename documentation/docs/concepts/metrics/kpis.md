# Key Performance Indicators

Core metrics for measuring warehouse performance.

---

## What are KPIs?

Key Performance Indicators (KPIs) are the critical metrics that define success. They summarize complex system behavior into actionable numbers.

---

## Throughput KPIs

### Tasks Per Hour

Primary throughput measure:

```
Throughput = Tasks Completed / Hours
```

**Target**: Depends on warehouse requirements

**Example**:
```
1,250 tasks in 1 hour
Throughput = 1,250 tasks/hour
```

### Orders Per Hour

Order-level throughput:

```
Order Throughput = Orders Completed / Hours
```

Note: One order may have multiple tasks.

### Peak Throughput

Maximum sustainable rate:

```
Peak Throughput = Max tasks/hour over sustained period
```

---

## Time KPIs

### Average Task Time

Mean time from creation to completion:

```
Avg Task Time = Σ(completion_time - creation_time) / task_count
```

**Components**:
```
Task Time = Queue Wait + Travel + Traffic Wait + Station Queue + Service
```

### Order Cycle Time

Time to fulfill complete order:

```
Cycle Time = Order Completion - Order Creation
```

### P95 Task Time

95th percentile task time:

```
95% of tasks complete within this time
```

Captures worst-case performance better than average.

---

## Utilization KPIs

### Robot Utilization

How busy robots are:

```
Robot Utilization = (Working + Traveling) / Total Time
```

**Breakdown**:
```
100% = Working + Traveling + Waiting + Idle + Charging + Maintenance
```

**Target**: 70-85% (too high causes congestion)

### Station Utilization

How busy stations are:

```
Station Utilization = Service Time / Total Time
```

**Target**: Varies by station type

### Fleet Efficiency

Productive work vs. overhead:

```
Fleet Efficiency = Work Time / (Work Time + Travel Time + Wait Time)
```

---

## Quality KPIs

### On-Time Delivery Rate

Tasks meeting due time:

```
On-Time Rate = On-Time Tasks / Total Tasks × 100%
```

**Target**: 95%+ typically

### Late Task Count

Absolute count of missed due times:

```
Late Tasks = Tasks where completion > due_time
```

### Average Lateness

For late tasks only:

```
Avg Lateness = Σ(completion_time - due_time) / late_task_count
```

---

## Congestion KPIs

### Average Wait Time

Time robots spend waiting:

```
Avg Wait = Total Wait Time / Task Count
```

### Congestion Rate

Percentage of time edges/nodes are at capacity:

```
Congestion Rate = Time at Capacity / Total Time
```

### Deadlock Count

Number of deadlocks detected:

```
Deadlocks = Count of deadlock events
```

**Target**: 0 ideally

---

## Reliability KPIs

### Robot Availability

Fleet uptime:

```
Availability = (Operating + Idle) / Total Time
```

### Mean Time Between Failures

Average operating time before failure:

```
MTBF = Total Operating Time / Failure Count
```

### Mean Time To Repair

Average repair duration:

```
MTTR = Total Repair Time / Failure Count
```

---

## KPI Dashboard

### Summary View

```
┌─────────────────────────────────────────────────┐
│                  KPI DASHBOARD                   │
├─────────────────────────────────────────────────┤
│ Throughput                                       │
│   Tasks/Hour:     1,250    (Target: 1,000) ✓    │
│   Orders/Hour:      312    (Target: 250)  ✓    │
│                                                  │
│ Time                                             │
│   Avg Task Time:   42.3s   (Target: 60s)  ✓    │
│   P95 Task Time:   85.0s   (Target: 120s) ✓    │
│                                                  │
│ Utilization                                      │
│   Robot:           78%     (Target: 70-85%)✓    │
│   Station:         82%     (Target: 80%)  ✓    │
│                                                  │
│ Quality                                          │
│   On-Time Rate:    96.5%   (Target: 95%)  ✓    │
│   Late Tasks:      44      (Target: <100) ✓    │
│                                                  │
│ Congestion                                       │
│   Avg Wait:        3.2s    (Target: <5s)  ✓    │
│   Deadlocks:       0       (Target: 0)    ✓    │
└─────────────────────────────────────────────────┘
```

---

## Setting KPI Targets

### Baseline First

Run baseline simulation:

```bash
waremax run baseline.yaml -o baseline/
```

Extract current KPIs as starting point.

### Industry Benchmarks

| KPI | Good | Excellent |
|-----|------|-----------|
| Robot utilization | 70% | 80% |
| On-time rate | 95% | 99% |
| P95 task time | 2× average | 1.5× average |

### Business Requirements

Work backward from business needs:

```
Need: 1,000 orders/day
Hours: 16 operating hours
Required: 62.5 orders/hour
```

---

## Monitoring KPIs

### Real-Time Tracking

```yaml
metrics:
  timeseries:
    enabled: true
    interval_s: 60
    include:
      - throughput
      - utilization
      - queue_length
```

### Alerting Thresholds

```yaml
metrics:
  alerts:
    throughput_below: 800
    utilization_above: 90
    wait_time_above: 10.0
```

---

## KPI Trade-offs

### Throughput vs. Utilization

```
High utilization → More congestion → Lower effective throughput
```

Find balance point.

### Speed vs. Quality

```
Faster service → Less thorough → Potential errors
```

### Cost vs. Performance

```
More robots → Higher throughput → Higher cost
```

---

## Configuration

```yaml
metrics:
  kpis:
    enabled: true
    include:
      - throughput
      - task_time
      - utilization
      - on_time_rate
      - wait_time

  targets:
    throughput_per_hour: 1000
    avg_task_time_s: 60
    on_time_rate_pct: 95
```

---

## Related

- [Time Series Data](timeseries.md)
- [Metrics Configuration](../../configuration/metrics.md)
- [Analysis Command](../../cli/analyze.md)
