"""Shared metric extraction for baseline vs RL comparison."""

from __future__ import annotations

import statistics


def metrics_from_report(report: dict) -> dict:
    """Extract comparison metrics from a simulation-report JSON dict."""
    completed = report.get("orders_completed", 0)
    late = report.get("orders_late", 0)
    sla = report.get("sla")
    if sla is not None:
        on_time_rate = 1.0 - sla.get("sla_miss_rate", 0.0)
        p95_lateness = sla.get("p95_lateness_s", 0.0)
    else:
        on_time_rate = (completed - late) / completed if completed > 0 else 0.0
        p95_lateness = 0.0
    return {
        "on_time_pct": 100.0 * on_time_rate,
        "p95_lateness_s": p95_lateness,
        "throughput_per_hour": report.get("throughput_per_hour", 0.0),
        "orders_completed": completed,
    }


def metrics_from_info(info: dict) -> dict:
    """Extract comparison metrics from the terminal-step `info` dict.

    Uses the same SLA-based definitions as `metrics_from_report` so RL and
    heuristic rows in the comparison table are directly comparable.
    """
    return {
        "on_time_pct": 100.0 * info.get("on_time_rate", 0.0),
        "p95_lateness_s": info.get("p95_lateness_s", 0.0),
        "throughput_per_hour": info.get("throughput_per_hour", 0.0),
        "orders_completed": info.get("orders_completed", 0),
    }


def aggregate(rows: list[dict]) -> dict:
    """Mean +/- stdev over a list of metric dicts."""
    keys = ["on_time_pct", "p95_lateness_s", "throughput_per_hour", "orders_completed"]
    out = {}
    for k in keys:
        vals = [r[k] for r in rows]
        mean = statistics.fmean(vals)
        sd = statistics.pstdev(vals) if len(vals) > 1 else 0.0
        out[k] = (mean, sd)
    return out


def format_table(results: dict[str, dict]) -> str:
    """results: policy_name -> aggregated metrics."""
    header = f"{'policy':<22}{'SLA on-time %':>16}{'p95 lateness s':>18}{'throughput/h':>16}"
    lines = [header, "-" * len(header)]
    for name, agg in results.items():
        ot = agg["on_time_pct"]
        p95 = agg["p95_lateness_s"]
        thr = agg["throughput_per_hour"]
        lines.append(
            f"{name:<22}{ot[0]:>10.1f}±{ot[1]:<4.1f}{p95[0]:>12.1f}±{p95[1]:<4.1f}"
            f"{thr[0]:>10.1f}±{thr[1]:<4.1f}"
        )
    return "\n".join(lines)
