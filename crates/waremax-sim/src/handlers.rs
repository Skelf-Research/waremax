//! Event handlers for simulation events

use crate::world::World;
use waremax_analysis::DelayCategory;
use waremax_core::{
    ChargingStationId, Kernel, MaintenanceStationId, OrderId, RobotId, ScheduledEvent, SimEvent,
    SimTime, TaskId,
};
use waremax_entities::{BinLocation, Order, OrderLine, RobotState, Task};
use waremax_map::ReservableResource;
use waremax_metrics::{MetricsCollector, TraceDetails};

/// Handles simulation events and produces new events
pub struct EventHandler {
    /// Tracks assigned robots to avoid double-assignment in dispatch
    assigned_this_round: Vec<waremax_core::RobotId>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            assigned_this_round: Vec::new(),
        }
    }

    /// Handle an event and schedule any resulting events
    pub fn handle(
        &mut self,
        kernel: &mut Kernel,
        world: &mut World,
        event: &ScheduledEvent,
        metrics: &mut MetricsCollector,
    ) {
        let current_time = event.time;

        match &event.event {
            SimEvent::OrderArrival { order_id } => {
                self.handle_order_arrival(kernel, world, current_time, *order_id);
            }
            SimEvent::TaskAssignment { task_id, robot_id } => {
                self.handle_task_assignment(kernel, world, current_time, *task_id, *robot_id);
            }
            SimEvent::RobotDepartNode {
                robot_id,
                from_node,
                to_node,
                edge_id,
            } => {
                self.handle_robot_depart(
                    kernel,
                    world,
                    current_time,
                    *robot_id,
                    *from_node,
                    *to_node,
                    *edge_id,
                    metrics,
                );
            }
            SimEvent::RobotArriveNode {
                robot_id,
                node_id,
                from_node,
            } => {
                self.handle_robot_arrive(
                    kernel,
                    world,
                    current_time,
                    *robot_id,
                    *node_id,
                    *from_node,
                );
            }
            SimEvent::StationServiceStart {
                robot_id,
                station_id,
                task_id,
            } => {
                self.handle_service_start(
                    kernel,
                    world,
                    current_time,
                    *robot_id,
                    *station_id,
                    *task_id,
                );
            }
            SimEvent::StationServiceEnd {
                robot_id,
                station_id,
                task_id,
            } => {
                self.handle_service_end(
                    kernel,
                    world,
                    current_time,
                    *robot_id,
                    *station_id,
                    *task_id,
                    metrics,
                );
            }
            SimEvent::DispatchTasks => {
                self.handle_dispatch_tasks(kernel, world, current_time);
            }
            // v1: Metrics sampling
            SimEvent::MetricsSampleTick => {
                self.handle_metrics_sample_tick(kernel, world, current_time);
            }
            // v1: Battery and charging
            SimEvent::RobotLowBattery { robot_id, soc: _ } => {
                self.handle_robot_low_battery(kernel, world, current_time, *robot_id);
            }
            SimEvent::RobotChargingStart {
                robot_id,
                station_id,
            } => {
                self.handle_robot_charging_start(
                    kernel,
                    world,
                    current_time,
                    *robot_id,
                    *station_id,
                    metrics,
                );
            }
            SimEvent::RobotChargingEnd {
                robot_id,
                station_id,
                energy_charged_wh,
            } => {
                self.handle_robot_charging_end(
                    kernel,
                    world,
                    current_time,
                    *robot_id,
                    *station_id,
                    *energy_charged_wh,
                );
            }
            // v2: Deadlock detection
            SimEvent::DeadlockDetected { robots } => {
                self.handle_deadlock_detected(kernel, world, current_time, robots.clone());
            }
            SimEvent::DeadlockResolved {
                robots,
                resolver_robot,
            } => {
                self.handle_deadlock_resolved(kernel, world, *resolver_robot, robots.clone());
            }
            // v3: Robot Failures & Maintenance
            SimEvent::RobotFailure {
                robot_id,
                interrupted_task,
            } => {
                self.handle_robot_failure(
                    kernel,
                    world,
                    current_time,
                    *robot_id,
                    *interrupted_task,
                    metrics,
                );
            }
            SimEvent::RobotMaintenanceDue {
                robot_id,
                operating_hours: _,
            } => {
                self.handle_robot_maintenance_due(kernel, world, current_time, *robot_id);
            }
            SimEvent::MaintenanceStart {
                robot_id,
                station_id,
                is_repair,
            } => {
                self.handle_maintenance_start(
                    kernel,
                    world,
                    current_time,
                    *robot_id,
                    *station_id,
                    *is_repair,
                    metrics,
                );
            }
            SimEvent::MaintenanceEnd {
                robot_id,
                station_id,
                is_repair,
                duration_s,
            } => {
                self.handle_maintenance_end(
                    kernel,
                    world,
                    current_time,
                    *robot_id,
                    *station_id,
                    *is_repair,
                    *duration_s,
                    metrics,
                );
            }
            _ => {
                // Handle other events as needed (inbound/outbound flow - future)
            }
        }
    }

    fn handle_order_arrival(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        order_id: OrderId,
    ) {
        // Schedule next order arrival using configured distribution
        let interarrival = world
            .distributions
            .arrivals
            .next_interarrival(&mut world.rng);
        let next_order_id = world.next_order_id();
        kernel.schedule_after(
            SimTime::from_seconds(interarrival),
            SimEvent::OrderArrival {
                order_id: next_order_id,
            },
        );

        // Generate order lines using configured distribution
        let num_skus = world.skus.count().max(1);
        let num_lines = world.distributions.lines.next_lines(&mut world.rng);
        let num_lines = num_lines.min(10); // Cap at 10 lines per order

        let mut order_lines = Vec::new();
        let mut tasks_to_create = Vec::new();

        // Check that at least one pick station exists
        if world.pick_stations().next().is_none() {
            // No pick stations - can't process orders
            return;
        }

        for _ in 0..num_lines {
            // Pick a SKU using configured distribution
            let sku_idx = world
                .distributions
                .skus
                .next_sku(&mut world.rng, num_skus as u32);
            let sku_id = waremax_core::SkuId(sku_idx);

            // Random quantity 1-5
            let quantity = world.rng.gen_range(1..=5u32);

            // Create order line
            order_lines.push(OrderLine::new(sku_id, quantity));

            // Find inventory location for this SKU
            if let Some((bin_addr, access_node)) = world.find_sku_location(sku_id, quantity) {
                let task_id = world.next_task_id();
                let bin_location = BinLocation::new(bin_addr, access_node);

                // Use station assignment policy to select destination station
                // Create a temporary task to pass to the policy
                let temp_task = Task::new_pick(
                    task_id,
                    order_id,
                    sku_id,
                    quantity,
                    bin_location.clone(),
                    waremax_core::StationId(0), // Placeholder
                    current_time,
                );

                let ctx = world.policy_context(current_time);
                if let Some(assigned_station) =
                    world.policies.station_assignment.assign(&ctx, &temp_task)
                {
                    tasks_to_create.push((
                        task_id,
                        sku_id,
                        quantity,
                        bin_location,
                        assigned_station,
                    ));
                }
            }
        }

        // Calculate due time if configured
        let due_time = world
            .due_time_offset_min
            .map(|offset| current_time + SimTime::from_minutes(offset));

        // Only create orders that have at least one task that can be fulfilled
        if tasks_to_create.is_empty() {
            return;
        }

        // Create the order - set tasks_total to actual number of tasks we can create
        let mut order = Order::new(order_id, current_time, order_lines, due_time);
        order.tasks_total = tasks_to_create.len() as u32; // Override with actual task count
        world.orders.insert(order_id, order);

        // Create tasks for each line that has inventory
        for (task_id, sku_id, quantity, bin_location, dest_station) in tasks_to_create {
            let task = Task::new_pick(
                task_id,
                order_id,
                sku_id,
                quantity,
                bin_location,
                dest_station,
                current_time,
            );
            world.tasks.insert(task_id, task);
            world.pending_tasks.push(task_id);

            // Attribution tracking: start tracking this task and begin assignment wait phase
            if world.attribution_collector.is_enabled() {
                world
                    .attribution_collector
                    .start_task(task_id, Some(order_id), current_time);
                world.attribution_collector.start_phase(
                    task_id,
                    DelayCategory::RobotAssignment,
                    current_time,
                );
            }
        }

        // Schedule task dispatch
        kernel.schedule_now(SimEvent::DispatchTasks);
    }

    fn handle_task_assignment(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        task_id: waremax_core::TaskId,
        robot_id: waremax_core::RobotId,
    ) {
        // Assign task to robot
        if let Some(task) = world.get_task_mut(task_id) {
            task.assign(robot_id, current_time);
        }

        if let Some(robot) = world.get_robot_mut(robot_id) {
            robot.start_task(task_id);
        }

        // Attribution tracking: record robot assignment and start travel phase
        if world.attribution_collector.is_enabled() {
            world
                .attribution_collector
                .record_robot_assignment(task_id, robot_id, current_time);
            world.attribution_collector.start_phase(
                task_id,
                DelayCategory::TravelToPickup,
                current_time,
            );
        }

        // v3: Trace task assignment
        world.trace_collector.record_sampled(
            current_time,
            "TaskAssign",
            TraceDetails::TaskAssign {
                task_id: task_id.0,
                robot_id: robot_id.0,
            },
        );

        // Get task and robot info needed for routing
        let route_info = {
            let task = world.get_task(task_id);
            let robot = world.get_robot(robot_id);
            match (task, robot) {
                (Some(t), Some(r)) => Some((t.source.access_node, r.current_node, r.max_speed_mps)),
                _ => None,
            }
        };

        // Start moving robot to pickup location
        if let Some((pickup_node, robot_node, robot_speed)) = route_info {
            if let Some(route) = world.router.find_route(&world.map, robot_node, pickup_node) {
                // v2: Reserve path segments if reservation system is enabled
                if world.reservation_manager.enabled {
                    let mut time_offset = current_time;

                    for window in route.path.windows(2) {
                        let from_node = window[0];
                        let to_node = window[1];

                        // Find edge between nodes
                        if let Some((_, edge_id, length)) = world
                            .map
                            .neighbors(from_node)
                            .find(|(n, _, _)| *n == to_node)
                        {
                            let travel_time = SimTime::from_seconds(length / robot_speed);
                            let end_time = time_offset + travel_time;

                            // Reserve the edge (ignore errors - best effort)
                            let _ = world.reservation_manager.reserve(
                                ReservableResource::Edge(edge_id),
                                robot_id,
                                time_offset,
                                end_time,
                            );

                            time_offset = end_time;
                        }
                    }
                }

                if let Some(robot) = world.get_robot_mut(robot_id) {
                    robot.set_path(route.path);
                }
                // Start movement
                if let Some(robot) = world.get_robot(robot_id) {
                    if let Some(next_node) = robot.next_node_in_path() {
                        // Find edge to next node
                        if let Some((_, edge_id, _)) = world
                            .map
                            .neighbors(robot.current_node)
                            .find(|(n, _, _)| *n == next_node)
                        {
                            kernel.schedule_now(SimEvent::RobotDepartNode {
                                robot_id,
                                from_node: robot.current_node,
                                to_node: next_node,
                                edge_id,
                            });
                        }
                    } else {
                        // Already at destination - go to service
                        let dest_station = world.get_task(task_id).map(|t| t.destination_station);
                        if let Some(station_id) = dest_station {
                            kernel.schedule_now(SimEvent::StationServiceStart {
                                robot_id,
                                station_id,
                                task_id,
                            });
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_robot_depart(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        robot_id: waremax_core::RobotId,
        from_node: waremax_core::NodeId,
        to_node: waremax_core::NodeId,
        edge_id: waremax_core::EdgeId,
        _metrics: &mut MetricsCollector,
    ) {
        // Check if edge is available
        if !world.traffic.can_enter_edge(edge_id, robot_id) {
            // v1: Record wait event for congestion metrics
            world
                .time_series
                .record_edge_wait(edge_id, SimTime::from_seconds(0.5));

            // v2: Record wait in wait-for graph for deadlock detection
            world.traffic.record_edge_wait(robot_id, edge_id);

            // v2: Check for deadlock
            if let Some(cycle) = world.traffic.check_deadlock() {
                kernel.schedule_now(SimEvent::DeadlockDetected { robots: cycle });
            }

            // Schedule wait and retry
            kernel.schedule_after(
                SimTime::from_seconds(0.5),
                SimEvent::RobotDepartNode {
                    robot_id,
                    from_node,
                    to_node,
                    edge_id,
                },
            );
            return;
        }

        // v2: Clear wait status since we successfully entered the edge
        world.traffic.clear_wait(robot_id);

        // Update traffic
        world.traffic.leave_node(from_node, robot_id);
        world.traffic.enter_edge(edge_id, robot_id);

        // v1: Record edge traversal for congestion metrics
        world.time_series.record_edge_traversal(edge_id);

        // Get edge length for battery consumption
        let edge_length = world
            .map
            .get_edge(edge_id)
            .map(|e| e.length_m)
            .unwrap_or(0.0);

        // Update robot state and consume battery
        if let Some(robot) = world.get_robot_mut(robot_id) {
            robot.state = RobotState::Moving {
                destination: to_node,
            };
            robot.update_stats(current_time);

            // v1: Consume battery energy for travel
            robot.consume_travel_energy(edge_length);

            // v1: Check if battery is low after movement
            if robot.needs_charging() && !robot.seeking_charging {
                let soc = robot.soc();
                kernel.schedule_now(SimEvent::RobotLowBattery { robot_id, soc });
            }
        }

        // Calculate travel time and schedule arrival
        if let (Some(robot), Some(edge)) = (world.get_robot(robot_id), world.map.get_edge(edge_id))
        {
            let travel_time = robot.travel_time(edge.length_m);
            kernel.schedule_after(
                travel_time,
                SimEvent::RobotArriveNode {
                    robot_id,
                    node_id: to_node,
                    from_node,
                },
            );
        }
    }

    fn handle_robot_arrive(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        robot_id: waremax_core::RobotId,
        node_id: waremax_core::NodeId,
        from_node: waremax_core::NodeId,
    ) {
        // Leave the edge we just traversed
        if let Some((_, edge_id, _)) = world
            .map
            .neighbors(from_node)
            .find(|(n, _, _)| *n == node_id)
        {
            world.traffic.leave_edge(edge_id, robot_id);
        }

        // Update traffic and robot state
        world.traffic.enter_node(node_id, robot_id);

        if let Some(robot) = world.get_robot_mut(robot_id) {
            robot.current_node = node_id;
            robot.advance_path();
            robot.update_stats(current_time);
        }

        // v3: Trace robot movement
        world.trace_collector.record_sampled(
            current_time,
            "RobotMove",
            TraceDetails::RobotMove {
                robot_id: robot_id.0,
                from_node: from_node.0,
                to_node: node_id.0,
            },
        );

        // Check if we've reached destination
        let (has_reached, current_task, next_node) = {
            let robot = world.get_robot(robot_id);
            (
                robot.map(|r| r.has_reached_destination()).unwrap_or(true),
                robot.and_then(|r| r.current_task),
                robot.and_then(|r| r.next_node_in_path()),
            )
        };

        if has_reached {
            // Check what to do at this destination
            if let Some(task_id) = current_task {
                // Extract task info first to avoid borrow conflicts
                let task_info = world
                    .get_task(task_id)
                    .map(|task| (task.source.access_node, task.destination_station));

                if let Some((source_node, destination_station)) = task_info {
                    if node_id == source_node {
                        // At pickup - end TravelToPickup, start TravelToStation
                        if world.attribution_collector.is_enabled() {
                            // End travel-to-pickup phase, start travel-to-station
                            world.attribution_collector.start_phase(
                                task_id,
                                DelayCategory::TravelToStation,
                                current_time,
                            );
                        }

                        // Go to station
                        let station_node = world.get_station(destination_station).map(|s| s.node);
                        let robot_current = world.get_robot(robot_id).map(|r| r.current_node);

                        if let (Some(station_node), Some(robot_current)) =
                            (station_node, robot_current)
                        {
                            if let Some(route) =
                                world
                                    .router
                                    .find_route(&world.map, robot_current, station_node)
                            {
                                if let Some(robot) = world.get_robot_mut(robot_id) {
                                    robot.set_path(route.path);
                                }
                            }
                        }

                        // Continue to next node or arrive at station
                        if let Some(robot) = world.get_robot(robot_id) {
                            if let Some(next) = robot.next_node_in_path() {
                                if let Some((_, edge_id, _)) = world
                                    .map
                                    .neighbors(robot.current_node)
                                    .find(|(n, _, _)| *n == next)
                                {
                                    kernel.schedule_now(SimEvent::RobotDepartNode {
                                        robot_id,
                                        from_node: robot.current_node,
                                        to_node: next,
                                        edge_id,
                                    });
                                }
                            } else {
                                // At station - end travel phase, start queue/service phase
                                if world.attribution_collector.is_enabled() {
                                    world.attribution_collector.start_phase(
                                        task_id,
                                        DelayCategory::StationQueue,
                                        current_time,
                                    );
                                }
                                kernel.schedule_now(SimEvent::StationServiceStart {
                                    robot_id,
                                    station_id: destination_station,
                                    task_id,
                                });
                            }
                        }
                    } else {
                        // At station - end travel phase, start queue/service phase
                        if world.attribution_collector.is_enabled() {
                            world.attribution_collector.start_phase(
                                task_id,
                                DelayCategory::StationQueue,
                                current_time,
                            );
                        }
                        kernel.schedule_now(SimEvent::StationServiceStart {
                            robot_id,
                            station_id: destination_station,
                            task_id,
                        });
                    }
                }
            }
        } else if let Some(next) = next_node {
            // Continue along path
            if let Some(robot) = world.get_robot(robot_id) {
                if let Some((_, edge_id, _)) = world
                    .map
                    .neighbors(robot.current_node)
                    .find(|(n, _, _)| *n == next)
                {
                    kernel.schedule_now(SimEvent::RobotDepartNode {
                        robot_id,
                        from_node: robot.current_node,
                        to_node: next,
                        edge_id,
                    });
                }
            }
        }
    }

    fn handle_service_start(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        robot_id: waremax_core::RobotId,
        station_id: waremax_core::StationId,
        task_id: waremax_core::TaskId,
    ) {
        // Check if robot is already being served (started from queue in handle_service_end)
        let already_serving = world
            .get_station(station_id)
            .map(|s| s.is_robot_being_served(robot_id))
            .unwrap_or(false);
        let can_serve = world
            .get_station(station_id)
            .map(|s| s.can_serve())
            .unwrap_or(false);

        if already_serving || can_serve {
            // Start service (only add to serving if not already there)
            if !already_serving {
                if let Some(station) = world.get_station_mut(station_id) {
                    station.serving.push(robot_id);
                }
            }

            if let Some(robot) = world.get_robot_mut(robot_id) {
                robot.state = waremax_entities::RobotState::Servicing {
                    at_station: station_id,
                };
                robot.update_stats(current_time);
            }

            // Attribution tracking: end queue phase, start service phase
            if world.attribution_collector.is_enabled() {
                world.attribution_collector.start_phase(
                    task_id,
                    DelayCategory::StationService,
                    current_time,
                );
            }

            // Get task quantity and bin location for service time calculation
            let (item_count, level_access_time) = world
                .get_task(task_id)
                .map(|t| {
                    let access_time = world
                        .racks
                        .get(&t.source.bin_address.rack_id)
                        .map(|rack| rack.access_time(t.source.bin_address.level))
                        .unwrap_or(0.0);
                    (t.quantity, access_time)
                })
                .unwrap_or((1, 0.0));

            // Calculate service time based on item count plus level access time
            let base_service_time = world
                .get_station(station_id)
                .map(|s| s.service_time.calculate(item_count))
                .unwrap_or(SimTime::from_seconds(10.0));
            let service_time = base_service_time + SimTime::from_seconds(level_access_time);

            kernel.schedule_after(
                service_time,
                SimEvent::StationServiceEnd {
                    robot_id,
                    station_id,
                    task_id,
                },
            );
        } else {
            // Queue the robot (StationQueue phase was already started in handle_robot_arrive)
            if let Some(station) = world.get_station_mut(station_id) {
                station.enqueue(robot_id);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_service_end(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        robot_id: waremax_core::RobotId,
        station_id: waremax_core::StationId,
        task_id: waremax_core::TaskId,
        metrics: &mut MetricsCollector,
    ) {
        // Get service time from task quantity plus level access time
        let (item_count, level_access_time) = world
            .get_task(task_id)
            .map(|t| {
                let access_time = world
                    .racks
                    .get(&t.source.bin_address.rack_id)
                    .map(|rack| rack.access_time(t.source.bin_address.level))
                    .unwrap_or(0.0);
                (t.quantity, access_time)
            })
            .unwrap_or((1, 0.0));
        let base_service_time = world
            .get_station(station_id)
            .map(|s| s.service_time.calculate(item_count))
            .unwrap_or(SimTime::from_seconds(10.0));
        let service_time = base_service_time + SimTime::from_seconds(level_access_time);

        // Complete service with actual service time
        if let Some(station) = world.get_station_mut(station_id) {
            station.end_service(robot_id, service_time);
        }

        // Get order_id from task before completing
        let order_id = world.get_task(task_id).and_then(|t| t.order_id);

        // Complete task
        if let Some(task) = world.get_task_mut(task_id) {
            task.complete(current_time);
        }

        // Attribution tracking: complete task attribution
        if world.attribution_collector.is_enabled() {
            world
                .attribution_collector
                .complete_task(task_id, current_time);
        }

        // v2: Release all reservations for this robot
        world.reservation_manager.release_all(robot_id);

        // Track order completion
        if let Some(order_id) = order_id {
            // Get order info before mutation
            let order_info = world.get_order(order_id).map(|o| {
                (
                    o.arrival_time,
                    o.due_time,
                    o.tasks_completed + 1 >= o.tasks_total,
                )
            });

            // Mark task complete in order
            if let Some(order) = world.get_order_mut(order_id) {
                order.mark_task_complete();

                // Check if all tasks complete
                if order.all_tasks_complete() {
                    order.complete(current_time);
                }
            }

            // Record metrics if order just completed
            if let Some((arrival_time, due_time, was_last_task)) = order_info {
                if was_last_task {
                    let cycle_time = current_time - arrival_time;

                    // v1: Calculate lateness for SLA tracking
                    let lateness_s = if let Some(due) = due_time {
                        (current_time - due).as_seconds()
                    } else {
                        0.0 // No due time = on-time
                    };

                    // Use SLA-aware recording
                    metrics.record_order_with_sla(cycle_time, lateness_s);

                    // v3: Trace order completion
                    world.trace_collector.record_sampled(
                        current_time,
                        "OrderComplete",
                        TraceDetails::OrderComplete {
                            order_id: order_id.0,
                            cycle_time_s: cycle_time.as_seconds(),
                            is_late: lateness_s > 0.0,
                        },
                    );
                }
            }
        }

        // Record task completion
        metrics.record_task_complete(robot_id);

        // v3: Trace task completion and service
        world.trace_collector.record_sampled(
            current_time,
            "TaskComplete",
            TraceDetails::TaskComplete {
                task_id: task_id.0,
                robot_id: robot_id.0,
            },
        );

        world.trace_collector.record_sampled(
            current_time,
            "StationService",
            TraceDetails::StationService {
                station_id: station_id.0,
                robot_id: robot_id.0,
                duration_s: service_time.as_seconds(),
            },
        );

        // Robot becomes idle
        if let Some(robot) = world.get_robot_mut(robot_id) {
            robot.state = waremax_entities::RobotState::Idle;
            robot.complete_task();
            robot.update_stats(current_time);
        }

        // Try to start next robot in queue
        let next_robot = world
            .get_station_mut(station_id)
            .and_then(|s| s.start_service());
        if let Some(next_robot_id) = next_robot {
            if let Some(next_task_id) = world.get_robot(next_robot_id).and_then(|r| r.current_task)
            {
                kernel.schedule_now(SimEvent::StationServiceStart {
                    robot_id: next_robot_id,
                    station_id,
                    task_id: next_task_id,
                });
            }
        }

        // Schedule task dispatch
        kernel.schedule_now(SimEvent::DispatchTasks);
    }

    fn handle_dispatch_tasks(
        &mut self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
    ) {
        // Clear assigned tracker for this dispatch round
        self.assigned_this_round.clear();

        // Get pending tasks
        let mut pending: Vec<TaskId> = world.pending_tasks.clone();
        if pending.is_empty() {
            return;
        }

        // Apply priority policy to sort pending tasks
        {
            let ctx = world.policy_context(current_time);
            world.policies.priority.prioritize(&ctx, &mut pending);
        }

        // Apply batching policy to group tasks
        let batches = {
            let ctx = world.policy_context(current_time);
            world.policies.batching.batch(&ctx, &pending)
        };

        let mut tasks_to_remove = Vec::new();

        // Process each batch (with NoBatching, each is a single task)
        for batch in batches {
            for task_id in batch {
                // Skip if task no longer pending
                if !world
                    .get_task(task_id)
                    .map(|t| t.is_pending())
                    .unwrap_or(false)
                {
                    continue;
                }

                // Use task allocation policy to select robot
                let robot_id = {
                    let ctx = world.policy_context(current_time);
                    world.policies.task_allocation.allocate(&ctx, task_id)
                };

                if let Some(robot_id) = robot_id {
                    // Check robot hasn't been assigned this round
                    if self.assigned_this_round.contains(&robot_id) {
                        continue;
                    }

                    // Verify robot is still available
                    if !world
                        .get_robot(robot_id)
                        .map(|r| r.is_available())
                        .unwrap_or(false)
                    {
                        continue;
                    }

                    // Mark robot as assigned this round
                    self.assigned_this_round.push(robot_id);
                    tasks_to_remove.push(task_id);

                    // Schedule task assignment
                    kernel.schedule_now(SimEvent::TaskAssignment { task_id, robot_id });
                }
            }
        }

        // Remove assigned tasks from pending list
        for task_id in tasks_to_remove {
            world.pending_tasks.retain(|&t| t != task_id);
        }
    }

    // ==========================================================================
    // v1: Metrics sampling handler
    // ==========================================================================

    fn handle_metrics_sample_tick(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
    ) {
        // Sample station queues and utilization
        for (station_id, station) in &world.stations {
            world.time_series.record_station_queue(
                *station_id,
                current_time,
                station.queue_length(),
            );

            // Calculate current utilization
            let utilization = station.serving_count() as f64 / station.concurrency.max(1) as f64;
            world
                .time_series
                .record_station_utilization(*station_id, current_time, utilization);
        }

        // Sample charging station states
        for (station_id, station) in &world.charging_stations {
            world
                .time_series
                .record_charging_queue(*station_id, current_time, station.queue.len());
            world.time_series.record_charging_bays(
                *station_id,
                current_time,
                station.charging.len(),
            );

            let utilization = station.charging.len() as f64 / station.bays.max(1) as f64;
            world
                .time_series
                .record_charging_utilization(*station_id, current_time, utilization);
        }

        // Record node/edge occupancy
        for node_id in world.map.nodes.keys() {
            let occupancy = world.traffic.get_node_occupancy(*node_id);
            if occupancy > 0 {
                world.time_series.record_node_occupancy(*node_id, occupancy);
                world.time_series.record_node_traversal(*node_id);
            }
        }

        // v2: Clean up expired reservations
        world.reservation_manager.cleanup_expired(current_time);

        // Schedule next sample tick
        let next_interval = SimTime::from_seconds(world.metrics_sample_interval_s);
        kernel.schedule_after(next_interval, SimEvent::MetricsSampleTick);
    }

    // ==========================================================================
    // v1: Battery and charging handlers
    // ==========================================================================

    fn handle_robot_low_battery(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        _current_time: SimTime,
        robot_id: RobotId,
    ) {
        // Mark robot as seeking charging
        if let Some(robot) = world.get_robot_mut(robot_id) {
            if robot.seeking_charging {
                return; // Already handling low battery
            }
            robot.seeking_charging = true;
        }

        // Find nearest available charging station
        let robot_node = world.get_robot(robot_id).map(|r| r.current_node);
        if let Some(from_node) = robot_node {
            if let Some(charging_station_id) = world.find_nearest_charging_station(from_node) {
                // Route robot to charging station
                let station_node = world
                    .get_charging_station(charging_station_id)
                    .map(|s| s.node);

                if let Some(dest_node) = station_node {
                    if let Some(route) = world.router.find_route(&world.map, from_node, dest_node) {
                        if let Some(robot) = world.get_robot_mut(robot_id) {
                            robot.set_path(route.path);
                            robot.assigned_charging_station = Some(charging_station_id);
                            robot.state = RobotState::SeekingCharge {
                                destination: charging_station_id,
                            };
                        }

                        // Start movement to charging station
                        if let Some(robot) = world.get_robot(robot_id) {
                            if let Some(next_node) = robot.next_node_in_path() {
                                if let Some((_, edge_id, _)) = world
                                    .map
                                    .neighbors(robot.current_node)
                                    .find(|(n, _, _)| *n == next_node)
                                {
                                    kernel.schedule_now(SimEvent::RobotDepartNode {
                                        robot_id,
                                        from_node: robot.current_node,
                                        to_node: next_node,
                                        edge_id,
                                    });
                                }
                            } else {
                                // Already at charging station
                                kernel.schedule_now(SimEvent::RobotChargingStart {
                                    robot_id,
                                    station_id: charging_station_id,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_robot_charging_start(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        robot_id: RobotId,
        station_id: ChargingStationId,
        metrics: &mut MetricsCollector,
    ) {
        // Check if station has a free bay
        let has_free_bay = world
            .get_charging_station(station_id)
            .map(|s| s.has_free_bay())
            .unwrap_or(false);

        if has_free_bay {
            // Start charging
            if let Some(station) = world.get_charging_station_mut(station_id) {
                station.start_charging(robot_id, current_time);
            }

            // Update robot state
            if let Some(robot) = world.get_robot_mut(robot_id) {
                robot.state = RobotState::Charging {
                    at_station: station_id,
                };
                robot.update_stats(current_time);
            }

            // Calculate charge duration based on SOC deficit and charge rate
            let (charge_duration, energy_charged_wh, soc_for_trace) = {
                let robot = world.get_robot(robot_id);
                let station = world.get_charging_station(station_id);

                if let (Some(robot), Some(station)) = (robot, station) {
                    let battery = &robot.battery;
                    if battery.enabled {
                        // Record SOC at charge start for metrics
                        metrics.record_charging_start_soc(battery.soc);

                        let soc = battery.soc;
                        // Calculate energy needed to reach full charge
                        let soc_deficit = 1.0 - battery.soc;
                        let energy_needed_wh = soc_deficit * battery.capacity_wh;
                        // Use station's duration calculation
                        let duration =
                            station.charging_duration(battery.soc, 1.0, battery.capacity_wh);
                        (duration, energy_needed_wh, Some(soc))
                    } else {
                        (SimTime::from_seconds(300.0), 0.0, None) // Default 5 min if no battery
                    }
                } else {
                    (SimTime::from_seconds(300.0), 0.0, None)
                }
            };

            // v3: Trace charging start (outside borrow scope)
            if let Some(soc) = soc_for_trace {
                world.trace_collector.record_sampled(
                    current_time,
                    "ChargingStart",
                    TraceDetails::ChargingStart {
                        robot_id: robot_id.0,
                        station_id: station_id.0,
                        soc,
                    },
                );
            }

            // Schedule charging end
            kernel.schedule_after(
                charge_duration,
                SimEvent::RobotChargingEnd {
                    robot_id,
                    station_id,
                    energy_charged_wh,
                },
            );
        } else {
            // Queue the robot
            if let Some(station) = world.get_charging_station_mut(station_id) {
                station.enqueue(robot_id);
            }
        }
    }

    fn handle_robot_charging_end(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        robot_id: RobotId,
        station_id: ChargingStationId,
        energy_charged_wh: f64,
    ) {
        // Calculate the charging duration from the start time
        let charging_duration = {
            world
                .get_charging_station(station_id)
                .and_then(|s| {
                    s.charging
                        .iter()
                        .find(|(r, _)| *r == robot_id)
                        .map(|(_, start_time)| current_time - *start_time)
                })
                .unwrap_or(SimTime::ZERO)
        };

        // Complete charging at station
        if let Some(station) = world.get_charging_station_mut(station_id) {
            station.end_charging(robot_id, energy_charged_wh, charging_duration);
        }

        // Update robot battery and state
        if let Some(robot) = world.get_robot_mut(robot_id) {
            robot.charge(energy_charged_wh, charging_duration);
            robot.state = RobotState::Idle;
            robot.seeking_charging = false;
            robot.assigned_charging_station = None;
            // Note: charging_events is incremented in robot.charge()
            robot.update_stats(current_time);
        }

        // v3: Trace charging end
        world.trace_collector.record_sampled(
            current_time,
            "ChargingEnd",
            TraceDetails::ChargingEnd {
                robot_id: robot_id.0,
                energy_wh: energy_charged_wh,
            },
        );

        // Start next robot in queue
        let next_robot = world
            .get_charging_station_mut(station_id)
            .and_then(|s| s.next_in_queue());

        if let Some(next_robot_id) = next_robot {
            kernel.schedule_now(SimEvent::RobotChargingStart {
                robot_id: next_robot_id,
                station_id,
            });
        }

        // Schedule task dispatch to assign work to now-available robot
        kernel.schedule_now(SimEvent::DispatchTasks);
    }

    // === v2: Deadlock Detection Handlers ===

    fn handle_deadlock_detected(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        _current_time: SimTime,
        cycle: Vec<RobotId>,
    ) {
        use waremax_policies::{DeadlockContext, DeadlockResolution};

        // Build context for the resolver
        let mut ctx = DeadlockContext::new(cycle.clone());

        // Gather position and priority info for each robot in the cycle
        for &robot_id in &cycle {
            if let Some(robot) = world.get_robot(robot_id) {
                ctx = ctx.with_position(robot_id, robot.current_node);
                // No previous_node tracked in Robot, use None
                ctx = ctx.with_previous(robot_id, None);

                // Use task priority based on task type
                if let Some(task_id) = robot.current_task {
                    if let Some(task) = world.get_task(task_id) {
                        // Derive priority from task type (Pick=1, Putaway=2, Replenishment=3)
                        let priority = match task.task_type {
                            waremax_entities::TaskType::Pick => 1,
                            waremax_entities::TaskType::Putaway => 2,
                            waremax_entities::TaskType::Replenishment => 3,
                        };
                        ctx = ctx.with_priority(robot_id, priority);
                    }
                } else {
                    ctx = ctx.with_priority(robot_id, u32::MAX); // No task = lowest priority
                }
            }
        }

        // Get resolution from configured policy
        let resolution = world.deadlock_resolver.resolve(&ctx);

        // Apply the resolution
        match resolution {
            DeadlockResolution::BackUp { robot, to_node } => {
                // Clear wait status
                world.traffic.clear_wait(robot);

                // Get current node before mutating
                let current = world.get_robot(robot).map(|r| r.current_node);

                // Update traffic - leave current node, enter new node
                if let Some(current_node) = current {
                    world.traffic.leave_node(current_node, robot);
                }
                world.traffic.enter_node(to_node, robot);

                // Update robot position
                if let Some(r) = world.get_robot_mut(robot) {
                    r.current_node = to_node;
                    r.state = RobotState::Idle;
                }

                // Schedule event to log resolution
                kernel.schedule_now(SimEvent::DeadlockResolved {
                    robots: cycle,
                    resolver_robot: robot,
                });

                // Retry movement for all involved robots
                kernel.schedule_now(SimEvent::DispatchTasks);
            }
            DeadlockResolution::AbortTask { robot } => {
                // Clear wait status
                world.traffic.clear_wait(robot);

                // Get current task before mutating
                let task_to_requeue = world.get_robot(robot).and_then(|r| r.current_task);

                // Abort current task and requeue
                if let Some(r) = world.get_robot_mut(robot) {
                    r.current_task = None;
                    r.state = RobotState::Idle;
                }

                // Requeue the task (done separately to avoid borrow conflict)
                if let Some(task_id) = task_to_requeue {
                    world.pending_tasks.push(task_id);
                }

                // Schedule event to log resolution
                kernel.schedule_now(SimEvent::DeadlockResolved {
                    robots: cycle,
                    resolver_robot: robot,
                });

                // Dispatch tasks to reassign
                kernel.schedule_now(SimEvent::DispatchTasks);
            }
            DeadlockResolution::WaitAndRetry { duration } => {
                // Just wait - the scheduled retry events will handle it
                // Schedule a retry check after the wait duration
                kernel.schedule_after(duration, SimEvent::DispatchTasks);
            }
        }
    }

    fn handle_deadlock_resolved(
        &self,
        _kernel: &mut Kernel,
        _world: &mut World,
        _resolver_robot: RobotId,
        _robots: Vec<RobotId>,
    ) {
        // This handler is primarily for event logging/metrics
        // The actual resolution is done in handle_deadlock_detected
        // Future: could add metrics tracking for deadlock frequency
    }

    // === v3: Robot Failures & Maintenance Handlers ===

    fn handle_robot_failure(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        robot_id: RobotId,
        interrupted_task: Option<TaskId>,
        metrics: &mut MetricsCollector,
    ) {
        // Mark robot as failed
        if let Some(robot) = world.get_robot_mut(robot_id) {
            robot.update_stats(current_time);
            robot.mark_failed(current_time);
            robot.state = RobotState::Failed;

            // Clear current task
            if robot.current_task.is_some() {
                robot.current_task = None;
            }
        }

        // Requeue the interrupted task
        if let Some(task_id) = interrupted_task {
            if let Some(task) = world.get_task_mut(task_id) {
                task.status = waremax_entities::TaskStatus::Pending;
            }
            world.pending_tasks.push(task_id);
        }

        // Record failure in metrics
        metrics.record_robot_failure();

        // v3: Trace robot failure
        world.trace_collector.record_sampled(
            current_time,
            "RobotFailure",
            TraceDetails::RobotFailure {
                robot_id: robot_id.0,
            },
        );

        // Find nearest maintenance station for repair
        let robot_node = world.get_robot(robot_id).map(|r| r.current_node);
        if let Some(from_node) = robot_node {
            if let Some(station_id) = world.find_nearest_maintenance_station(from_node) {
                // Route robot to maintenance station for repair
                let station_node = world.get_maintenance_station(station_id).map(|s| s.node);
                if let Some(dest_node) = station_node {
                    if let Some(route) = world.router.find_route(&world.map, from_node, dest_node) {
                        if let Some(robot) = world.get_robot_mut(robot_id) {
                            robot.set_path(route.path.clone());
                            robot.state = RobotState::SeekingMaintenance {
                                destination: station_id,
                                is_repair: true,
                            };
                            robot.seeking_maintenance = true;
                            robot.assigned_maintenance_station = Some(station_id);
                        }

                        // Schedule first depart event if robot can move
                        if route.path.len() > 1 {
                            // Find edge between nodes
                            if let Some((_, edge_id, _)) = world
                                .map
                                .neighbors(route.path[0])
                                .find(|(n, _, _)| *n == route.path[1])
                            {
                                kernel.schedule_now(SimEvent::RobotDepartNode {
                                    robot_id,
                                    from_node,
                                    to_node: route.path[1],
                                    edge_id,
                                });
                            }
                        } else {
                            // Already at station, start maintenance immediately
                            kernel.schedule_now(SimEvent::MaintenanceStart {
                                robot_id,
                                station_id,
                                is_repair: true,
                            });
                        }
                    }
                }
            }
        }
    }

    fn handle_robot_maintenance_due(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        robot_id: RobotId,
    ) {
        // Check if robot is available and not already seeking maintenance
        let (is_available, from_node, seeking) = {
            world
                .get_robot(robot_id)
                .map(|r| {
                    (
                        r.is_idle() && r.current_task.is_none(),
                        r.current_node,
                        r.seeking_maintenance,
                    )
                })
                .unwrap_or((false, waremax_core::NodeId(0), true))
        };

        if !is_available || seeking {
            // Robot is busy - reschedule check for later
            kernel.schedule_after(
                SimTime::from_seconds(60.0), // Check again in 1 minute
                SimEvent::RobotMaintenanceDue {
                    robot_id,
                    operating_hours: 0.0, // Will be recalculated
                },
            );
            return;
        }

        // Find nearest maintenance station
        if let Some(station_id) = world.find_nearest_maintenance_station(from_node) {
            let station_node = world.get_maintenance_station(station_id).map(|s| s.node);
            if let Some(dest_node) = station_node {
                if let Some(route) = world.router.find_route(&world.map, from_node, dest_node) {
                    if let Some(robot) = world.get_robot_mut(robot_id) {
                        robot.update_stats(current_time);
                        robot.set_path(route.path.clone());
                        robot.state = RobotState::SeekingMaintenance {
                            destination: station_id,
                            is_repair: false,
                        };
                        robot.seeking_maintenance = true;
                        robot.assigned_maintenance_station = Some(station_id);
                    }

                    // Schedule first depart event
                    if route.path.len() > 1 {
                        // Find edge between nodes
                        if let Some((_, edge_id, _)) = world
                            .map
                            .neighbors(route.path[0])
                            .find(|(n, _, _)| *n == route.path[1])
                        {
                            kernel.schedule_now(SimEvent::RobotDepartNode {
                                robot_id,
                                from_node,
                                to_node: route.path[1],
                                edge_id,
                            });
                        }
                    } else {
                        // Already at station
                        kernel.schedule_now(SimEvent::MaintenanceStart {
                            robot_id,
                            station_id,
                            is_repair: false,
                        });
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_maintenance_start(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        robot_id: RobotId,
        station_id: MaintenanceStationId,
        is_repair: bool,
        metrics: &mut MetricsCollector,
    ) {
        // Check if station has capacity and get repair time model info
        let (has_free_bay, maintenance_duration_s, repair_model) = {
            if let Some(station) = world.get_maintenance_station(station_id) {
                (
                    station.has_free_bay(),
                    station.maintenance_duration_s,
                    station.repair_time_model.clone(),
                )
            } else {
                return;
            }
        };

        // Calculate duration (needs rng for repair, fixed for maintenance)
        let service_duration = if has_free_bay {
            let duration = if is_repair {
                repair_model.sample(&mut world.rng, 1)
            } else {
                SimTime::from_seconds(maintenance_duration_s)
            };

            // Now start the service at the station
            if let Some(station) = world.get_maintenance_station_mut(station_id) {
                station.queue.retain(|&r| r != robot_id);
                station.servicing.push((robot_id, current_time, is_repair));
            }

            Some(duration)
        } else {
            None
        };

        if let Some(duration) = service_duration {
            // Update robot state
            if let Some(robot) = world.get_robot_mut(robot_id) {
                robot.update_stats(current_time);
                robot.state = RobotState::InMaintenance {
                    at_station: station_id,
                    is_repair,
                };
            }

            // Record metrics
            if is_repair {
                metrics.record_repair_start();
            } else {
                metrics.record_maintenance_start();
            }

            // Schedule maintenance end
            kernel.schedule_after(
                duration,
                SimEvent::MaintenanceEnd {
                    robot_id,
                    station_id,
                    is_repair,
                    duration_s: duration.as_seconds(),
                },
            );
        } else {
            // Station is full, add to queue
            if let Some(station) = world.get_maintenance_station_mut(station_id) {
                station.enqueue(robot_id);
            }
            // Robot waits in queue - state stays as SeekingMaintenance
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_maintenance_end(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        robot_id: RobotId,
        station_id: MaintenanceStationId,
        is_repair: bool,
        duration_s: f64,
        metrics: &mut MetricsCollector,
    ) {
        // Complete service at station
        if let Some(station) = world.get_maintenance_station_mut(station_id) {
            station.end_service(robot_id, SimTime::from_seconds(duration_s), is_repair);
        }

        // Update robot state
        if let Some(robot) = world.get_robot_mut(robot_id) {
            robot.update_stats(current_time);
            robot.complete_maintenance(current_time);
            robot.state = RobotState::Idle;
        }

        // Record metrics
        if is_repair {
            metrics.record_repair_end(duration_s);
        } else {
            metrics.record_maintenance_end(duration_s);
        }

        // Start next robot in queue
        let next_robot_info = {
            if let Some(station) = world.get_maintenance_station_mut(station_id) {
                station.next_in_queue()
            } else {
                None
            }
        };

        if let Some(next_robot_id) = next_robot_info {
            // Determine if this robot needs repair or scheduled maintenance
            let next_is_repair = world
                .get_robot(next_robot_id)
                .map(|r| r.is_failed())
                .unwrap_or(false);

            kernel.schedule_now(SimEvent::MaintenanceStart {
                robot_id: next_robot_id,
                station_id,
                is_repair: next_is_repair,
            });
        }

        // Schedule task dispatch to assign work to now-available robot
        kernel.schedule_now(SimEvent::DispatchTasks);
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
