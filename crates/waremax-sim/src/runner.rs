//! Simulation runner

use waremax_core::{Kernel, SimTime, SimEvent};
use waremax_metrics::{
    MetricsCollector, SimulationReport, SLAReport, CongestionReport, BatteryReport,
    RobotReport, StationReport, ReliabilityReport, HeatmapData, NodeCongestion, EdgeCongestion,
};
use crate::world::World;
use crate::handlers::EventHandler;

/// Main simulation runner
pub struct SimulationRunner {
    kernel: Kernel,
    world: World,
    handler: EventHandler,
    metrics: MetricsCollector,
    end_time: SimTime,
    warmup_time: SimTime,
}

impl SimulationRunner {
    pub fn new(
        world: World,
        duration_minutes: f64,
        warmup_minutes: f64,
    ) -> Self {
        Self {
            kernel: Kernel::new(),
            world,
            handler: EventHandler::new(),
            metrics: MetricsCollector::new(),
            end_time: SimTime::from_minutes(warmup_minutes + duration_minutes),
            warmup_time: SimTime::from_minutes(warmup_minutes),
        }
    }

    /// Initialize the simulation
    pub fn initialize(&mut self) {
        // Schedule first order arrival
        let first_order_id = self.world.next_order_id();
        self.kernel.schedule_now(SimEvent::OrderArrival { order_id: first_order_id });

        // Place robots at their starting positions
        for robot in self.world.robots.values() {
            self.world.traffic.enter_node(robot.current_node, robot.id);
        }

        // v1: Schedule first metrics sample tick
        if self.world.metrics_sample_interval_s > 0.0 {
            let sample_time = SimTime::from_seconds(self.world.metrics_sample_interval_s);
            self.kernel.schedule_after(sample_time, SimEvent::MetricsSampleTick);
        }
    }

    /// Get reference to the world (useful after simulation for analysis)
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Run the simulation
    pub fn run(&mut self) -> SimulationReport {
        self.initialize();

        while self.kernel.has_events() {
            if self.kernel.now() >= self.end_time {
                break;
            }

            if let Some(event) = self.kernel.pop_next() {
                // Record metrics after warmup
                if self.kernel.now() >= self.warmup_time {
                    self.metrics.record_event(&event);
                }

                // Handle the event
                self.handler.handle(&mut self.kernel, &mut self.world, &event, &mut self.metrics);
            }
        }

        self.generate_report()
    }

