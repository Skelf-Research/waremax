//! Controllable simulation for real-time web UI
//!
//! Provides a simulation runner that can be paused, resumed, and speed-controlled
//! for interactive visualization.

use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};

use waremax_config::ScenarioConfig;
use waremax_core::{Kernel, NodeId, RobotId, ScheduledEvent, SimEvent, SimTime};
use waremax_entities::Robot;
use waremax_metrics::MetricsCollector;
use waremax_sim::{EventHandler, World};
use waremax_testing::ScenarioBuilder;

use crate::types::{MetricsSnapshot, RobotState, SimulationState, SimulationStatus, StationState};

/// Control commands sent to the simulation task
#[derive(Clone, Debug)]
pub enum SimCommand {
    Pause,
    Resume,
    SetSpeed(f64),
    Step,
    AddRobot { node_id: Option<u32> },
    Stop,
    GetState,
}

/// Events emitted by the simulation
#[derive(Clone, Debug)]
pub enum SimUpdate {
    StateChanged(SimulationState),
    Tick {
        time_s: f64,
        events_processed: u64,
    },
    RobotMoved {
        robot_id: u32,
        from_node: u32,
        to_node: u32,
        time_s: f64,
    },
    RobotStateChanged {
        robot_id: u32,
        old_state: String,
        new_state: String,
        time_s: f64,
    },
    OrderCompleted {
        order_id: u32,
        cycle_time_s: f64,
        on_time: bool,
    },
    MetricsUpdate(MetricsSnapshot),
    Finished(MetricsSnapshot),
    Error(String),
}

/// Configuration for controllable simulation
#[derive(Clone, Debug)]
pub struct SimulationConfig {
    pub preset: String,
    pub robot_count: Option<u32>,
    pub order_rate: Option<f64>,
    pub duration_minutes: f64,
    pub grid_rows: Option<u32>,
    pub grid_cols: Option<u32>,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            preset: "standard".to_string(),
            robot_count: None,
            order_rate: None,
            duration_minutes: 60.0,
            grid_rows: None,
            grid_cols: None,
        }
    }
}

/// A controllable simulation that can be paused, resumed, and speed-controlled
pub struct ControllableSimulation {
    kernel: Kernel,
    world: World,
    handler: EventHandler,
    metrics: MetricsCollector,
    end_time: SimTime,
    warmup_time: SimTime,

    // Control state
    paused: bool,
    speed: f64,
    events_processed: u64,
    orders_completed: u64,

    // Communication channels
    command_rx: mpsc::Receiver<SimCommand>,
    update_tx: broadcast::Sender<SimUpdate>,

    // Timing
    last_update: Instant,
    update_interval: Duration,
}

