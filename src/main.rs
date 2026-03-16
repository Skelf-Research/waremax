//! Waremax CLI - Warehouse Robot Simulation
//!
//! Run discrete-event simulations for warehouse robot operations.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Parser)]
#[command(name = "waremax")]
#[command(author = "Waremax Team")]
#[command(version = "0.1.0")]
#[command(about = "Warehouse Robot Discrete-Event Simulation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a simulation from a scenario file
    Run {
        /// Path to the scenario YAML file
        #[arg(short, long)]
        scenario: PathBuf,

        /// Random seed (optional, defaults to scenario seed)
        #[arg(long)]
        seed: Option<u64>,

        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        output: String,
    },
    /// Validate a scenario file without running
    Validate {
        /// Path to the scenario YAML file
        #[arg(short, long)]
        scenario: PathBuf,
    },
    /// Run a quick demo simulation
    Demo {
        /// Duration in minutes
        #[arg(short, long, default_value = "60")]
        duration: f64,

        /// Number of robots
        #[arg(short, long, default_value = "5")]
        robots: usize,

        /// Order arrival rate per minute
        #[arg(long, default_value = "4.0")]
        order_rate: f64,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            scenario,
            seed,
            output,
        } => {
            run_simulation(&scenario, seed, &output);
        }
        Commands::Validate { scenario } => {
            validate_scenario(&scenario);
        }
        Commands::Demo {
            duration,
            robots,
            order_rate,
        } => {
            run_demo(duration, robots, order_rate);
        }
    }
}

fn run_simulation(
    scenario_path: &PathBuf,
    seed_override: Option<u64>,
    output_format: &str,
) {
    println!("Loading scenario from: {}", scenario_path.display());

    let path_str = scenario_path.to_string_lossy();

    // Load scenario
    let scenario = match waremax_config::ScenarioConfig::from_file(&path_str) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading scenario: {}", e);
            std::process::exit(1);
        }
    };

    // Use seed override if provided
    let seed = seed_override.unwrap_or(scenario.seed);

    println!("Running simulation with seed: {}", seed);
    println!(
        "Duration: {} minutes (warmup: {} minutes)",
        scenario.simulation.duration_minutes, scenario.simulation.warmup_minutes
    );

    // Build world from scenario
    let world = build_world_from_scenario(&scenario, seed, scenario_path);

    // Create and run simulation
    let mut runner = waremax_sim::SimulationRunner::new(
        world,
        scenario.simulation.duration_minutes,
        scenario.simulation.warmup_minutes,
    );

    let report = runner.run();

    // Output results
    match output_format {
        "json" => println!("{}", report.to_json()),
        _ => println!("{}", report.summary()),
    }
}

fn validate_scenario(scenario_path: &PathBuf) {
    println!("Validating scenario: {}", scenario_path.display());

    let path_str = scenario_path.to_string_lossy();

    // Load scenario
    let scenario = match waremax_config::ScenarioConfig::from_file(&path_str) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load scenario: {}", e);
            std::process::exit(1);
        }
    };

    // Try to load map and storage (optional - may not exist)
    let base_dir = scenario_path.parent().unwrap_or(std::path::Path::new("."));

    let map_path = base_dir.join(&scenario.map.file);
    let map = waremax_config::MapConfig::from_file(&map_path.to_string_lossy()).ok();
    if map.is_none() {
        println!("  Note: Map file '{}' not found, skipping map validation", scenario.map.file);
    }

    let storage_path = base_dir.join(&scenario.storage.file);
    let storage = waremax_config::StorageConfig::from_file(&storage_path.to_string_lossy()).ok();
    if storage.is_none() {
        println!(
            "  Note: Storage file '{}' not found, skipping storage validation",
            scenario.storage.file
        );
    }

    // Run validation
    match waremax_config::validate_scenario(&scenario, map.as_ref(), storage.as_ref()) {
        Ok(warnings) => {
            println!("Scenario valid!");
            println!("  Seed: {}", scenario.seed);
            println!("  Duration: {} minutes", scenario.simulation.duration_minutes);
            println!("  Warmup: {} minutes", scenario.simulation.warmup_minutes);
            println!("  Robot count: {}", scenario.robots.count);
            println!("  Stations: {}", scenario.stations.len());

            if !warnings.is_empty() {
                println!("\nWarnings ({}):", warnings.len());
                for w in &warnings {
                    println!("  - {}", w);
                }
            }
        }
        Err(errors) => {
            eprintln!("Validation failed with {} error(s):", errors.len());
            for e in &errors {
                eprintln!("  - {}", e);
            }
            std::process::exit(1);
        }
    }
}

