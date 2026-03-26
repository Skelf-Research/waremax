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

        // v3: Export options
        /// Output directory for CSV/JSON exports
        #[arg(long)]
        output_dir: Option<PathBuf>,

        /// Generate per-robot breakdown CSV
        #[arg(long)]
        per_robot: bool,

        /// Generate per-station breakdown CSV
        #[arg(long)]
        per_station: bool,

        /// Generate congestion heatmap CSVs
        #[arg(long)]
        heatmap: bool,

        /// Generate time-series CSV
        #[arg(long)]
        timeseries: bool,

        /// Enable event tracing
        #[arg(long)]
        trace: bool,

        /// Enable attribution tracking for delay analysis (RCA)
        #[arg(long)]
        attribution: bool,
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

    // v4: Testing & Benchmarking commands
    /// Generate a scenario file from a preset
    Generate {
        /// Preset name: minimal, quick, standard, baseline, high_load, peak_hours, stress_test, battery_test, maintenance_test
        #[arg(long)]
        preset: String,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Override number of robots
        #[arg(long)]
        robots: Option<u32>,

        /// Override order rate (orders/hour)
        #[arg(long)]
        order_rate: Option<f64>,

        /// Override grid size (e.g., "10x10")
        #[arg(long)]
        grid: Option<String>,

        /// Override seed
        #[arg(long)]
        seed: Option<u64>,
    },

    /// Run a parameter sweep
    Sweep {
        /// Base scenario file
        #[arg(long)]
        base: PathBuf,

        /// Sweep definition (e.g., "robots:5,10,15,20" or "order_rate:30,60,90")
        #[arg(long)]
        sweep: String,

        /// Number of replications per configuration
        #[arg(long, default_value = "3")]
        replications: u32,

        /// Output directory for results
        #[arg(long)]
        output_dir: PathBuf,
    },

    /// Compare two scenario configurations
    Compare {
        /// Baseline scenario file
        #[arg(long)]
        baseline: PathBuf,

        /// Variant scenario file
        #[arg(long)]
        variant: PathBuf,

        /// Number of replications
        #[arg(long, default_value = "5")]
        replications: u32,

        /// Output file for results (JSON)
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Run A/B test between configurations
    AbTest {
        /// Baseline scenario file
        #[arg(long)]
        baseline: PathBuf,

        /// Variant scenario file
        #[arg(long)]
        variant: PathBuf,

        /// Number of replications per variant
        #[arg(long, default_value = "10")]
        replications: u32,

        /// Significance level (alpha)
        #[arg(long, default_value = "0.05")]
        alpha: f64,

        /// Output file for results (JSON)
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Run benchmark suite
    Benchmark {
        /// Custom benchmark suite file (JSON), or uses default presets
        #[arg(long)]
        suite: Option<PathBuf>,

        /// Number of replications per benchmark
        #[arg(long, default_value = "3")]
        replications: u32,

        /// History file for regression tracking
        #[arg(long)]
        history: Option<PathBuf>,

        /// Regression threshold percentage (e.g., 5.0 for 5%)
        #[arg(long, default_value = "5.0")]
        regression_threshold: f64,

        /// Output file for results (JSON)
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Run stress test with configurable parameters
    StressTest {
        /// Number of robots (default: 50)
        #[arg(long, default_value = "50")]
        robots: u32,

        /// Order rate per hour (default: 300)
        #[arg(long, default_value = "300.0")]
        order_rate: f64,

        /// Duration in minutes (default: 60)
        #[arg(long, default_value = "60.0")]
        duration: f64,

        /// Grid size (e.g., "20x20")
        #[arg(long, default_value = "20x20")]
        grid: String,

        /// Output file for results (JSON)
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// List available presets
    ListPresets,

    // v5: Root Cause Analysis command
    /// Run root cause analysis on a simulation
    Analyze {
        /// Path to the scenario YAML file
        #[arg(short, long)]
        scenario: PathBuf,

        /// Output file for the RCA report
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format: text, json, compact
        #[arg(long, default_value = "text")]
        format: String,

        /// Include detailed bottleneck analysis
        #[arg(long)]
        detailed: bool,

        /// Anomaly detection threshold (z-score, default: 2.0)
        #[arg(long, default_value = "2.0")]
        anomaly_threshold: f64,
    },

    /// Launch interactive web UI for simulation visualization
    Ui {
        /// Port to serve on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Open browser automatically
        #[arg(long, default_value = "true")]
        open: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            scenario,
            seed,
            output,
            output_dir,
            per_robot,
            per_station,
            heatmap,
            timeseries,
            trace,
            attribution,
        } => {
            run_simulation(
                &scenario,
                seed,
                &output,
                output_dir.as_deref(),
                per_robot,
                per_station,
                heatmap,
                timeseries,
                trace,
                attribution,
            );
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

        // v4: Testing & Benchmarking commands
        Commands::Generate {
            preset,
            output,
            robots,
            order_rate,
            grid,
            seed,
        } => {
            run_generate(&preset, &output, robots, order_rate, grid, seed);
        }
        Commands::Sweep {
            base,
            sweep,
            replications,
            output_dir,
        } => {
            run_sweep(&base, &sweep, replications, &output_dir);
        }
        Commands::Compare {
            baseline,
            variant,
            replications,
            output,
        } => {
            run_compare(&baseline, &variant, replications, output.as_deref());
        }
        Commands::AbTest {
            baseline,
            variant,
            replications,
            alpha,
            output,
        } => {
            run_ab_test(&baseline, &variant, replications, alpha, output.as_deref());
        }
        Commands::Benchmark {
            suite,
            replications,
            history,
            regression_threshold,
            output,
        } => {
            run_benchmark(
                suite.as_deref(),
                replications,
                history.as_deref(),
                regression_threshold,
                output.as_deref(),
            );
        }
        Commands::StressTest {
            robots,
            order_rate,
            duration,
            grid,
            output,
        } => {
            run_stress_test(robots, order_rate, duration, &grid, output.as_deref());
        }
        Commands::ListPresets => {
            run_list_presets();
        }

        // v5: Root Cause Analysis
        Commands::Analyze {
            scenario,
            output,
            format,
            detailed,
            anomaly_threshold,
        } => {
            run_analyze(
                &scenario,
                output.as_deref(),
                &format,
                detailed,
                anomaly_threshold,
            );
        }

        // v6: Interactive Web UI
        Commands::Ui { port, open } => {
            run_ui(port, open);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_simulation(
    scenario_path: &std::path::Path,
    seed_override: Option<u64>,
    output_format: &str,
    output_dir: Option<&std::path::Path>,
    per_robot: bool,
    per_station: bool,
    heatmap: bool,
    timeseries: bool,
    trace: bool,
    attribution: bool,
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
    let mut world = build_world_from_scenario(&scenario, seed, scenario_path);

    // v3: Configure trace collector from CLI or config
    let trace_enabled = trace || scenario.metrics.trace.enabled;
    if trace_enabled {
        world.trace_collector.set_enabled(true);
    }

    // v6: Enable attribution tracking for delay analysis
    if attribution {
        world.attribution_collector.enable();
        println!("Attribution tracking enabled for delay analysis");
    }

    // Create and run simulation
    let mut runner = waremax_sim::SimulationRunner::new(
        world,
        scenario.simulation.duration_minutes,
        scenario.simulation.warmup_minutes,
    );

    // Determine what to include in report
    let include_robots = per_robot || scenario.metrics.per_robot_breakdown;
    let include_stations = per_station || scenario.metrics.per_station_breakdown;
    let include_heatmap = heatmap || scenario.metrics.generate_heatmap;
    let include_reliability = true; // Always include if there are maintenance stations

    // Run simulation and generate full report
    runner.run();
    let report = runner.generate_full_report(
        include_robots,
        include_stations,
        include_reliability,
        include_heatmap,
    );

    // Output results to console
    match output_format {
        "json" => println!("{}", report.to_json()),
        _ => println!("{}", report.summary()),
    }

    // v3: Export to files if output directory specified
    if let Some(out_dir) = output_dir {
        use waremax_metrics::ExportOptions;

        let options = ExportOptions {
            robots: include_robots,
            stations: include_stations,
            heatmap: include_heatmap,
            timeseries,
            trace: trace_enabled,
            json: true,
        };

        let world = runner.world();
        let timeseries_ref = if timeseries {
            Some(&world.time_series)
        } else {
            None
        };
        let trace_ref = if trace_enabled {
            Some(&world.trace_collector)
        } else {
            None
        };

        match waremax_metrics::write_exports(out_dir, &report, timeseries_ref, trace_ref, &options)
        {
            Ok(()) => {
                println!("\nExports written to: {}", out_dir.display());
                if options.json {
                    println!("  - report.json");
                }
                if options.robots && report.robot_reports.is_some() {
                    println!("  - robots.csv");
                }
                if options.stations && report.station_reports.is_some() {
                    println!("  - stations.csv");
                }
                if options.heatmap && report.heatmap.is_some() {
                    println!("  - node_congestion.csv");
                    println!("  - edge_congestion.csv");
                }
                if options.timeseries {
                    println!("  - timeseries.csv");
                }
                if options.trace && !world.trace_collector.is_empty() {
                    println!("  - trace.csv");
                }
            }
            Err(e) => {
                eprintln!("Error writing exports: {}", e);
            }
        }
    }
}

fn validate_scenario(scenario_path: &std::path::Path) {
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
        println!(
            "  Note: Map file '{}' not found, skipping map validation",
            scenario.map.file
        );
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
            println!(
                "  Duration: {} minutes",
                scenario.simulation.duration_minutes
            );
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
    _scenario_path: &std::path::Path,
) -> waremax_sim::World {
    use waremax_core::{EdgeId, NodeId, RobotId, StationId};
    use waremax_entities::{
        BatteryConsumptionModel, ChargingStation, MaintenanceStation, Robot, ServiceTimeModel,
        Station, StationType,
    };
    use waremax_map::{Edge, Node, NodeType, Router, TrafficManager, WarehouseMap};
    use waremax_metrics::TimeSeriesCollector;

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
            let node_type = if id == 0 {
                NodeType::StationPick
            } else {
                NodeType::Aisle
            };
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

    // Add robots
    for i in 0..scenario.robots.count {
        let start_node = i % (grid_size * grid_size) as u32;
        let mut robot = if scenario.robots.battery.enabled {
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

        // v3: Enable maintenance if configured
        if scenario.robots.maintenance.enabled {
            robot.enable_maintenance(scenario.robots.maintenance.interval_hours * 3600.0);
        }

        // v3: Enable random failures if configured
        if scenario.robots.failure.enabled {
            robot.enable_failures(scenario.robots.failure.mtbf_hours * 3600.0);
        }

        world.robots.insert(RobotId(i), robot);
    }

    if scenario.robots.battery.enabled {
        println!(
            "Battery: enabled (capacity: {} Wh, min SOC: {:.0}%)",
            scenario.robots.battery.capacity_wh,
            scenario.robots.battery.min_soc * 100.0
        );
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

        let service_time = match station_cfg.service_time_s.distribution.as_str() {
            "lognormal" => ServiceTimeModel::lognormal(
                station_cfg.service_time_s.base,
                station_cfg.service_time_s.base_stddev,
                station_cfg.service_time_s.per_item,
                station_cfg.service_time_s.per_item_stddev,
            ),
            "exponential" => ServiceTimeModel::exponential(station_cfg.service_time_s.base),
            "uniform" => ServiceTimeModel::uniform(
                station_cfg.service_time_s.min_s,
                station_cfg.service_time_s.max_s,
                station_cfg.service_time_s.per_item,
            ),
            _ => ServiceTimeModel::constant(
                station_cfg.service_time_s.base,
                station_cfg.service_time_s.per_item,
            ),
        };

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

    // Create charging stations from config
    for cfg in &scenario.charging_stations {
        let node_id: u32 = cfg.node.parse().unwrap_or(0);
        let station_id = world.next_charging_id();
        let mut station = ChargingStation::new(
            station_id,
            cfg.id.clone(),
            NodeId(node_id),
            cfg.bays,
            cfg.charge_rate_w,
        );
        if let Some(capacity) = cfg.queue_capacity {
            station = station.with_queue_capacity(capacity);
        }
        world.charging_stations.insert(station_id, station);
    }

    if !scenario.charging_stations.is_empty() {
        println!("Charging Stations: {}", scenario.charging_stations.len());
    }

    // v3: Create maintenance stations from config
    for cfg in &scenario.maintenance_stations {
        let node_id: u32 = cfg.node.parse().unwrap_or(0);
        let station_id = world.next_maintenance_id();

        let repair_time_model = match cfg.repair_time.distribution.as_str() {
            "lognormal" => ServiceTimeModel::lognormal(
                cfg.repair_time.base,
                cfg.repair_time.base_stddev,
                cfg.repair_time.per_item,
                cfg.repair_time.per_item_stddev,
            ),
            "exponential" => ServiceTimeModel::exponential(cfg.repair_time.base),
            "uniform" => ServiceTimeModel::uniform(
                cfg.repair_time.min_s,
                cfg.repair_time.max_s,
                cfg.repair_time.per_item,
            ),
            _ => ServiceTimeModel::constant(cfg.repair_time.base, cfg.repair_time.per_item),
        };

        let mut station = MaintenanceStation::new(
            station_id,
            cfg.id.clone(),
            NodeId(node_id),
            cfg.bays,
            cfg.maintenance_duration_s,
        )
        .with_repair_time_model(repair_time_model);

        if let Some(capacity) = cfg.queue_capacity {
            station = station.with_queue_capacity(capacity);
        }
        world.maintenance_stations.insert(station_id, station);
    }

    if !scenario.maintenance_stations.is_empty() {
        println!(
            "Maintenance Stations: {}",
            scenario.maintenance_stations.len()
        );
    }

    // v3: Print maintenance/failure settings
    if scenario.robots.maintenance.enabled {
        println!(
            "Maintenance: enabled (interval: {:.1} hours)",
            scenario.robots.maintenance.interval_hours
        );
    }
    if scenario.robots.failure.enabled {
        println!(
            "Failures: enabled (MTBF: {:.1} hours)",
            scenario.robots.failure.mtbf_hours
        );
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

    // Create policies from config (including traffic policy)
    world.policies =
        waremax_sim::create_policies_with_traffic(&scenario.policies, &scenario.traffic);
    let (alloc, station, batch, prio, traffic) = world.policies.all_names();
    println!("Policies:");
    println!("  Task Allocation: {}", alloc);
    println!("  Station Assignment: {}", station);
    println!("  Batching: {}", batch);
    println!("  Priority: {}", prio);
    println!("  Traffic: {}", traffic);

    // Set metrics sample interval from config
    world.metrics_sample_interval_s = scenario.metrics.sample_interval_s;
    world.time_series = TimeSeriesCollector::new(scenario.metrics.sample_interval_s);

    // v2: Set up reservation-based traffic control
    world.reservation_manager.enabled = scenario.traffic.reservation_enabled;
    if scenario.traffic.reservation_enabled {
        println!(
            "  Reservation System: enabled (lookahead: {:.1}s)",
            scenario.traffic.reservation_lookahead_s
        );
    }

    // Initialize demo inventory with 20 SKUs
    world.init_demo_inventory(20);

    world
}

fn build_demo_world(seed: u64, num_robots: usize, order_rate: f64) -> waremax_sim::World {
    use waremax_core::{EdgeId, NodeId, RobotId, StationId};
    use waremax_entities::{Robot, ServiceTimeModel, Station, StationType};
    use waremax_map::{Edge, Node, NodeType, Router, TrafficManager, WarehouseMap};

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

            map.add_node(Node::new(
                NodeId(id as u32),
                format!("N{}", id),
                x,
                y,
                node_type,
            ));
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
                map.add_edge(Edge::new(
                    EdgeId(edge_id),
                    NodeId(id as u32),
                    NodeId(neighbor as u32),
                    spacing,
                ));
                edge_id += 1;
            }

            // Connect to bottom neighbor
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
    // Allow 2 robots per edge/node to reduce congestion in demo
    world.traffic = TrafficManager::new(2, 2);

    // Add robots
    for i in 0..num_robots {
        let start_node = (i % (grid_size * grid_size)) as u32;
        let robot = Robot::new(RobotId(i as u32), NodeId(start_node), 1.5, 25.0);
        world.robots.insert(RobotId(i as u32), robot);
    }

    // Add a pick station at node 0
    let service_time = ServiceTimeModel::constant(5.0, 2.0);
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
    use waremax_sim::distributions::{
        DistributionSet, ExponentialArrivals, NegBinomialLines, ZipfSkus,
    };
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

// =============================================================================
// v4: Testing & Benchmarking Functions
// =============================================================================

fn run_generate(
    preset: &str,
    output: &PathBuf,
    robots_override: Option<u32>,
    order_rate_override: Option<f64>,
    grid_override: Option<String>,
    seed_override: Option<u64>,
) {
    use waremax_testing::ScenarioPreset;

    println!("Generating scenario from preset: {}", preset);

    // Get the preset
    let preset_enum = match ScenarioPreset::from_name(preset) {
        Some(p) => p,
        None => {
            eprintln!("Unknown preset: {}", preset);
            eprintln!("Available presets:");
            for p in ScenarioPreset::all() {
                eprintln!("  {} - {}", p.name(), p.description());
            }
            std::process::exit(1);
        }
    };

    // Start with the preset builder and apply overrides
    let mut builder = preset_enum.builder();

    if let Some(robots) = robots_override {
        builder = builder.robots(robots);
    }

    if let Some(rate) = order_rate_override {
        builder = builder.order_rate(rate);
    }

    if let Some(grid) = grid_override {
        let parts: Vec<&str> = grid.split('x').collect();
        if parts.len() == 2 {
            if let (Ok(rows), Ok(cols)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                builder = builder.grid(rows, cols);
            } else {
                eprintln!("Invalid grid format. Use 'NxM' (e.g., '10x10')");
                std::process::exit(1);
            }
        } else {
            eprintln!("Invalid grid format. Use 'NxM' (e.g., '10x10')");
            std::process::exit(1);
        }
    }

    if let Some(seed) = seed_override {
        builder = builder.seed(seed);
    }

    let config = builder.build();

    // Serialize to YAML
    let yaml = match serde_yaml::to_string(&config) {
        Ok(y) => y,
        Err(e) => {
            eprintln!("Failed to serialize config: {}", e);
            std::process::exit(1);
        }
    };

    // Write to file
    if let Err(e) = std::fs::write(output, yaml) {
        eprintln!("Failed to write output file: {}", e);
        std::process::exit(1);
    }

    println!("Scenario generated: {}", output.display());
    println!(
        "  Preset: {} - {}",
        preset_enum.name(),
        preset_enum.description()
    );
    println!("  Robots: {}", config.robots.count);
    println!("  Stations: {}", config.stations.len());
    println!(
        "  Order Rate: {:.1} orders/hr",
        config.orders.arrival_process.rate_per_min * 60.0
    );
    println!("  Duration: {:.1} min", config.simulation.duration_minutes);
}

fn run_sweep(
    base: &std::path::Path,
    sweep_spec: &str,
    replications: u32,
    output_dir: &std::path::Path,
) {
    use waremax_testing::{BatchRunner, ScenarioComparator, SweepGenerator};

    println!("Running parameter sweep...");
    println!("  Base scenario: {}", base.display());
    println!("  Sweep: {}", sweep_spec);
    println!("  Replications: {}", replications);

    // Load base scenario
    let base_config = match waremax_config::ScenarioConfig::from_file(&base.to_string_lossy()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load base scenario: {}", e);
            std::process::exit(1);
        }
    };

    // Parse sweep specification (e.g., "robots:5,10,15,20" or "order_rate:30,60,90")
    let parts: Vec<&str> = sweep_spec.split(':').collect();
    if parts.len() != 2 {
        eprintln!("Invalid sweep format. Use 'param:v1,v2,v3' (e.g., 'robots:5,10,15')");
        std::process::exit(1);
    }

    let param = parts[0];
    let values: Vec<&str> = parts[1].split(',').collect();

    // Create sweep generator from base config
    let mut generator = SweepGenerator::from_config(base_config);

    match param {
        "robots" => {
            let vals: Vec<u32> = values.iter().filter_map(|v| v.parse().ok()).collect();
            generator = generator.sweep_robot_count(&vals);
        }
        "order_rate" => {
            let vals: Vec<f64> = values.iter().filter_map(|v| v.parse().ok()).collect();
            generator = generator.sweep_order_rate(&vals);
        }
        "stations" => {
            let vals: Vec<u32> = values.iter().filter_map(|v| v.parse().ok()).collect();
            generator = generator.sweep_station_count(&vals);
        }
        _ => {
            eprintln!(
                "Unknown sweep parameter: {}. Supported: robots, order_rate, stations",
                param
            );
            std::process::exit(1);
        }
    }

    // Add seed replications
    generator = generator.sweep_seeds(replications);

    // Generate scenarios
    let scenarios = generator.generate();
    println!("Generated {} scenarios", scenarios.len());

    // Create output directory
    if let Err(e) = std::fs::create_dir_all(output_dir) {
        eprintln!("Failed to create output directory: {}", e);
        std::process::exit(1);
    }

    // Run all scenarios
    let runner = BatchRunner::new(scenarios);
    let results = runner.run();

    // Group results by config (excluding seed)
    let mut comparator = ScenarioComparator::new();
    for result in &results {
        // Extract base label (without seed suffix)
        let base_label = result.label.split("_seed=").next().unwrap_or(&result.label);
        comparator.add_results(base_label, vec![result.clone()]);
    }

    // Generate ranking
    let ranking = comparator.rank_by_throughput();

    println!("\nResults by Throughput:");
    println!("{:-<60}", "");
    for (i, (label, stats)) in ranking.iter().enumerate() {
        println!(
            "{:2}. {:30} {:.1} ± {:.1} orders/hr",
            i + 1,
            label,
            stats.mean,
            stats.std_dev
        );
    }

    // Save results
    let results_path = output_dir.join("sweep_results.json");
    let results_json = serde_json::to_string_pretty(
        &results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "label": r.label,
                    "seed": r.seed,
                    "throughput": r.throughput(),
                    "p95_cycle_time": r.p95_cycle_time(),
                    "robot_utilization": r.robot_utilization(),
                    "station_utilization": r.station_utilization(),
                    "duration_ms": r.duration_ms,
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();

    if let Err(e) = std::fs::write(&results_path, results_json) {
        eprintln!("Failed to write results: {}", e);
    } else {
        println!("\nResults saved to: {}", results_path.display());
    }
}

fn run_compare(
    baseline_path: &std::path::Path,
    variant_path: &std::path::Path,
    replications: u32,
    output: Option<&std::path::Path>,
) {
    use waremax_testing::{BatchRunner, ScenarioComparator};

    println!("Comparing configurations...");
    println!("  Baseline: {}", baseline_path.display());
    println!("  Variant: {}", variant_path.display());
    println!("  Replications: {}", replications);

    // Load configs
    let baseline_config =
        match waremax_config::ScenarioConfig::from_file(&baseline_path.to_string_lossy()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to load baseline: {}", e);
                std::process::exit(1);
            }
        };

    let variant_config =
        match waremax_config::ScenarioConfig::from_file(&variant_path.to_string_lossy()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to load variant: {}", e);
                std::process::exit(1);
            }
        };

    // Run baseline with multiple seeds
    let seeds: Vec<u64> = (0..replications).map(|i| 1000 + i as u64).collect();

    let baseline_scenarios: Vec<(String, waremax_config::ScenarioConfig)> = seeds
        .iter()
        .map(|&seed| {
            let mut config = baseline_config.clone();
            config.seed = seed;
            (format!("baseline_seed={}", seed), config)
        })
        .collect();

    let variant_scenarios: Vec<(String, waremax_config::ScenarioConfig)> = seeds
        .iter()
        .map(|&seed| {
            let mut config = variant_config.clone();
            config.seed = seed;
            (format!("variant_seed={}", seed), config)
        })
        .collect();

    println!("\nRunning {} baseline simulations...", replications);
    let baseline_runner = BatchRunner::new(baseline_scenarios);
    let baseline_results = baseline_runner.run();

    println!("Running {} variant simulations...", replications);
    let variant_runner = BatchRunner::new(variant_scenarios);
    let variant_results = variant_runner.run();

    // Compare
    let mut comparator = ScenarioComparator::new();
    comparator.add_results("baseline", baseline_results);
    comparator.add_results("variant", variant_results);

    if let Some(report) = comparator.compare("baseline", "variant") {
        println!("\n{}", report.to_string());

        // Save if output specified
        if let Some(out) = output {
            let json = serde_json::to_string_pretty(&report).unwrap();
            if let Err(e) = std::fs::write(out, json) {
                eprintln!("Failed to write results: {}", e);
            } else {
                println!("Results saved to: {}", out.display());
            }
        }
    }
}

fn run_ab_test(
    baseline_path: &std::path::Path,
    variant_path: &std::path::Path,
    replications: u32,
    alpha: f64,
    output: Option<&std::path::Path>,
) {
    use waremax_testing::{ABTestConfig, ABTestRunner};

    println!("Running A/B test...");
    println!("  Baseline: {}", baseline_path.display());
    println!("  Variant: {}", variant_path.display());
    println!("  Replications: {}", replications);
    println!("  Alpha: {}", alpha);

    // Load configs
    let baseline_config =
        match waremax_config::ScenarioConfig::from_file(&baseline_path.to_string_lossy()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to load baseline: {}", e);
                std::process::exit(1);
            }
        };

    let variant_config =
        match waremax_config::ScenarioConfig::from_file(&variant_path.to_string_lossy()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to load variant: {}", e);
                std::process::exit(1);
            }
        };

    // Configure and run A/B test
    let config = ABTestConfig::new(baseline_config, variant_config)
        .replications(replications)
        .alpha(alpha);

    let runner = ABTestRunner::new(config);

    println!(
        "\nRunning A/B test ({} replications per variant)...",
        replications
    );
    let result = runner.run();

    // Print results
    println!("\n{}", result.summary());

    // Save if output specified
    if let Some(out) = output {
        let json = serde_json::to_string_pretty(&result).unwrap();
        if let Err(e) = std::fs::write(out, json) {
            eprintln!("Failed to write results: {}", e);
        } else {
            println!("Results saved to: {}", out.display());
        }
    }
}

fn run_benchmark(
    suite_path: Option<&std::path::Path>,
    replications: u32,
    history_path: Option<&std::path::Path>,
    regression_threshold: f64,
    output: Option<&std::path::Path>,
) {
    use waremax_testing::{BenchmarkHistory, BenchmarkSuite, ScenarioPreset};

    println!("Running benchmark suite...");
    println!("  Replications: {}", replications);
    println!("  Regression threshold: {}%", regression_threshold);

    // Create or load suite
    let suite = if let Some(_path) = suite_path {
        // TODO: Load custom suite from file
        eprintln!("Custom suite loading not yet implemented, using defaults");
        BenchmarkSuite::default_suite().replications(replications)
    } else {
        // Use default suite with all standard presets
        BenchmarkSuite::new("default")
            .replications(replications)
            .add_preset(ScenarioPreset::Minimal)
            .add_preset(ScenarioPreset::Quick)
            .add_preset(ScenarioPreset::Standard)
    };

    println!("\nRunning benchmarks...");
    let results = suite.run();

    // Print results
    println!("\n{}", results.summary());

    // Check against history for regressions
    if let Some(hist_path) = history_path {
        let mut history = BenchmarkHistory::load(hist_path).unwrap_or_else(|_| {
            println!("Creating new benchmark history");
            BenchmarkHistory::new(100)
        });

        let regressions = history.detect_regressions(&results, regression_threshold);
        if !regressions.is_empty() {
            println!(
                "\n{} REGRESSION(S) DETECTED vs. history:",
                regressions.len()
            );
            for reg in &regressions {
                println!(
                    "  {} - {}: {:.2} → {:.2} ({:+.1}%)",
                    reg.benchmark, reg.metric, reg.expected, reg.actual, reg.deviation_pct
                );
            }
        } else {
            println!("\nNo regressions detected vs. history");
        }

        // Add to history
        history.add(results.clone());
        if let Err(e) = history.save(hist_path) {
            eprintln!("Failed to save history: {}", e);
        } else {
            println!("History updated: {} entries", history.len());
        }
    }

    // Save results if output specified
    if let Some(out) = output {
        if let Err(e) = results.save(out) {
            eprintln!("Failed to write results: {}", e);
        } else {
            println!("Results saved to: {}", out.display());
        }
    }
}

fn run_stress_test(
    robots: u32,
    order_rate: f64,
    duration: f64,
    grid: &str,
    output: Option<&std::path::Path>,
) {
    use waremax_testing::{BatchRunner, ScenarioBuilder};

    println!("Running stress test...");
    println!("  Robots: {}", robots);
    println!("  Order rate: {:.1} orders/hr", order_rate);
    println!("  Duration: {:.1} min", duration);
    println!("  Grid: {}", grid);

    // Parse grid
    let parts: Vec<&str> = grid.split('x').collect();
    let (rows, cols) = if parts.len() == 2 {
        let r = parts[0].parse::<u32>().unwrap_or(20);
        let c = parts[1].parse::<u32>().unwrap_or(20);
        (r, c)
    } else {
        (20, 20)
    };

    // Build stress test config
    let config = ScenarioBuilder::new()
        .grid(rows, cols)
        .robots(robots)
        .pick_stations((robots / 5).max(4)) // 1 station per ~5 robots
        .station_concurrency(2)
        .order_rate(order_rate)
        .items_per_order(3.0)
        .duration(duration)
        .warmup(10.0)
        .traffic_policy("reroute_on_wait")
        .routing_algorithm("astar")
        .congestion_aware(true)
        .build();

    // Run with multiple seeds
    let scenarios = vec![
        ("stress_seed1".to_string(), {
            let mut c = config.clone();
            c.seed = 1;
            c
        }),
        ("stress_seed2".to_string(), {
            let mut c = config.clone();
            c.seed = 2;
            c
        }),
        ("stress_seed3".to_string(), {
            let mut c = config.clone();
            c.seed = 3;
            c
        }),
    ];

    println!("\nRunning 3 replications...");
    let runner = BatchRunner::new(scenarios);
    let results = runner.run();

    // Compute aggregates
    let throughputs: Vec<f64> = results.iter().map(|r| r.throughput()).collect();
    let p95s: Vec<f64> = results.iter().map(|r| r.p95_cycle_time()).collect();
    let utils: Vec<f64> = results.iter().map(|r| r.robot_utilization()).collect();

    let avg_throughput = throughputs.iter().sum::<f64>() / throughputs.len() as f64;
    let avg_p95 = p95s.iter().sum::<f64>() / p95s.len() as f64;
    let avg_util = utils.iter().sum::<f64>() / utils.len() as f64;

    println!("\nStress Test Results:");
    println!("{:=<50}", "");
    println!("  Average Throughput: {:.1} orders/hr", avg_throughput);
    println!("  Average P95 Cycle Time: {:.2} s", avg_p95);
    println!("  Average Robot Utilization: {:.1}%", avg_util * 100.0);
    println!(
        "  Avg Run Time: {:.1}s",
        results.iter().map(|r| r.duration_ms).sum::<u64>() as f64 / results.len() as f64 / 1000.0
    );

    // Check for issues
    if avg_util > 0.95 {
        println!("\nWarning: High robot utilization may indicate capacity constraints");
    }
    if avg_p95 > 300.0 {
        println!("\nWarning: High P95 cycle time may indicate bottlenecks");
    }

    // Save if output specified
    if let Some(out) = output {
        let json = serde_json::to_string_pretty(&serde_json::json!({
            "config": {
                "robots": robots,
                "order_rate": order_rate,
                "duration_min": duration,
                "grid": grid,
            },
            "results": {
                "avg_throughput": avg_throughput,
                "avg_p95_cycle_time": avg_p95,
                "avg_robot_utilization": avg_util,
            },
            "runs": results.iter().map(|r| {
                serde_json::json!({
                    "label": r.label,
                    "throughput": r.throughput(),
                    "p95_cycle_time": r.p95_cycle_time(),
                    "robot_utilization": r.robot_utilization(),
                    "duration_ms": r.duration_ms,
                })
            }).collect::<Vec<_>>(),
        }))
        .unwrap();

        if let Err(e) = std::fs::write(out, json) {
            eprintln!("Failed to write results: {}", e);
        } else {
            println!("\nResults saved to: {}", out.display());
        }
    }
}

fn run_list_presets() {
    use waremax_testing::ScenarioPreset;

    println!("Available Scenario Presets:");
    println!("{:=<70}", "");

    for preset in ScenarioPreset::all() {
        let config = preset.config();
        println!("\n{}", preset.name());
        println!("  {}", preset.description());
        println!("  Grid: inferred from stations");
        println!("  Robots: {}", config.robots.count);
        println!("  Stations: {}", config.stations.len());
        println!(
            "  Order Rate: {:.0} orders/hr",
            config.orders.arrival_process.rate_per_min * 60.0
        );
        println!(
            "  Duration: {:.0} min (warmup: {:.0} min)",
            config.simulation.duration_minutes, config.simulation.warmup_minutes
        );

        if config.robots.battery.enabled {
            println!(
                "  Battery: enabled ({:.0} Wh)",
                config.robots.battery.capacity_wh
            );
        }
        if config.robots.maintenance.enabled {
            println!(
                "  Maintenance: enabled ({:.1} hr interval)",
                config.robots.maintenance.interval_hours
            );
        }
    }
}

// v5: Root Cause Analysis command implementation
fn run_analyze(
    scenario_path: &std::path::Path,
    output_path: Option<&std::path::Path>,
    format: &str,
    detailed: bool,
    _anomaly_threshold: f64,
) {
    use waremax_analysis::{AnalyzerInput, RCAReporter, ReportFormat, RootCauseAnalyzer};
    use waremax_core::SimTime;

    println!("Running Root Cause Analysis...");
    println!("Scenario: {}", scenario_path.display());

    let path_str = scenario_path.to_string_lossy();

    // Load scenario
    let scenario = match waremax_config::ScenarioConfig::from_file(&path_str) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading scenario: {}", e);
            std::process::exit(1);
        }
    };

    let seed = scenario.seed;
    println!("Running simulation with seed: {}", seed);

    // Build world from scenario
    let mut world = build_world_from_scenario(&scenario, seed, scenario_path);

    // Enable attribution collection for RCA
    world.attribution_collector.enabled = true;

    // Create and run simulation
    let mut runner = waremax_sim::SimulationRunner::new(
        world,
        scenario.simulation.duration_minutes,
        scenario.simulation.warmup_minutes,
    );

    let report = runner.run();

    println!("\nSimulation complete.");
    println!("Orders completed: {}", report.orders_completed);
    println!("Throughput: {:.1} orders/hr", report.throughput_per_hour);

    // Get reference to world for analysis
    let world = runner.world();
    let total_time = SimTime::from_seconds(report.duration_s);

    // Collect data for analysis
    let attributions = world
        .attribution_collector
        .completed_attributions()
        .to_vec();

    // Collect congestion data from time series
    let node_congestion: Vec<_> = world
        .time_series
        .node_congestion
        .iter()
        .map(|(id, m)| {
            (
                *id,
                m.congestion_score(),
                m.wait_event_count,
                m.total_wait_time_s,
            )
        })
        .collect();

    let edge_congestion: Vec<_> = world
        .time_series
        .edge_congestion
        .iter()
        .map(|(id, m)| {
            (
                *id,
                m.congestion_score(),
                m.wait_event_count,
                m.total_wait_time_s,
            )
        })
        .collect();

    // Collect station data
    let station_data: Vec<_> = world
        .stations
        .iter()
        .map(|(id, s)| {
            let ts_data = world.time_series.station_series.get(id);
            let avg_queue = ts_data.map(|d| d.avg_queue_length()).unwrap_or(0.0);
            let max_queue = ts_data.map(|d| d.max_queue_length()).unwrap_or(0);
            (
                *id,
                s.string_id.clone(),
                s.utilization(total_time),
                avg_queue,
                max_queue,
            )
        })
        .collect();

    // Collect robot utilizations
    let robot_utilizations: Vec<_> = world
        .robots
        .iter()
        .map(|(id, r)| (*id, r.utilization(total_time)))
        .collect();

    // Collect queue time series
    let station_queue_series: Vec<_> = world
        .stations
        .iter()
        .filter_map(|(id, s)| {
            world.time_series.station_series.get(id).map(|ts| {
                let samples: Vec<_> = ts
                    .queue_length
                    .iter()
                    .map(|dp| (dp.time_s, dp.value))
                    .collect();
                (*id, s.string_id.clone(), samples)
            })
        })
        .collect();

    // Create analyzer input
    let input = AnalyzerInput {
        attributions,
        node_congestion,
        edge_congestion,
        station_data,
        charging_data: Vec::new(), // Can add if needed
        robot_utilizations,
        station_queue_series,
    };

    // Run analysis
    let mut analyzer = RootCauseAnalyzer::new();
    let rca_report = analyzer.analyze(&input);

    // Generate report
    let report_format = match format.to_lowercase().as_str() {
        "json" => ReportFormat::Json,
        "compact" | "summary" => ReportFormat::Compact,
        _ => ReportFormat::Text,
    };

    let reporter = RCAReporter::new()
        .with_format(report_format.clone())
        .with_recommendations(true)
        .with_anomalies(true)
        .with_max_bottlenecks(if detailed { 20 } else { 10 })
        .with_max_anomalies(if detailed { 20 } else { 10 });

    let output = reporter.generate(&rca_report);

    // Output report
    if let Some(path) = output_path {
        match std::fs::write(path, &output) {
            Ok(_) => println!("\nRCA report saved to: {}", path.display()),
            Err(e) => eprintln!("Failed to write report: {}", e),
        }
    } else {
        println!("\n{}", output);
    }

    // Print summary stats
    println!("\n{:=<60}", "");
    println!("Analysis Summary:");
    println!("  Health Score: {:.0}/100", rca_report.summary.health_score);
    println!("  Orders Analyzed: {}", rca_report.summary.orders_analyzed);
    println!(
        "  Primary Issue: {}",
        rca_report.summary.primary_delay_source
    );
    println!(
        "  Bottlenecks Found: {}",
        rca_report.bottleneck_analysis.summary.total_count
    );
    println!("  Anomalies Detected: {}", rca_report.summary.anomaly_count);
}

// =============================================================================
// v6: Interactive Web UI
// =============================================================================

fn run_ui(port: u16, open: bool) {
    let config = waremax_ui::ServerConfig {
        port,
        open_browser: open,
        ..Default::default()
    };

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        if let Err(e) = waremax_ui::run_server(config).await {
            eprintln!("Server error: {}", e);
            std::process::exit(1);
        }
    });
}