impl ControllableSimulation {
    /// Create a new controllable simulation from config
    pub fn new(
        config: &SimulationConfig,
        command_rx: mpsc::Receiver<SimCommand>,
        update_tx: broadcast::Sender<SimUpdate>,
    ) -> Self {
        let scenario = Self::build_scenario(config);
        let seed = scenario.seed;

        let world = build_world_from_config(&scenario, seed);

        let end_time = SimTime::from_minutes(
            scenario.simulation.warmup_minutes + scenario.simulation.duration_minutes,
        );
        let warmup_time = SimTime::from_minutes(scenario.simulation.warmup_minutes);

        Self {
            kernel: Kernel::new(),
            world,
            handler: EventHandler::new(),
            metrics: MetricsCollector::new(),
            end_time,
            warmup_time,
            paused: true,
            speed: 1.0,
            events_processed: 0,
            orders_completed: 0,
            command_rx,
            update_tx,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100), // Update every 100ms
        }
    }

    /// Build scenario config from simulation config
    fn build_scenario(config: &SimulationConfig) -> ScenarioConfig {
        let mut builder = match config.preset.as_str() {
            "small" => ScenarioBuilder::new()
                .grid(5, 5)
                .robots(5)
                .pick_stations(2)
                .order_rate(30.0),
            "large" => ScenarioBuilder::new()
                .grid(15, 15)
                .robots(30)
                .pick_stations(8)
                .order_rate(120.0),
            _ => ScenarioBuilder::new()
                .grid(10, 10)
                .robots(15)
                .pick_stations(4)
                .order_rate(60.0),
        };

        if let Some(robots) = config.robot_count {
            builder = builder.robots(robots);
        }
        if let Some(rate) = config.order_rate {
            builder = builder.order_rate(rate);
        }
        if let (Some(rows), Some(cols)) = (config.grid_rows, config.grid_cols) {
            builder = builder.grid(rows, cols);
        }

        builder = builder.duration(config.duration_minutes).warmup(5.0);

        builder.build()
    }

    /// Initialize the simulation
    pub fn initialize(&mut self) {
        // Schedule first order arrival
        let first_order_id = self.world.next_order_id();
        self.kernel.schedule_now(SimEvent::OrderArrival {
            order_id: first_order_id,
        });

        // Place robots at their starting positions
        for robot in self.world.robots.values() {
            self.world.traffic.enter_node(robot.current_node, robot.id);
        }

        // Schedule first metrics sample tick
        if self.world.metrics_sample_interval_s > 0.0 {
            let sample_time = SimTime::from_seconds(self.world.metrics_sample_interval_s);
            self.kernel
                .schedule_after(sample_time, SimEvent::MetricsSampleTick);
        }
    }

    /// Run the simulation loop (async)
    pub async fn run(mut self) {
        self.initialize();

        // Send initial state
        let _ = self
            .update_tx
            .send(SimUpdate::StateChanged(self.get_state()));

        // Track wall-clock time for proper pacing
        let mut last_frame = Instant::now();
        let frame_duration = Duration::from_millis(16); // ~60fps
        let mut sim_time_anchor = self.kernel.now().as_seconds();
        let mut wall_time_anchor = Instant::now();

        loop {
            // Check for commands (non-blocking)
            loop {
                match self.command_rx.try_recv() {
                    Ok(cmd) => {
                        if !self.handle_command(cmd) {
                            return;
                        }
                        // Reset time anchors on resume
                        if !self.paused {
                            sim_time_anchor = self.kernel.now().as_seconds();
                            wall_time_anchor = Instant::now();
                        }
                    }
                    Err(mpsc::error::TryRecvError::Empty) => break,
                    Err(mpsc::error::TryRecvError::Disconnected) => return,
                }
            }

            if self.paused {
                // When paused, send state updates less frequently
                if self.last_update.elapsed() >= Duration::from_millis(500) {
                    let _ = self
                        .update_tx
                        .send(SimUpdate::StateChanged(self.get_state()));
                    self.last_update = Instant::now();
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            }

            // Check if simulation is finished
            if !self.kernel.has_events() || self.kernel.now() >= self.end_time {
                let final_metrics = self.compute_metrics();
                let _ = self
                    .update_tx
                    .send(SimUpdate::StateChanged(self.get_state()));
                let _ = self.update_tx.send(SimUpdate::Finished(final_metrics));
                break;
            }

            // Calculate target simulation time based on wall-clock elapsed time
            let wall_elapsed = wall_time_anchor.elapsed().as_secs_f64();
            let target_sim_time = sim_time_anchor + (wall_elapsed * self.speed);
            let target_time =
                SimTime::from_seconds(target_sim_time.min(self.end_time.as_seconds()));

            // Process events up to target time
            let mut events_this_frame = 0u64;
            let max_events_per_frame = 1000; // Prevent infinite loops

            while self.kernel.has_events()
                && self.kernel.now() <= target_time
                && events_this_frame < max_events_per_frame
            {
                // Peek at next event time
                if let Some(next_event) = self.kernel.peek_next() {
                    if next_event.time > target_time {
                        break; // Next event is in the future
                    }
                }

                if let Some(event) = self.kernel.pop_next() {
                    // Record metrics after warmup
                    if self.kernel.now() >= self.warmup_time {
                        self.metrics.record_event(&event);
                    }

                    // Track dashboard events for UI
                    self.track_event_for_ui(&event);

                    // Handle the event
                    self.handler.handle(
                        &mut self.kernel,
                        &mut self.world,
                        &event,
                        &mut self.metrics,
                    );

                    self.events_processed += 1;
                    events_this_frame += 1;
                }
            }

            // Send state updates at frame rate
            if self.last_update.elapsed() >= self.update_interval {
                let _ = self
                    .update_tx
                    .send(SimUpdate::StateChanged(self.get_state()));
                self.last_update = Instant::now();
            }

            // Wait for next frame (actual wall-clock pacing)
            let frame_elapsed = last_frame.elapsed();
            if frame_elapsed < frame_duration {
                tokio::time::sleep(frame_duration - frame_elapsed).await;
            }
            last_frame = Instant::now();
        }
    }

    /// Handle a control command
    fn handle_command(&mut self, cmd: SimCommand) -> bool {
        match cmd {
            SimCommand::Pause => {
                self.paused = true;
                let _ = self
                    .update_tx
                    .send(SimUpdate::StateChanged(self.get_state()));
            }
            SimCommand::Resume => {
                self.paused = false;
                let _ = self
                    .update_tx
                    .send(SimUpdate::StateChanged(self.get_state()));
            }
            SimCommand::SetSpeed(speed) => {
                self.speed = speed.clamp(0.1, 100.0);
                let _ = self
                    .update_tx
                    .send(SimUpdate::StateChanged(self.get_state()));
            }
            SimCommand::Step => {
                // Process exactly one event
                if let Some(event) = self.kernel.pop_next() {
                    if self.kernel.now() >= self.warmup_time {
                        self.metrics.record_event(&event);
                    }
                    self.track_event_for_ui(&event);
                    self.handler.handle(
                        &mut self.kernel,
                        &mut self.world,
                        &event,
                        &mut self.metrics,
                    );
                    self.events_processed += 1;
                }
                let _ = self
                    .update_tx
                    .send(SimUpdate::StateChanged(self.get_state()));
            }
            SimCommand::AddRobot { node_id } => {
                self.add_robot(node_id);
                let _ = self
                    .update_tx
                    .send(SimUpdate::StateChanged(self.get_state()));
            }
            SimCommand::GetState => {
                let _ = self
                    .update_tx
                    .send(SimUpdate::StateChanged(self.get_state()));
            }
            SimCommand::Stop => {
                return false;
            }
        }
        true
    }

    /// Track events for UI updates
    fn track_event_for_ui(&mut self, event: &ScheduledEvent) {
        match &event.event {
            SimEvent::RobotArriveNode {
                robot_id,
                node_id,
                from_node,
            } => {
                let _ = self.update_tx.send(SimUpdate::RobotMoved {
                    robot_id: robot_id.0,
                    from_node: from_node.0,
                    to_node: node_id.0,
                    time_s: self.kernel.now().as_seconds(),
                });
            }
            SimEvent::OutboundReady { order_id } => {
                // Get cycle time from completed order
                if let Some(order) = self.world.orders.get(order_id) {
                    self.orders_completed += 1;
                    let cycle_time_s = order.cycle_time().map(|t| t.as_seconds()).unwrap_or(0.0);
                    let on_time = !order.is_late(self.kernel.now());
                    let _ = self.update_tx.send(SimUpdate::OrderCompleted {
                        order_id: order_id.0,
                        cycle_time_s,
                        on_time,
                    });
                }
            }
            _ => {}
        }
    }

    /// Add a new robot to the simulation
    fn add_robot(&mut self, node_id: Option<u32>) {
        let new_id = RobotId(self.world.robots.len() as u32);
        let start_node = node_id.map(NodeId).unwrap_or_else(|| {
            // Find a node that isn't too crowded
            self.world
                .map
                .nodes
                .keys()
                .next()
                .copied()
                .unwrap_or(NodeId(0))
        });

        let robot = Robot::new(new_id, start_node, 1.5, 25.0);
        self.world.traffic.enter_node(start_node, new_id);
        self.world.robots.insert(new_id, robot);
    }

    /// Get current simulation state
    fn get_state(&self) -> SimulationState {
        let status = if !self.kernel.has_events() || self.kernel.now() >= self.end_time {
            SimulationStatus::Finished
        } else if self.paused {
            SimulationStatus::Paused
        } else {
            SimulationStatus::Running
        };

        let robots: Vec<RobotState> = self
            .world
            .robots
            .values()
            .map(|r| RobotState {
                id: r.id.0,
                node_id: r.current_node.0,
                state: format!("{:?}", r.state)
                    .split_whitespace()
                    .next()
                    .unwrap_or("Unknown")
                    .to_string(),
                battery_soc: if r.battery.capacity_wh > 0.0 {
                    Some(r.battery.soc)
                } else {
                    None
                },
                current_task: r.current_task.map(|t| t.0),
                is_failed: r.maintenance.is_failed,
            })
            .collect();

        let stations: Vec<StationState> = self
            .world
            .stations
            .values()
            .map(|s| StationState {
                id: s.id.0,
                name: s.string_id.clone(),
                node_id: s.node.0,
                station_type: format!("{:?}", s.station_type),
                queue_length: s.queue.len(),
                serving_count: s.serving.len(),
                concurrency: s.concurrency,
            })
            .collect();

        SimulationState {
            status,
            time_s: self.kernel.now().as_seconds(),
            speed: self.speed,
            events_processed: self.events_processed,
            orders_completed: self.orders_completed,
            robots,
            stations,
            metrics: self.compute_metrics(),
        }
    }

    /// Compute current metrics snapshot
    fn compute_metrics(&self) -> MetricsSnapshot {
        // Use time since warmup for throughput calculation
        let post_warmup_s = (self.kernel.now() - self.warmup_time).as_seconds().max(1.0);
        let hours = post_warmup_s / 3600.0;

        let throughput = if hours > 0.0 {
            self.orders_completed as f64 / hours
        } else {
            0.0
        };

        // For utilization, use FULL simulation time since robot/station stats accumulate from start
        let total_sim_time = self.kernel.now().as_seconds().max(1.0);

        // Calculate robot utilization
        let total_robot_time = total_sim_time * self.world.robots.len() as f64;
        let total_active_time: f64 = self
            .world
            .robots
            .values()
            .map(|r| r.total_move_time.as_seconds() + r.total_service_time.as_seconds())
            .sum();
        let robot_utilization = if total_robot_time > 0.0 {
            (total_active_time / total_robot_time).min(1.0) // Cap at 100%
        } else {
            0.0
        };

        // Calculate station utilization
        let total_station_capacity: f64 = self
            .world
            .stations
            .values()
            .map(|s| s.concurrency as f64 * total_sim_time)
            .sum();
        let total_station_busy: f64 = self
            .world
            .stations
            .values()
            .map(|s| s.total_service_time.as_seconds())
            .sum();
        let station_utilization = if total_station_capacity > 0.0 {
            (total_station_busy / total_station_capacity).min(1.0) // Cap at 100%
        } else {
            0.0
        };

        MetricsSnapshot {
            throughput_per_hour: throughput,
            orders_completed: self.orders_completed,
            orders_pending: self
                .world
                .orders
                .values()
                .filter(|o| !o.is_complete())
                .count() as u64,
            robot_utilization,
            station_utilization,
            avg_cycle_time_s: self.metrics.avg_cycle_time(),
            late_orders: self.metrics.orders_late() as u64,
        }
    }

    /// Get the world map data for frontend rendering
    pub fn get_map_data(&self) -> crate::types::MapData {
        use crate::types::{EdgeData, MapBounds, MapData, NodeData};

        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        let nodes: Vec<NodeData> = self
            .world
            .map
            .nodes
            .values()
            .map(|n| {
                min_x = min_x.min(n.x);
                max_x = max_x.max(n.x);
                min_y = min_y.min(n.y);
                max_y = max_y.max(n.y);

                NodeData {
                    id: n.id.0,
                    name: n.string_id.clone(),
                    x: n.x,
                    y: n.y,
                    node_type: format!("{:?}", n.node_type),
                }
            })
            .collect();

        let edges: Vec<EdgeData> = self
            .world
            .map
            .edges
            .values()
            .map(|e| EdgeData {
                id: e.id.0,
                from: e.from.0,
                to: e.to.0,
                length: e.length_m,
                bidirectional: matches!(e.direction, waremax_map::EdgeDirection::Bidirectional),
            })
            .collect();

        // Add padding to bounds
        let padding = 2.0;
        MapData {
            nodes,
            edges,
            bounds: MapBounds {
                min_x: min_x - padding,
                max_x: max_x + padding,
                min_y: min_y - padding,
                max_y: max_y + padding,
            },
        }
    }
}