fn run_demo(duration_minutes: f64, num_robots: usize, order_rate: f64) {
    println!("Running demo simulation...");
    println!("  Duration: {} minutes", duration_minutes);
    println!("  Robots: {}", num_robots);
    println!("  Order rate: {}/min", order_rate);
    println!();

    let seed = 42u64;
    let world = build_demo_world(seed, num_robots, order_rate);

    let mut runner = waremax_sim::SimulationRunner::new(world, duration_minutes, 5.0);

    let report = runner.run();
    println!("{}", report.summary());
}

fn build_world_from_scenario(
    scenario: &waremax_config::ScenarioConfig,
    seed: u64,
    _scenario_path: &PathBuf,
) -> waremax_sim::World {
    use waremax_core::{NodeId, EdgeId, RobotId, StationId};
    use waremax_entities::{Robot, Station, StationType, ServiceTimeModel};
    use waremax_map::{WarehouseMap, Node, Edge, NodeType, Router, TrafficManager};

    let mut world = waremax_sim::World::new(seed);

    // Build a simple map for now (in a real scenario, load from map.file)
    // Create a simple 5x5 grid as default
    let mut map = WarehouseMap::new();
    let grid_size = 5;
    let spacing = 3.0;

    for row in 0..grid_size {
        for col in 0..grid_size {
            let id = row * grid_size + col;
            let x = col as f64 * spacing;
            let y = row as f64 * spacing;
            let node_type = if id == 0 { NodeType::StationPick } else { NodeType::Aisle };
            let node = Node::new(NodeId(id as u32), format!("N{}", id), x, y, node_type);
            map.add_node(node);
        }
    }

    let mut edge_id = 0u32;
    for row in 0..grid_size {
        for col in 0..grid_size {
            let id = row * grid_size + col;
            if col < grid_size - 1 {
                let neighbor = id + 1;
                map.add_edge(
                    Edge::new(EdgeId(edge_id), NodeId(id as u32), NodeId(neighbor as u32), spacing),
                    true,
                );
                edge_id += 1;
            }
            if row < grid_size - 1 {
                let neighbor = id + grid_size;
                map.add_edge(
                    Edge::new(EdgeId(edge_id), NodeId(id as u32), NodeId(neighbor as u32), spacing),
                    true,
                );
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

    // Add robots
    for i in 0..scenario.robots.count {
        let start_node = (i % (grid_size * grid_size) as u32) as u32;
        let robot = Robot::new(
            RobotId(i),
            NodeId(start_node),
            scenario.robots.max_speed_mps,
            scenario.robots.max_payload_kg,
        );
        world.robots.insert(RobotId(i), robot);
    }

    // Add stations from config
    for (idx, station_cfg) in scenario.stations.iter().enumerate() {
        let station_type = match station_cfg.station_type.as_str() {
            "pick" => StationType::Pick,
            "drop" => StationType::Drop,
            "inbound" => StationType::Inbound,
            "outbound" => StationType::Outbound,
            _ => StationType::Pick,
        };

        let service_time = ServiceTimeModel::new(
            station_cfg.service_time_s.base,
            station_cfg.service_time_s.per_item,
        );

        // Map station node to NodeId (for now assume it's a numeric string)
        let node_id: u32 = station_cfg.node.parse().unwrap_or(0);

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
    let (arrivals, lines, skus) = world.distributions.names();
    println!("Distributions:");
    println!("  Arrivals: {}", arrivals);
    println!("  Lines/Order: {}", lines);
    println!("  SKU Selection: {}", skus);

    // Create policies from config
    world.policies = waremax_sim::create_policies(&scenario.policies);
    let (alloc, station, batch, prio) = world.policies.names();
    println!("Policies:");
    println!("  Task Allocation: {}", alloc);
    println!("  Station Assignment: {}", station);
    println!("  Batching: {}", batch);
    println!("  Priority: {}", prio);

    // Initialize demo inventory with 20 SKUs
    world.init_demo_inventory(20);

    world
}

fn build_demo_world(seed: u64, num_robots: usize, order_rate: f64) -> waremax_sim::World {
    use waremax_core::{NodeId, EdgeId, RobotId, StationId};
    use waremax_entities::{Robot, Station, StationType, ServiceTimeModel};
    use waremax_map::{WarehouseMap, Node, Edge, NodeType, Router, TrafficManager};

    let mut world = waremax_sim::World::new(seed);

    // Create a simple grid map: 5x5 nodes
    let mut map = WarehouseMap::new();

    // Add nodes in a 5x5 grid
    let grid_size = 5;
    let spacing = 3.0; // 3 meters between nodes

    for row in 0..grid_size {
        for col in 0..grid_size {
            let id = row * grid_size + col;
            let x = col as f64 * spacing;
            let y = row as f64 * spacing;

            let node_type = if id == 0 {
                NodeType::StationPick // Pick station at (0,0)
            } else if row == 0 || row == grid_size - 1 || col == 0 || col == grid_size - 1 {
                NodeType::Aisle
            } else {
                NodeType::Rack
            };

            map.add_node(Node::new(NodeId(id as u32), format!("N{}", id), x, y, node_type));
        }
    }

    // Add edges connecting adjacent nodes
    let mut edge_id = 0u32;
    for row in 0..grid_size {
        for col in 0..grid_size {
            let id = row * grid_size + col;

            // Connect to right neighbor
            if col < grid_size - 1 {
                let neighbor = id + 1;
                map.add_edge(
                    Edge::new(EdgeId(edge_id), NodeId(id as u32), NodeId(neighbor as u32), spacing),
                    true,
                );
                edge_id += 1;
            }

            // Connect to bottom neighbor
            if row < grid_size - 1 {
                let neighbor = id + grid_size;
                map.add_edge(
                    Edge::new(EdgeId(edge_id), NodeId(id as u32), NodeId(neighbor as u32), spacing),
                    true,
                );
                edge_id += 1;
            }
        }
    }

    world.map = map;
    world.router = Router::new(true);
    // Allow 2 robots per edge/node to reduce congestion in demo
    world.traffic = TrafficManager::new(2, 2);

    // Add robots
    for i in 0..num_robots {
        let start_node = (i % (grid_size * grid_size)) as u32;
        let robot = Robot::new(RobotId(i as u32), NodeId(start_node), 1.5, 25.0);
        world.robots.insert(RobotId(i as u32), robot);
    }

    // Add a pick station at node 0
    let service_time = ServiceTimeModel::new(5.0, 2.0);
    let station = Station::new(
        StationId(0),
        "S0".to_string(),
        NodeId(0),
        StationType::Pick,
        2,
        None,
        service_time,
    );
    world.stations.insert(StationId(0), station);

    // Set order generation parameters
    world.due_time_offset_min = Some(60.0);

    // Create demo distributions with specified order rate
    use waremax_sim::distributions::{ExponentialArrivals, NegBinomialLines, ZipfSkus, DistributionSet};
    world.distributions = DistributionSet::new(
        Box::new(ExponentialArrivals::new(order_rate / 60.0)), // Convert per-min to per-sec
        Box::new(NegBinomialLines::new(2.0, 1.0)),
        Box::new(ZipfSkus::new(1.0)),
    );
    let (arrivals, lines, skus) = world.distributions.names();
    println!("Distributions:");
    println!("  Arrivals: {}", arrivals);
    println!("  Lines/Order: {}", lines);
    println!("  SKU Selection: {}", skus);

    // Log default policies
    let (alloc, station, batch, prio) = world.policies.names();
    println!("Policies:");
    println!("  Task Allocation: {}", alloc);
    println!("  Station Assignment: {}", station);
    println!("  Batching: {}", batch);
    println!("  Priority: {}", prio);

    // Initialize demo inventory with 20 SKUs
    world.init_demo_inventory(20);

    world
}
