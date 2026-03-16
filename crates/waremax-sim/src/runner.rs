//! Simulation runner

use waremax_core::{Kernel, SimTime, SimEvent};
use waremax_metrics::{MetricsCollector, SimulationReport};
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

        SimulationReport::new(
            duration_s,
            self.kernel.events_processed(),
            self.metrics.orders_completed(),
            self.metrics.orders_late(),
            self.metrics.avg_cycle_time(),
            self.metrics.p95_cycle_time(),
            robot_utilization,
            station_utilization,
        )
    }

    /// Get reference to the world
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get current simulation time
    pub fn now(&self) -> SimTime {
        self.kernel.now()
    }
}