/// Build a World from ScenarioConfig (simplified version for UI)
fn build_world_from_config(scenario: &ScenarioConfig, seed: u64) -> World {
    use waremax_core::{EdgeId, NodeId, RobotId, StationId};
    use waremax_entities::{
        BatteryConsumptionModel, Robot, ServiceTimeModel, Station, StationType,
    };
    use waremax_map::{Edge, Node, NodeType, Router, TrafficManager, WarehouseMap};
    use waremax_metrics::TimeSeriesCollector;

    let mut world = World::new(seed);

    // Determine grid size based on preset
    let grid_rows = match scenario.stations.len() {
        0..=2 => 5,  // small preset
        3..=5 => 10, // standard preset
        _ => 15,     // large preset
    };
    let grid_cols = grid_rows;
    let grid_size = grid_rows;
    let spacing = 3.0;

    // Calculate station positions - distribute them around the perimeter
    let num_stations = scenario.stations.len();
    let mut station_nodes: std::collections::HashSet<u32> = std::collections::HashSet::new();

    for i in 0..num_stations {
        // Place stations along the edges of the grid, evenly spaced
        let node_id = match i % 4 {
            0 => {
                // Top edge
                let col = (i / 4 + 1) * grid_cols / (num_stations / 4 + 2);
                col as u32
            }
            1 => {
                // Right edge
                let row = (i / 4 + 1) * grid_rows / (num_stations / 4 + 2);
                (row * grid_cols + grid_cols - 1) as u32
            }
            2 => {
                // Bottom edge
                let col = (i / 4 + 1) * grid_cols / (num_stations / 4 + 2);
                ((grid_rows - 1) * grid_cols + col) as u32
            }
            _ => {
                // Left edge
                let row = (i / 4 + 1) * grid_rows / (num_stations / 4 + 2);
                (row * grid_cols) as u32
            }
        };
        station_nodes.insert(node_id);
    }

    // Helper to determine if a position should be a rack (storage) node
    // Creates a warehouse-like layout with alternating rack/aisle rows
    fn is_rack_position(row: usize, col: usize, grid_size: usize) -> bool {
        // Skip first/last rows (perimeter for stations/aisles)
        if row == 0 || row == grid_size - 1 {
            return false;
        }
        // Skip first/last cols (perimeter)
        if col == 0 || col == grid_size - 1 {
            return false;
        }
        // Every other row is a rack row (rows 2, 4, 6, etc.)
        let is_rack_row = row % 2 == 0;
        // Every 3rd column is a main aisle, others can be racks
        let is_rack_col = col % 3 != 0;
        is_rack_row && is_rack_col
    }

    // Build the grid map with proper warehouse layout
    let mut map = WarehouseMap::new();

    for row in 0..grid_size {
        for col in 0..grid_size {
            let id = (row * grid_size + col) as u32;
            let x = col as f64 * spacing;
            let y = row as f64 * spacing;
            let node_type = if station_nodes.contains(&id) {
                NodeType::StationPick
            } else if is_rack_position(row, col, grid_size) {
                NodeType::Rack // Interior storage nodes
            } else {
                NodeType::Aisle
            };
            let node = Node::new(NodeId(id), format!("N{}", id), x, y, node_type);
            map.add_node(node);
        }
    }

    let mut edge_id = 0u32;
    for row in 0..grid_size {
        for col in 0..grid_size {
            let id = row * grid_size + col;
            if col < grid_size - 1 {
                let neighbor = id + 1;
                map.add_edge(Edge::new(
                    EdgeId(edge_id),
                    NodeId(id as u32),
                    NodeId(neighbor as u32),
                    spacing,
                ));
                edge_id += 1;
            }
            if row < grid_size - 1 {
                let neighbor = id + grid_size;
                map.add_edge(Edge::new(
                    EdgeId(edge_id),
                    NodeId(id as u32),
                    NodeId(neighbor as u32),
                    spacing,
                ));
                edge_id += 1;
            }
        }
    }

    world.map = map;
    world.router = Router::new(true);
    world.traffic = TrafficManager::new(
        scenario.traffic.node_capacity_default,
        scenario.traffic.edge_capacity_default,
    );

    let total_nodes = (grid_size * grid_size) as u32;

    // Add robots - distribute them across the grid
    for i in 0..scenario.robots.count {
        // Spread robots across the grid, avoiding station nodes initially
        let mut start_node = ((i * 7) % total_nodes) as u32; // Use prime multiplier for better spread
                                                             // Skip station nodes for initial placement
        while station_nodes.contains(&start_node) {
            start_node = (start_node + 1) % total_nodes;
        }
        let robot = if scenario.robots.battery.enabled {
            Robot::with_battery(
                RobotId(i),
                NodeId(start_node),
                scenario.robots.max_speed_mps,
                scenario.robots.max_payload_kg,
                scenario.robots.battery.capacity_wh,
                scenario.robots.battery.min_soc,
                BatteryConsumptionModel {
                    per_meter_wh: scenario.robots.battery.consumption.per_meter_wh,
                    per_kg_per_meter_wh: scenario.robots.battery.consumption.per_kg_per_meter_wh,
                    idle_power_w: scenario.robots.battery.consumption.idle_power_w,
                    service_power_w: scenario.robots.battery.consumption.service_power_w,
                },
            )
        } else {
            Robot::new(
                RobotId(i),
                NodeId(start_node),
                scenario.robots.max_speed_mps,
                scenario.robots.max_payload_kg,
            )
        };
        world.robots.insert(RobotId(i), robot);
    }

    // Add stations - use the distributed positions we calculated
    let station_node_vec: Vec<u32> = station_nodes.iter().cloned().collect();
    for (idx, station_cfg) in scenario.stations.iter().enumerate() {
        let station_type = match station_cfg.station_type.as_str() {
            "pick" => StationType::Pick,
            "drop" => StationType::Drop,
            "inbound" => StationType::Inbound,
            "outbound" => StationType::Outbound,
            _ => StationType::Pick,
        };

        let service_time = match station_cfg.service_time_s.distribution.as_str() {
            "lognormal" => ServiceTimeModel::lognormal(
                station_cfg.service_time_s.base,
                station_cfg.service_time_s.base_stddev,
                station_cfg.service_time_s.per_item,
                station_cfg.service_time_s.per_item_stddev,
            ),
            _ => ServiceTimeModel::constant(
                station_cfg.service_time_s.base,
                station_cfg.service_time_s.per_item,
            ),
        };

        // Use our calculated distributed positions instead of config (which puts them adjacent)
        let node_id = if idx < station_node_vec.len() {
            station_node_vec[idx]
        } else {
            station_cfg.node.parse().unwrap_or(0)
        };

        let station = Station::new(
            StationId(idx as u32),
            station_cfg.id.clone(),
            NodeId(node_id),
            station_type,
            station_cfg.concurrency,
            station_cfg.queue_capacity,
            service_time,
        );
        world.stations.insert(StationId(idx as u32), station);
    }

    // Set due time offset
    world.due_time_offset_min = scenario.orders.due_times.as_ref().map(|d| d.minutes);

    // Create distributions from config
    world.distributions = waremax_sim::create_distributions(&scenario.orders);

    // Create policies
    world.policies =
        waremax_sim::create_policies_with_traffic(&scenario.policies, &scenario.traffic);

    // Set metrics sample interval
    world.metrics_sample_interval_s = scenario.metrics.sample_interval_s;
    world.time_series = TimeSeriesCollector::new(scenario.metrics.sample_interval_s);

    // Initialize demo inventory
    // Initialize inventory with more SKUs to spread across rack nodes
    world.init_demo_inventory(50);

    world
}