    fn generate_report(&self) -> SimulationReport {
        let duration = self.kernel.now() - self.warmup_time;
        let duration_s = duration.as_seconds().max(0.0);

        // Calculate robot utilization
        let total_robot_time = duration_s * self.world.robots.len() as f64;
        let total_active_time: f64 = self.world.robots.values()
            .map(|r| r.total_move_time.as_seconds() + r.total_service_time.as_seconds())
            .sum();
        let robot_utilization = if total_robot_time > 0.0 {
            total_active_time / total_robot_time
        } else {
            0.0
        };

        // Calculate station utilization
        let total_station_capacity: f64 = self.world.stations.values()
            .map(|s| s.concurrency as f64 * duration_s)
            .sum();
        let total_station_busy: f64 = self.world.stations.values()
            .map(|s| s.total_service_time.as_seconds())
            .sum();
        let station_utilization = if total_station_capacity > 0.0 {
            total_station_busy / total_station_capacity
        } else {
            0.0
        };

        let mut report = SimulationReport::new(
            duration_s,
            self.kernel.events_processed(),
            self.metrics.orders_completed(),
            self.metrics.orders_late(),
            self.metrics.avg_cycle_time(),
            self.metrics.p95_cycle_time(),
            robot_utilization,
            station_utilization,
        );

        // v1: Add SLA metrics
        let sla = &self.metrics.sla_metrics;
        if sla.total_orders() > 0 {
            report = report.with_sla(SLAReport {
                orders_on_time: sla.orders_on_time,
                orders_late: sla.orders_late,
                sla_miss_rate: sla.sla_miss_rate(),
                avg_lateness_s: sla.avg_lateness_s(),
                p95_lateness_s: sla.p95_lateness_s(),
                max_lateness_s: sla.max_lateness_s(),
            });
        }

        // v1: Add congestion metrics from time-series collector
        let ts = &self.world.time_series;
        let total_node_wait = ts.total_node_wait_events();
        let total_edge_wait = ts.total_edge_wait_events();
        if total_node_wait > 0 || total_edge_wait > 0 {
            report = report.with_congestion(CongestionReport {
                total_node_wait_events: total_node_wait,
                total_edge_wait_events: total_edge_wait,
                total_node_wait_time_s: ts.total_node_wait_time(),
                total_edge_wait_time_s: ts.total_edge_wait_time(),
                top_congested_nodes: ts.top_congested_nodes(10),
                top_congested_edges: ts.top_congested_edges(10),
            });
        }

        // v1: Add battery metrics
        let total_charging_events: u32 = self.world.robots.values()
            .map(|r| r.charging_events)
            .sum();
        let total_energy_consumed: f64 = self.world.robots.values()
            .map(|r| r.total_energy_consumed_wh)
            .sum();
        let total_charging_time: f64 = self.world.robots.values()
            .map(|r| r.total_charging_time.as_seconds())
            .sum();

        if total_charging_events > 0 || total_energy_consumed > 0.0 {
            // Get average SOC at charge start from metrics
            let avg_soc_at_charge = self.metrics.avg_soc_at_charge();

            report = report.with_battery(BatteryReport {
                total_charging_events,
                total_energy_consumed_wh: total_energy_consumed,
                total_charging_time_s: total_charging_time,
                avg_soc_at_charge,
            });
        }

        report
    }

    /// Get current simulation time
    pub fn now(&self) -> SimTime {
        self.kernel.now()
    }

    /// Get mutable reference to metrics
    pub fn metrics(&self) -> &MetricsCollector {
        &self.metrics
    }

    /// Get mutable reference to world
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    // === v3: Enhanced Report Generation ===

    /// Generate per-robot breakdown reports
    pub fn generate_robot_reports(&self, total_time_s: f64) -> Vec<RobotReport> {
        self.world.robots.values().map(|robot| {
            let working_time = robot.total_move_time.as_seconds() + robot.total_service_time.as_seconds();
            // Estimate maintenance time from maintenance count (use avg duration from metrics)
            let maintenance_time = if robot.maintenance.maintenance_count > 0 {
                robot.maintenance.maintenance_count as f64 * 300.0 // Default ~5 min per maintenance
            } else {
                0.0
            };
            let idle_time = total_time_s - working_time - robot.total_charging_time.as_seconds()
                - maintenance_time;
            let utilization = if total_time_s > 0.0 {
                working_time / total_time_s
            } else {
                0.0
            };

            RobotReport {
                robot_id: robot.id.0,
                tasks_completed: robot.tasks_completed,
                distance_traveled_m: robot.total_distance,
                energy_consumed_wh: robot.total_energy_consumed_wh,
                idle_time_s: idle_time.max(0.0),
                working_time_s: working_time,
                charging_time_s: robot.total_charging_time.as_seconds(),
                maintenance_time_s: maintenance_time,
                failure_count: robot.maintenance.failure_count,
                utilization,
            }
        }).collect()
    }

