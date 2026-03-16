//! Event handlers for simulation events

use waremax_core::{SimTime, SimEvent, ScheduledEvent, Kernel, OrderId, TaskId};
use waremax_entities::{Order, OrderLine, Task, BinLocation};
use waremax_metrics::MetricsCollector;
use crate::world::World;

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
    pub fn handle(&mut self, kernel: &mut Kernel, world: &mut World, event: &ScheduledEvent, metrics: &mut MetricsCollector) {
        let current_time = event.time;

        match &event.event {
            SimEvent::OrderArrival { order_id } => {
                self.handle_order_arrival(kernel, world, current_time, *order_id);
            }
            SimEvent::TaskAssignment { task_id, robot_id } => {
                self.handle_task_assignment(kernel, world, current_time, *task_id, *robot_id);
            }
            SimEvent::RobotDepartNode { robot_id, from_node, to_node, edge_id } => {
                self.handle_robot_depart(kernel, world, current_time, *robot_id, *from_node, *to_node, *edge_id);
            }
            SimEvent::RobotArriveNode { robot_id, node_id, from_node } => {
                self.handle_robot_arrive(kernel, world, current_time, *robot_id, *node_id, *from_node);
            }
            SimEvent::StationServiceStart { robot_id, station_id, task_id } => {
                self.handle_service_start(kernel, world, current_time, *robot_id, *station_id, *task_id);
            }
            SimEvent::StationServiceEnd { robot_id, station_id, task_id } => {
                self.handle_service_end(kernel, world, current_time, *robot_id, *station_id, *task_id, metrics);
            }
            SimEvent::DispatchTasks => {
                self.handle_dispatch_tasks(kernel, world, current_time);
            }
            _ => {
                // Handle other events as needed
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
        let interarrival = world.distributions.arrivals.next_interarrival(&mut world.rng);
        let next_order_id = world.next_order_id();
        kernel.schedule_after(
            SimTime::from_seconds(interarrival),
            SimEvent::OrderArrival { order_id: next_order_id },
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
            let sku_idx = world.distributions.skus.next_sku(&mut world.rng, num_skus as u32);
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
                if let Some(assigned_station) = world.policies.station_assignment.assign(&ctx, &temp_task) {
                    tasks_to_create.push((task_id, sku_id, quantity, bin_location, assigned_station));
                }
            }
        }

        // Calculate due time if configured
        let due_time = world.due_time_offset_min.map(|offset| {
            current_time + SimTime::from_minutes(offset)
        });

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

        // Start moving robot to pickup location
        if let Some(task) = world.get_task(task_id) {
            if let Some(robot) = world.get_robot(robot_id) {
                let pickup_node = task.source.access_node;
                if let Some(route) = world.router.find_route(&world.map, robot.current_node, pickup_node) {
                    if let Some(robot) = world.get_robot_mut(robot_id) {
                        robot.set_path(route.path);
                    }
                    // Start movement
                    if let Some(robot) = world.get_robot(robot_id) {
                        if let Some(next_node) = robot.next_node_in_path() {
                            // Find edge to next node
                            if let Some((_, edge_id, _)) = world.map.neighbors(robot.current_node)
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
    }

    fn handle_robot_depart(
        &self,
        kernel: &mut Kernel,
        world: &mut World,
        current_time: SimTime,
        robot_id: waremax_core::RobotId,
        from_node: waremax_core::NodeId,
        to_node: waremax_core::NodeId,
        edge_id: waremax_core::EdgeId,
    ) {
        // Check if edge is available
        if !world.traffic.can_enter_edge(edge_id, robot_id) {
            // Schedule wait and retry
            kernel.schedule_after(
                SimTime::from_seconds(0.5),
                SimEvent::RobotDepartNode { robot_id, from_node, to_node, edge_id },
            );
            return;
        }

        // Update traffic
        world.traffic.leave_node(from_node, robot_id);
        world.traffic.enter_edge(edge_id, robot_id);

        // Update robot state
        if let Some(robot) = world.get_robot_mut(robot_id) {
            robot.state = waremax_entities::RobotState::Moving { destination: to_node };
            robot.update_stats(current_time);
        }

        // Calculate travel time and schedule arrival
        if let (Some(robot), Some(edge)) = (world.get_robot(robot_id), world.map.get_edge(edge_id)) {
            let travel_time = robot.travel_time(edge.length_m);
            kernel.schedule_after(travel_time, SimEvent::RobotArriveNode {
                robot_id,
                node_id: to_node,
                from_node,
            });
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
        if let Some((_, edge_id, _)) = world.map.neighbors(from_node).find(|(n, _, _)| *n == node_id) {
            world.traffic.leave_edge(edge_id, robot_id);
        }

        // Update traffic and robot state
        world.traffic.enter_node(node_id, robot_id);

        if let Some(robot) = world.get_robot_mut(robot_id) {
            robot.current_node = node_id;
            robot.advance_path();
            robot.update_stats(current_time);
        }

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
                let task_info = world.get_task(task_id).map(|task| {
                    (task.source.access_node, task.destination_station)
                });

                if let Some((source_node, destination_station)) = task_info {
                    if node_id == source_node {
                        // At pickup - go to station
                        let station_node = world.get_station(destination_station).map(|s| s.node);
                        let robot_current = world.get_robot(robot_id).map(|r| r.current_node);

                        if let (Some(station_node), Some(robot_current)) = (station_node, robot_current) {
                            if let Some(route) = world.router.find_route(&world.map, robot_current, station_node) {
                                if let Some(robot) = world.get_robot_mut(robot_id) {
                                    robot.set_path(route.path);
                                }
                            }
                        }

                        // Continue to next node or arrive at station
                        if let Some(robot) = world.get_robot(robot_id) {
                            if let Some(next) = robot.next_node_in_path() {
                                if let Some((_, edge_id, _)) = world.map.neighbors(robot.current_node)
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
                                // At station
                                kernel.schedule_now(SimEvent::StationServiceStart {
                                    robot_id,
                                    station_id: destination_station,
                                    task_id,
                                });
                            }
                        }
                    } else {
                        // At station
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
                if let Some((_, edge_id, _)) = world.map.neighbors(robot.current_node)
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
        let already_serving = world.get_station(station_id)
            .map(|s| s.is_robot_being_served(robot_id))
            .unwrap_or(false);
        let can_serve = world.get_station(station_id).map(|s| s.can_serve()).unwrap_or(false);

        if already_serving || can_serve {
            // Start service (only add to serving if not already there)
            if !already_serving {
                if let Some(station) = world.get_station_mut(station_id) {
                    station.serving.push(robot_id);
                }
            }

            if let Some(robot) = world.get_robot_mut(robot_id) {
                robot.state = waremax_entities::RobotState::Servicing { at_station: station_id };
                robot.update_stats(current_time);
            }

            // Get task quantity for service time calculation
            let item_count = world.get_task(task_id).map(|t| t.quantity).unwrap_or(1);

            // Calculate service time based on item count
            let service_time = world.get_station(station_id)
                .map(|s| s.service_time.calculate(item_count))
                .unwrap_or(SimTime::from_seconds(10.0));

            kernel.schedule_after(service_time, SimEvent::StationServiceEnd {
                robot_id,
                station_id,
                task_id,
            });
        } else {
            // Queue the robot
            if let Some(station) = world.get_station_mut(station_id) {
                station.enqueue(robot_id);
            }
        }
    }

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
        // Get service time from task quantity
        let item_count = world.get_task(task_id).map(|t| t.quantity).unwrap_or(1);
        let service_time = world.get_station(station_id)
            .map(|s| s.service_time.calculate(item_count))
            .unwrap_or(SimTime::from_seconds(10.0));

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

        // Track order completion
        if let Some(order_id) = order_id {
            // Get order info before mutation
            let order_info = world.get_order(order_id).map(|o| {
                (o.arrival_time, o.due_time, o.tasks_completed + 1 >= o.tasks_total)
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
                    let is_late = due_time.map(|d| current_time > d).unwrap_or(false);
                    metrics.record_order_complete(cycle_time, is_late);
                }
            }
        }

        // Record task completion
        metrics.record_task_complete(robot_id);

        // Robot becomes idle
        if let Some(robot) = world.get_robot_mut(robot_id) {
            robot.state = waremax_entities::RobotState::Idle;
            robot.complete_task();
            robot.update_stats(current_time);
        }

        // Try to start next robot in queue
        let next_robot = world.get_station_mut(station_id).and_then(|s| s.start_service());
        if let Some(next_robot_id) = next_robot {
            if let Some(next_task_id) = world.get_robot(next_robot_id).and_then(|r| r.current_task) {
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
                if !world.get_task(task_id).map(|t| t.is_pending()).unwrap_or(false) {
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
                    if !world.get_robot(robot_id).map(|r| r.is_available()).unwrap_or(false) {
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
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