/// Handle to control a running simulation
pub struct SimulationHandle {
    command_tx: mpsc::Sender<SimCommand>,
    update_rx: broadcast::Receiver<SimUpdate>,
    map_data: crate::types::MapData,
}

impl SimulationHandle {
    /// Send a control command to the simulation
    pub async fn send_command(
        &self,
        cmd: SimCommand,
    ) -> Result<(), mpsc::error::SendError<SimCommand>> {
        self.command_tx.send(cmd).await
    }

    /// Subscribe to simulation updates
    pub fn subscribe(&self) -> broadcast::Receiver<SimUpdate> {
        self.update_rx.resubscribe()
    }

    /// Get map data (doesn't change during simulation)
    pub fn map_data(&self) -> &crate::types::MapData {
        &self.map_data
    }

    /// Pause the simulation
    pub async fn pause(&self) -> Result<(), mpsc::error::SendError<SimCommand>> {
        self.send_command(SimCommand::Pause).await
    }

    /// Resume the simulation
    pub async fn resume(&self) -> Result<(), mpsc::error::SendError<SimCommand>> {
        self.send_command(SimCommand::Resume).await
    }

    /// Set simulation speed
    pub async fn set_speed(&self, speed: f64) -> Result<(), mpsc::error::SendError<SimCommand>> {
        self.send_command(SimCommand::SetSpeed(speed)).await
    }