    /// Generate per-station breakdown reports
    pub fn generate_station_reports(&self, total_time_s: f64) -> Vec<StationReport> {
        self.world.stations.values().map(|station| {
            let total_service_time = station.total_service_time.as_seconds();
            let avg_service_time = if station.total_served > 0 {
                total_service_time / station.total_served as f64
            } else {
                0.0
            };

            // Get queue statistics from time-series collector
            let queue_stats = self.world.time_series.station_series.get(&station.id);
            let (avg_queue, max_queue) = if let Some(stats) = queue_stats {
                let avg = if stats.queue_length.is_empty() {
                    0.0
                } else {
                    stats.queue_length.iter().map(|p| p.value as f64).sum::<f64>()
                        / stats.queue_length.len() as f64
                };
                let max = stats.queue_length.iter().map(|p| p.value).max().unwrap_or(0);
                (avg, max)
            } else {
                (0.0, station.max_queue_length)
            };

            let capacity_seconds = total_time_s * station.concurrency as f64;
            let utilization = if capacity_seconds > 0.0 {
                total_service_time / capacity_seconds
            } else {
                0.0
            };

            StationReport {
                station_id: station.id.0,
                string_id: station.string_id.clone(),
                station_type: format!("{:?}", station.station_type),
                orders_served: station.total_served,
                total_service_time_s: total_service_time,
                avg_service_time_s: avg_service_time,
                avg_queue_length: avg_queue,
                max_queue_length: max_queue,
                utilization,
            }
        }).collect()
    }

    /// Generate reliability metrics report
    pub fn generate_reliability_report(&self, total_time_s: f64) -> ReliabilityReport {
        let robot_count = self.world.robots.len() as u32;
        let total_operating_hours = (total_time_s * robot_count as f64) / 3600.0;

        ReliabilityReport {
            total_failures: self.metrics.failure_count(),
            total_maintenance_events: self.metrics.maintenance_count(),
            total_repair_events: self.metrics.repair_count(),
            actual_mtbf_hours: self.metrics.actual_mtbf_hours(total_operating_hours),
            mttr_s: self.metrics.mttr_s(),
            fleet_availability: self.metrics.fleet_availability(total_time_s, robot_count),
            tasks_impacted_by_failures: self.metrics.tasks_impacted_by_failures(),
        }
    }

    /// Generate heatmap data for congestion visualization
    pub fn generate_heatmap(&self) -> HeatmapData {
        let ts = &self.world.time_series;

        // Node congestion
        let node_congestion: Vec<NodeCongestion> = ts.node_congestion.iter().map(|(node_id, metrics)| {
            let (x, y) = self.world.map.get_node(*node_id)
                .map(|n| (n.x, n.y))
                .unwrap_or((0.0, 0.0));

            NodeCongestion {
                node_id: node_id.0,
                x,
                y,
                total_wait_time_s: metrics.total_wait_time_s,
                wait_event_count: metrics.wait_event_count,
                congestion_score: metrics.congestion_score(),
            }
        }).collect();

        // Edge congestion
        let edge_congestion: Vec<EdgeCongestion> = ts.edge_congestion.iter().map(|(edge_id, metrics)| {
            let (from, to) = self.world.map.get_edge(*edge_id)
                .map(|e| (e.from.0, e.to.0))
                .unwrap_or((0, 0));

            EdgeCongestion {
                edge_id: edge_id.0,
                from_node: from,
                to_node: to,
                total_wait_time_s: metrics.total_wait_time_s,
                wait_event_count: metrics.wait_event_count,
                congestion_score: metrics.congestion_score(),
            }
        }).collect();

        HeatmapData {
            node_congestion,
            edge_congestion,
        }
    }

    /// Generate a full report with optional v3 sections based on flags
    pub fn generate_full_report(
        &self,
        include_robots: bool,
        include_stations: bool,
        include_reliability: bool,
        include_heatmap: bool,
    ) -> SimulationReport {
        let mut report = self.generate_report();
        let duration_s = (self.kernel.now() - self.warmup_time).as_seconds().max(0.0);

        if include_robots {
            report = report.with_robot_reports(self.generate_robot_reports(duration_s));
        }

        if include_stations {
            report = report.with_station_reports(self.generate_station_reports(duration_s));
        }

        if include_reliability {
            report = report.with_reliability(self.generate_reliability_report(duration_s));
        }

        if include_heatmap {
            report = report.with_heatmap(self.generate_heatmap());
        }

        report
    }
}