    /// Step one event
    pub async fn step(&self) -> Result<(), mpsc::error::SendError<SimCommand>> {
        self.send_command(SimCommand::Step).await
    }

    /// Add a robot
    pub async fn add_robot(
        &self,
        node_id: Option<u32>,
    ) -> Result<(), mpsc::error::SendError<SimCommand>> {
        self.send_command(SimCommand::AddRobot { node_id }).await
    }

    /// Stop the simulation
    pub async fn stop(&self) -> Result<(), mpsc::error::SendError<SimCommand>> {
        self.send_command(SimCommand::Stop).await
    }

    /// Request current state
    pub async fn get_state(&self) -> Result<(), mpsc::error::SendError<SimCommand>> {
        self.send_command(SimCommand::GetState).await
    }
}

/// Spawn a new controllable simulation and return a handle to control it
pub fn spawn_simulation(
    config: SimulationConfig,
) -> (SimulationHandle, tokio::task::JoinHandle<()>) {
    let (command_tx, command_rx) = mpsc::channel(32);
    let (update_tx, update_rx) = broadcast::channel(256);

    let sim = ControllableSimulation::new(&config, command_rx, update_tx.clone());
    let map_data = sim.get_map_data();

    let handle = SimulationHandle {
        command_tx,
        update_rx,
        map_data,
    };

    let task = tokio::spawn(async move {
        sim.run().await;
    });

    (handle, task)
}
